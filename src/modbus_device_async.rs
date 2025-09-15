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
/// The `ModbusDeviceAsync` struct represents a Modbus device with input and holding registers in Rust.
/// 
/// Properties:
/// 
/// * `ctx`: It is used to store the context information related to the Modbus device. The `Option` type allows this field to be either
/// `Some(Context)` or `None`, providing flexibility in handling the presence or absence of the context
/// information
/// 
/// * `input_registers`: A HashMap that stores input registers for the Modbus device.
///  Each input register is identified by a unique String key and is associated with a `Register` struct.
/// 
/// * `holding_registers`: The `holding_registers` property in the `ModbusDeviceAsync` struct is a
/// HashMap that stores holding registers for the Modbus device. Holding registers are used in Modbus
/// communication to store data such as control values, setpoints, and other parameters that need to be
/// read from or written to by.
/// 
/// * `device`:  the configuration related to a Modbus device.
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
    
    /// The function `connect` establishes a connection to a Modbus device either over TCP or RTU
    /// protocol in Rust, handling different contexts accordingly.
    /// creation of the element to create the connextion and connect to it
    /// Returns:
    /// 
    /// The `connect` function returns a `Result` enum with the possible outcomes being `Ok(())` if the
    /// connection is successful or an `Err(ModbusError)` if an error occurs during the connection
    /// process.
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
 
    /// This function reads input registers by address.
    /// 
    /// Arguments:
    /// 
    /// * `addr`: The address of the Modbus register from which you want to start to read data.
    /// * `nb`: The `nb` parameter in the `read_raw_registers` function represents the number of
    /// registers to read starting from the specified address (`addr`).
    /// * `source`: specifies whether to read from input registers or holding registers.
    /// 
    /// Returns:
    /// 
    /// The function `read_raw_registers` returns a `Result` containing a vector of `u16` values if the
    /// operation is successful, or a `ModbusError` if there is an error during the process.
    async fn read_raw_registers(
        &mut self,
        addr: &Address,
        nb: &Quantity,
        source: &ModBusRegisters,
    ) -> Result<Vec<u16>, ModbusError> {
        debug!("read register {addr} x{nb}");
        // transformation to Ok if there is a Value to ctx
        let ctx = self
            .ctx
            .as_mut()
            .ok_or(ModbusError::DeviceNotConnectedError)?;  
        // recuperation of value
        let res = match source {
            ModBusRegisters::INPUT => ctx.read_input_registers(*addr, *nb),
            ModBusRegisters::HOLDING => ctx.read_holding_registers(*addr, *nb),
        }
        .await;
        // see if there is any error
        match res {
            Ok(res) => match res {
                Ok(res) => return Ok(res),
                Err(err) => Err(err.into()),
            },
            Err(err) => return Err(err.into()),
        }
    }

    /// This function writes input registers by address.
    /// 
    /// Arguments:
    /// 
    /// * `addr`: The address of the Modbus register from which you want to write data.
    /// * `data`: the value you want to write in the data
    /// 
    /// Returns:
    /// 
    /// The function `write_raw_holding_registers` returns a `Result` if the
    /// operation is successful, or a `ModbusError` if there is an error during the process.
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

    /// The `read_range` function in Rust reads a range of Modbus registers, converts the data types,
    /// and returns a HashMap of register values.
    /// 
    /// Arguments:
    /// 
    /// * `regs`: the list of regiister need to be read
    /// * `source`: specifies whether to read from input registers or holding registers.
    /// 
    /// Returns:
    /// 
    /// The `read_range` function returns a `Result` containing a `HashMap<String, RegisterValue>` or a
    /// `ModbusError`. The `HashMap` contains the converted values associated with the registers
    /// specified in the input `regs` slice.
    async fn read_range(
        &mut self,
        regs: &[Register],
        source: &ModBusRegisters,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError> {
        let s_reg = regs.first().unwrap();
        let e_reg = regs.last().unwrap();

        // definition between the start register and the end register
        let start_address = s_reg.addr;
        let read_len = e_reg.addr + e_reg.len - s_reg.addr;

        debug!("reading range {0}:{1}", start_address, read_len);

        // get the data
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

    /// The function `read_registers` reads a list of registers in order of address and returns a
    /// hashmap of register values.
    /// 
    /// Arguments:
    /// 
    /// * `regs`: the list of register need to be read
    /// * `source`: specifies whether to read from input registers or holding registers.
    /// 
    /// Returns:
    /// 
    /// The function `read_registers` returns a `Result` containing a `HashMap<String, RegisterValue>`
    /// on success or a `ModbusError` on failure.
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

    /// The `read_register` function in Rust reads a register value from a Modbus source asynchronously.
    /// 
    /// Arguments:
    /// 
    /// * `reg`:  the register to be read.
    /// * `source`: the device source from which the register value should be read.
    /// 
    /// Returns:
    /// 
    /// The `read_register` function returns a `Result` containing a `RegisterValue` if successful, or a
    /// `ModbusError` if an error occurs.
    async fn read_register(
        &mut self,
        reg: &Register,
        source: &ModBusRegisters,
    ) -> Result<RegisterValue, ModbusError> {
        let res = self.read_registers(&[reg.clone()], source).await?;

        let val = res.get(&reg.name).ok_or(ModbusError::ConversionError)?;

        Ok(val.clone())
    }

    /// This Rust function reads registers by name from a Modbus source and returns a HashMap of
    /// register values.
    /// 
    /// Arguments:
    /// 
    /// * `reg`:  the register's name to be read.
    /// * `source`: the device source from which the register value should be read.
    /// 
    /// Returns:
    /// 
    /// The `read_registers_by_name` function returns a `Result` containing a `HashMap` with keys of
    /// type `std::string::String` and values of type `RegisterValue`, or a `ModbusError` if an error
    /// occurs during the operation.
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

    /// The `dump_registers` function in Rust asynchronously reads and returns filtered register values
    /// based on the source ModBusRegisters type.
    /// 
    /// Arguments:
    /// 
    /// * `source`: specifies whether to read from input registers or holding registers.
    /// 
    /// Returns:
    /// 
    /// The `dump_registers` function returns a `Result` containing a `HashMap<String, RegisterValue>`
    /// on success or a `ModbusError` on failure.
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





    /// The function `read_input_registers_by_name` reads input registers by name in Rust
    /// asynchronously.
    /// 
    /// Arguments:
    /// 
    /// * `names`: list of names of the registers to read.
    /// 
    /// Returns:
    /// 
    /// The function `read_input_registers_by_name` is returning a `Result` containing a
    /// `HashMap<String, RegisterValue>` on success or a `ModbusError` on failure.
    async fn read_input_registers_by_name(
        &mut self,
        names: &[String],
    ) -> Result<HashMap<String, RegisterValue>, ModbusError> {
        self.read_registers_by_name(names, &ModBusRegisters::INPUT)
            .await
    }

    /// The function `read_input_registers` reads input registers asynchronously and returns a HashMap
    /// of register values.
    /// 
    /// Arguments:
    /// 
    /// * `regs`: the registers to be read from the Modbus device.
    /// 
    /// Returns:
    /// 
    /// This function `read_input_registers` is returning a `Result` containing a `HashMap` with keys of
    /// type `String` and values of type `RegisterValue`, or a `ModbusError` if an error occurs during
    /// the operation.
    async fn read_input_registers(
        &mut self,
        regs: &[Register],
    ) -> Result<HashMap<std::string::String, RegisterValue>, ModbusError> {
        self.read_registers(regs, &ModBusRegisters::INPUT).await
    }

    /// The `dump_input_registers` function dumps input registers and returns a
    /// HashMap of register values or a ModbusError.
    /// 
    /// Returns:
    /// 
    /// The `dump_input_registers` function is returning a `Result` containing a `HashMap` with keys of
    /// type `String` and values of type `RegisterValue`, or a `ModbusError` if an error occurs during
    /// the operation.
    async fn dump_input_registers(
        &mut self,
    ) -> Result<HashMap<std::string::String, RegisterValue>, ModbusError> {
        self.dump_registers(&ModBusRegisters::INPUT).await
    }

    /// The function `read_holding_registers_by_name` reads holding registers by name i
    /// 
    /// Arguments:
    /// 
    /// * `names`: list of names of the registers to read.
    /// 
    /// Returns:
    /// 
    /// The function `read_holding_registers_by_name` is returning a `Result` containing a
    /// `HashMap<String, RegisterValue>` on success or a `ModbusError` on failure.
    /// the operation.
    async fn read_holding_registers_by_name(
        &mut self,
        names: &[String],
    ) -> Result<HashMap<String, RegisterValue>, ModbusError> {
        self.read_registers_by_name(names, &ModBusRegisters::HOLDING)
            .await
    }
    /// The function `read_holding_registers` reads holding registers and returns a HashMap
    /// of register values.
    /// 
    /// Arguments:
    /// 
    /// * `regs`: list of the registers to read.
    /// 
    /// Returns:
    /// 
    /// This function `read_holding_registers` is returning a `Result` containing a `HashMap` with keys of
    /// type `String` and values of type `RegisterValue`, or a `ModbusError` if an error occurs during
    /// the operation.
    async fn read_holding_registers(
        &mut self,
        regs: &[Register],
    ) -> Result<HashMap<String, RegisterValue>, ModbusError> {
        self.read_registers(regs, &ModBusRegisters::HOLDING).await
    }

    /// The function `read_holding_register` reads a holding register asynchronously
    /// 
    /// Arguments:
    /// 
    /// * `reg`: list of the registers to read.
    /// 
    /// Returns:
    /// 
    /// The `read_holding_register` function is returning a `Result` enum with either a `RegisterValue`
    /// on success or a `ModbusError` on failure.
    async fn read_holding_register(
        &mut self,
        reg: &Register,
    ) -> Result<RegisterValue, ModbusError> {
        self.read_register(reg, &ModBusRegisters::HOLDING).await
    }

    /// The `dump_holding_registers` function dumps holding registers and returns a
    /// result containing a hashmap of register values or a Modbus error.
    /// 
    /// Returns:
    /// 
    /// This function is returning a `Result` containing a `HashMap` with keys of type `String` and
    /// values of type `RegisterValue`, or a `ModbusError` if an error occurs.
    async fn dump_holding_registers(
        &mut self,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError> {
        self.dump_registers(&ModBusRegisters::HOLDING).await
    }

    /// The function `write_holding_register` writes a value to a holding register in a Modbus device.
    /// 
    /// Arguments:
    /// 
    /// * `reg`:  the register to be write.
    /// * `val`: The `val` parameter represents the value that you want to write to the holding register
    /// 
    /// Returns:
    /// 
    /// The `write_holding_register` function returns a `Result` enum with the success type `()` (an
    /// empty tuple) and the error type `ModbusError`.
    async fn write_holding_register(
        &mut self,
        reg: &Register,
        val: &RegisterValue,
    ) -> Result<(), ModbusError> {
        let data: Vec<u16> = val.clone().try_into()?;

        self.write_raw_holding_registers(&reg.addr, &data).await
    }

    /// The function `write_holding_register` writes a value to a holding register in a Modbus device.
    /// 
    /// Arguments:
    /// 
    /// * `name`:  the name of ther register to be write.
    /// * `val`: The `val` parameter represents the value that you want to write to the holding register
    /// 
    /// Returns:
    /// 
    /// The `write_holding_register` function returns a `Result` enum with the success type `()` (an
    /// empty tuple) and the error type `ModbusError`.
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

    /// This function retrieves a holding register by its name from a data structure and returns it as
    /// an optional value.
    /// 
    /// Arguments:
    /// 
    /// * `name`: The `name` parameter represents the name of
    /// the holding register you want to retrieve.
    /// 
    /// Returns:
    /// 
    /// The function `get_holding_register_by_name` returns an `Option<Register>`.
    fn get_holding_register_by_name(&mut self, name: &str) -> Option<Register> {
        self.holding_registers.get(name).cloned()
    }

    /// This function retrieves a input register by its name from a data structure and returns it as
    /// an optional value.
    /// 
    /// Arguments:
    /// 
    /// * `name`: The `name` parameter represents the name of
    /// the input register you want to retrieve.
    /// 
    /// Returns:
    /// 
    /// The function `get_input_register_by_name` returns an `Option<Register>`
    fn get_input_register_by_name(&mut self, name: &str) -> Option<Register> {
        self.input_registers.get(name).cloned()
    }
}