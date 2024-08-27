use log::{debug, warn};
use std::collections::HashMap;
use tokio_modbus::{
    client::{rtu, tcp, Context},
    prelude::{Reader, Writer},
    Address, Quantity,
};

use tokio_serial::SerialStream;
use tokio_serial::{self, StopBits};

pub mod errors;
pub mod industrial_device;
pub mod modbus_connexion_async;
pub mod register;
pub mod types;
pub mod utils;

use crate::register::Register;
use crate::types::RegisterValue;
use crate::{
    errors::ModbusError,
    types::{ModBusContext, ModBusRegisters},
};

use crate::modbus_connexion_async::ModbusConnexionAsync;

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

impl ModbusConnexionAsync for ModbusDeviceAsync {
    async fn connect(&mut self) -> Result<(), ModbusError> {
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

    // read input registers by address
    async fn read_raw_registers(
        &mut self,
        addr: &Address,
        nb: &Quantity,
        source: &ModBusRegisters,
    ) -> Result<Vec<u16>, ModbusError> {
        debug!("read register {addr} x{nb}");
        let ctx = self
            .ctx
            .as_mut()
            .ok_or(ModbusError::DeviceNotConnectedError)?;
        let res = match source {
            ModBusRegisters::INPUT => ctx.read_input_registers(*addr, *nb),
            ModBusRegisters::HOLDING => ctx.read_holding_registers(*addr, *nb),
        }
        .await;
        match res {
            Ok(res) => match res {
                Ok(res) => return Ok(res),
                Err(err) => Err(err.into()),
            },
            Err(err) => return Err(err.into()),
        }
    }
    async fn write_raw_holding_registers(
        &mut self,
        addr: &Address,
        data: &[u16],
    ) -> Result<(), ModbusError> {
        let ctx = self
            .ctx
            .as_mut()
            .ok_or(ModbusError::DeviceNotConnectedError)?;
        let res = ctx.write_multiple_registers(*addr, data).await;
        match res {
            Ok(res) => match res {
                Ok(res) => return Ok(res),
                Err(err) => Err(err.into()),
            },
            Err(err) => return Err(err.into()),
        }
    }
    async fn read_range(
        &mut self,
        regs: &[Register],
        source: &ModBusRegisters,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError> {
        let s_reg = regs.first().unwrap();
        let e_reg = regs.last().unwrap();
        // Read the values

        let start_address = s_reg.addr;
        let read_len = e_reg.addr + e_reg.len - s_reg.addr;

        debug!("reading range {0}:{1}", start_address, read_len);

        let read_regs: Vec<u16> = self
            .read_raw_registers(&start_address, &read_len, source)
            .await?;

        // convert them to the types and make the association with the registers
        Ok(regs
            .iter()
            .filter_map(|v| {
                let start_off = v.addr - start_address;
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

    async fn read_registers(
        &mut self,
        regs: &[Register],
        source: &ModBusRegisters,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError> {
        // read registers in order of address
        let mut sorted_regs = regs.to_vec();
        sorted_regs.sort_by_key(|s| s.addr);

        // index of the start and end register for the current range
        let mut reg_range_start = 0;
        let mut reg_range_end = 0;

        let mut result: HashMap<String, RegisterValue> = HashMap::new();

        if sorted_regs.len() == 0 {
            debug!("There is no register to read");
            return Ok(HashMap::new());
        }
        // TODO: check if we can remove that
        if sorted_regs.len() == 1 {
            debug!("There is only one register to read");
            let reg = sorted_regs[0].clone();
            return Ok(self.read_range(&vec![reg], source).await?);
        }

        for (mut i, r) in sorted_regs.iter().skip(1).enumerate() {
            i = i + 1;
            // if the range is greater than the max request size we read this batch
            if r.addr - sorted_regs[reg_range_start].addr > MODBUS_MAX_READ_LEN
                || r.addr != sorted_regs[reg_range_end].addr + sorted_regs[reg_range_end].len
            {
                let read_regs_map = self
                    .read_range(&sorted_regs[reg_range_start..reg_range_end + 1], source)
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
            .read_range(&sorted_regs[reg_range_start..reg_range_end + 1], source)
            .await?;
        result.extend(read_regs_map);

        return Ok(result);
    }
    async fn read_register(
        &mut self,
        reg: &Register,
        source: &ModBusRegisters,
    ) -> Result<RegisterValue, ModbusError> {
        let res = self.read_registers(&[reg.clone()], source).await?;

        let val = res.get(&reg.name).ok_or(ModbusError::ConversionError)?;

        Ok(val.clone())
    }
    async fn read_registers_by_name(
        &mut self,
        names: &[String],
        source: &ModBusRegisters,
    ) -> Result<HashMap<std::string::String, RegisterValue>, ModbusError> {
        let registers_to_read: Vec<Register> = names
            .iter()
            .filter_map(|n| {
                let reg = match source {
                    ModBusRegisters::INPUT => self.get_input_register_by_name(n),
                    ModBusRegisters::HOLDING => self.get_holding_register_by_name(n),
                };
                if reg.is_none() {
                    warn!("Register {n} does not exist, skipping it");
                }
                reg
            })
            .collect();

        self.read_registers(&registers_to_read, source).await
    }
    async fn dump_registers(
        &mut self,
        source: &ModBusRegisters,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError> {
        let registers = match source {
            ModBusRegisters::INPUT => &self.input_registers,
            ModBusRegisters::HOLDING => &self.holding_registers,
        };

        let filtered_regs: Vec<Register> = registers
            .clone()
            .iter()
            .filter_map(|(_name, v)| match v.read {
                true => Some(v.clone()),
                false => None,
            })
            .collect();

        self.read_registers(&filtered_regs, source).await
    }

    async fn read_input_registers_by_name(
        &mut self,
        names: &[String],
    ) -> Result<HashMap<String, RegisterValue>, ModbusError> {
        self.read_registers_by_name(names, &ModBusRegisters::INPUT)
            .await
    }
    async fn read_input_registers(
        &mut self,
        regs: &[Register],
    ) -> Result<HashMap<std::string::String, RegisterValue>, ModbusError> {
        self.read_registers(regs, &ModBusRegisters::INPUT).await
    }
    async fn dump_input_registers(
        &mut self,
    ) -> Result<HashMap<std::string::String, RegisterValue>, ModbusError> {
        self.dump_registers(&ModBusRegisters::INPUT).await
    }

    async fn read_holding_registers_by_name(
        &mut self,
        names: &[String],
    ) -> Result<HashMap<String, RegisterValue>, ModbusError> {
        self.read_registers_by_name(names, &ModBusRegisters::HOLDING)
            .await
    }
    async fn read_holding_registers(
        &mut self,
        regs: &[Register],
    ) -> Result<HashMap<String, RegisterValue>, ModbusError> {
        self.read_registers(regs, &ModBusRegisters::HOLDING).await
    }
    async fn read_holding_register(
        &mut self,
        reg: &Register,
    ) -> Result<RegisterValue, ModbusError> {
        self.read_register(reg, &ModBusRegisters::HOLDING).await
    }
    async fn dump_holding_registers(
        &mut self,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError> {
        self.dump_registers(&ModBusRegisters::HOLDING).await
    }
    async fn write_holding_register(
        &mut self,
        reg: &Register,
        val: &RegisterValue,
    ) -> Result<(), ModbusError> {
        let data: Vec<u16> = val.clone().try_into()?;

        self.write_raw_holding_registers(&reg.addr, &data).await
    }
    async fn write_holding_register_by_name(
        &mut self,
        name: &str,
        val: &RegisterValue,
    ) -> Result<(), ModbusError> {
        let reg = self.get_holding_register_by_name(name).ok_or(
            ModbusError::RegisterDoesNotExistError {
                name: name.to_string(),
            },
        )?;
        self.write_holding_register(&reg, val).await
    }

    fn get_holding_register_by_name(&mut self, name: &str) -> Option<Register> {
        self.holding_registers.get(name).cloned()
    }
    fn get_input_register_by_name(&mut self, name: &str) -> Option<Register> {
        self.input_registers.get(name).cloned()
    }
}
