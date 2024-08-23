use log::{debug, warn};
use std::collections::HashMap;
use tokio_modbus::{
    client::{rtu, tcp, Context},
    prelude::{Reader, Writer},
    Address, Quantity,
};

use tokio_serial::SerialStream;
use tokio_serial::{self, StopBits};

use crate::register::Register;
use crate::types::RegisterValue;
use crate::{
    errors::{DeviceNotConnectedError, ModbusError},
    types::{ModBusContext, ModBusRegisters},
};

// maximum number of register that can be read at once (limited by the protocol)
const MODBUS_MAX_READ_LEN: u16 = 125;

#[derive(Debug)]
pub struct ModbusDeviceAsync {
    ctx: Option<Context>,
    input_registers: HashMap<String, Register>,
    holding_registers: HashMap<String, Register>,
    device: ModBusContext,
}

impl ModbusDeviceAsync {
    pub fn new(
        context: ModBusContext,
        input_registers: HashMap<String, Register>,
        holding_registers: HashMap<String, Register>,
    ) -> Self {
        ModbusDeviceAsync {
            ctx: None,
            input_registers,
            holding_registers,
            device: context,
        }
    }
}

#[trait_variant::make(ModbusConnexionAsync: Send)]
pub trait LocalModbusConnexionAsync {
    async fn connect(&mut self) -> Result<(), std::io::Error>;
    async fn read_raw_input_registers(
        &mut self,
        addr: Address,
        nb: Quantity,
    ) -> Result<Vec<u16>, ModbusError>;

    async fn read_input_registers_by_name(
        &mut self,
        names: Vec<String>,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError>;
    async fn read_input_registers(
        &mut self,
        regs: Vec<Register>,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError>;

    async fn read_range(
        &mut self,
        regs: Vec<Register>,
        source: ModBusRegisters,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError>;
    async fn read_register(
        &mut self,
        regs: Vec<Register>,
        source: ModBusRegisters,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError>;

    async fn dump_input_registers(&mut self)
        -> Result<HashMap<String, RegisterValue>, ModbusError>;

    async fn read_raw_holding_registers(
        &mut self,
        addr: Address,
        nb: Quantity,
    ) -> Result<Vec<u16>, ModbusError>;
    async fn read_holding_registers_by_name(
        &mut self,
        names: Vec<String>,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError>;
    async fn read_holding_registers(
        &mut self,
        regs: Vec<Register>,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError>;
    async fn read_holding_register(&mut self, regs: Register)
        -> Result<RegisterValue, ModbusError>;

    async fn dump_holding_registers(
        &mut self,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError>;

    fn get_holding_register_by_name(&mut self, name: String) -> Option<&Register>;
    async fn write_raw_holding_registers(
        &mut self,
        addr: Address,
        data: Vec<u16>,
    ) -> Result<(), ModbusError>;
    async fn write_holding_register(
        &mut self,
        reg: Register,
        val: RegisterValue,
    ) -> Result<(), ModbusError>;
}

impl LocalModbusConnexionAsync for ModbusDeviceAsync {
    // read input registers by address
    async fn read_raw_input_registers(
        &mut self,
        addr: Address,
        nb: Quantity,
    ) -> Result<Vec<u16>, ModbusError> {
        debug!("read register {addr} x{nb}");
        if self.ctx.is_none() {
            return Err(DeviceNotConnectedError.into());
        }
        let res = self
            .ctx
            .as_mut()
            .unwrap()
            .read_input_registers(addr, nb)
            .await;
        match res {
            Ok(res) => match res {
                Ok(res) => return Ok(res),
                Err(err) => Err(err.into()),
            },
            Err(err) => return Err(err.into()),
        }
    }

    async fn read_input_registers_by_name(
        &mut self,
        names: Vec<String>,
    ) -> Result<HashMap<std::string::String, RegisterValue>, ModbusError> {
        let registers_to_read: Vec<Register> = names
            .iter()
            .filter_map(|n| match self.input_registers.get(n) {
                Some(reg) => Some(reg.to_owned()),
                None => {
                    warn!("Register {n} does not exist, skipping it");
                    None
                }
            })
            .collect();
        self.read_input_registers(registers_to_read).await
    }

    async fn read_range(
        &mut self,
        regs: Vec<Register>,
        source: ModBusRegisters,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError> {
        let s_reg = regs.first().unwrap();
        let e_reg = regs.last().unwrap();
        // Read the values
        debug!(
            "reading range {0}:{1}",
            s_reg.addr,
            e_reg.addr + e_reg.len - s_reg.addr
        );
        let read_regs: Vec<u16> = match source {
            ModBusRegisters::INPUT => {
                self.read_raw_input_registers(s_reg.addr, e_reg.addr + e_reg.len - s_reg.addr)
                    .await?
            }
            ModBusRegisters::HOLDING => {
                self.read_raw_holding_registers(s_reg.addr, e_reg.addr + e_reg.len - s_reg.addr)
                    .await?
            }
        };

        // convert them to the types and make the association with the registers
        Ok(regs
            .iter()
            .filter_map(|v| {
                let start_off = v.addr - s_reg.addr;
                let value: Vec<u16> =
                    read_regs[start_off.into()..(start_off + v.len).into()].to_vec();
                match (value, v.data_type).try_into() {
                    Ok(res) => Some((v.name.to_owned(), res)),
                    Err(err) => {
                        warn!(
                            "There was an error converting field {0} dropping it ({err:?})",
                            v.name
                        );
                        None
                    }
                }
            })
            .collect())
    }

    async fn read_register(
        &mut self,
        mut regs: Vec<Register>,
        source: ModBusRegisters,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError> {
        // read registers in order of address
        regs.sort_by_key(|s| s.addr);

        // index of the start and end register for the current range
        let mut reg_range_start = 0;
        let mut reg_range_end = 0;

        let mut result: HashMap<String, RegisterValue> = HashMap::new();

        if regs.len() == 0 {
            debug!("There is no register to read");
            return Ok(HashMap::new());
        }
        // TODO: check if we can remove that
        if regs.len() == 1 {
            debug!("There is only one register to read");
            let reg = regs[0].clone();
            return Ok(self.read_range(vec![reg], source).await?);
        }

        for (mut i, r) in regs.iter().skip(1).enumerate() {
            i = i + 1;
            // if the range is greater than the max request size we read this batch
            if r.addr - regs[reg_range_start].addr > MODBUS_MAX_READ_LEN
                || r.addr != regs[reg_range_end].addr + regs[reg_range_end].len
            {
                let read_regs_map = self
                    .read_range(
                        regs[reg_range_start..reg_range_end + 1].to_vec(),
                        source.clone(),
                    )
                    .await?;
                // merge it with the result
                result.extend(read_regs_map);

                // reset range
                reg_range_start = i;
            }
            reg_range_end = i;
        }
        // read the last batch
        let read_regs_map = self
            .read_range(
                regs[reg_range_start..reg_range_end + 1].to_vec(),
                source.clone(),
            )
            .await?;
        result.extend(read_regs_map);

        return Ok(result);
    }

    async fn read_input_registers(
        &mut self,
        regs: Vec<Register>,
    ) -> Result<HashMap<std::string::String, RegisterValue>, ModbusError> {
        self.read_register(regs, ModBusRegisters::INPUT).await
    }

    async fn dump_input_registers(
        &mut self,
    ) -> Result<HashMap<std::string::String, RegisterValue>, ModbusError> {
        let registers = self.input_registers.to_owned();
        let keys: Vec<String> = registers.into_keys().collect();
        self.read_input_registers_by_name(keys).await
    }

    async fn read_raw_holding_registers(
        &mut self,
        addr: Address,
        nb: Quantity,
    ) -> Result<Vec<u16>, ModbusError> {
        debug!("read register {addr} x{nb}");
        if self.ctx.is_none() {
            return Err(DeviceNotConnectedError.into());
        }
        let res = self
            .ctx
            .as_mut()
            .unwrap()
            .read_holding_registers(addr, nb)
            .await;
        match res {
            Ok(res) => match res {
                Ok(res) => return Ok(res),
                Err(err) => Err(err.into()),
            },
            Err(err) => return Err(err.into()),
        }
    }

    async fn read_holding_registers_by_name(
        &mut self,
        names: Vec<String>,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError> {
        let registers_to_read: Vec<Register> = names
            .iter()
            .filter_map(|n| match self.holding_registers.get(n) {
                Some(reg) => Some(reg.to_owned()),
                None => {
                    warn!("Register {n} does not exist, skipping it");
                    None
                }
            })
            .collect();
        self.read_holding_registers(registers_to_read).await
    }
    async fn read_holding_registers(
        &mut self,
        regs: Vec<Register>,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError> {
        self.read_register(regs, ModBusRegisters::HOLDING).await
    }
    async fn read_holding_register(&mut self, reg: Register) -> Result<RegisterValue, ModbusError> {
        match self
            .read_register(vec![reg.clone()], ModBusRegisters::HOLDING)
            .await
        {
            Ok(val) => Ok(*val.get(&reg.name).unwrap()),
            Err(err) => Err(err),
        }
    }

    async fn dump_holding_registers(
        &mut self,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError> {
        let registers = self.holding_registers.to_owned();

        let keys: Vec<String> = registers
            .iter()
            .filter_map(|v| match v.1.read {
                true => Some(v.0.to_owned()),
                false => None,
            })
            .collect();
        self.read_holding_registers_by_name(keys).await
    }

    fn get_holding_register_by_name(&mut self, name: String) -> Option<&Register> {
        self.holding_registers.get(&name)
    }

    async fn write_raw_holding_registers(
        &mut self,
        addr: Address,
        data: Vec<u16>,
    ) -> Result<(), ModbusError> {
        if self.ctx.is_none() {
            return Err(DeviceNotConnectedError.into());
        }
        let res = self
            .ctx
            .as_mut()
            .unwrap()
            .write_multiple_registers(addr, &data)
            .await;
        match res {
            Ok(res) => match res {
                Ok(res) => return Ok(res),
                Err(err) => Err(err.into()),
            },
            Err(err) => return Err(err.into()),
        }
    }

    async fn write_holding_register(
        &mut self,
        reg: Register,
        val: RegisterValue,
    ) -> Result<(), ModbusError> {
        self.write_raw_holding_registers(reg.addr, val.try_into()?)
            .await
    }

    async fn connect(&mut self) -> Result<(), std::io::Error> {
        match &self.device {
            ModBusContext::TCP(ctx) => {
                self.ctx = Some(tcp::connect(ctx.addr).await?);
            }
            ModBusContext::RTU(ctx) => {
                let builder = tokio_serial::new(ctx.port.clone(), ctx.speed)
                    .stop_bits(StopBits::Two)
                    .parity(tokio_serial::Parity::None)
                    .data_bits(tokio_serial::DataBits::Eight);
                let port = SerialStream::open(&builder).unwrap();

                self.ctx = Some(rtu::attach_slave(port, ctx.slave));
                debug!("Connected to devices {0:?}", self.ctx);
            }
        }
        Ok(())
    }
}
