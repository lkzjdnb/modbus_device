use std::collections::HashMap;

use crate::errors::ModbusError;
use crate::modbus_connexion_async::ModbusConnexionAsync;
use crate::register::Register;
use crate::types::{ModBusRegisters, RegisterValue};
use crate::ModbusDeviceAsync;

use industrial_device::errors::IndustrialDeviceError;
use industrial_device::types::Value;
use industrial_device::IndustrialDevice;
use log::warn;

use async_trait::async_trait;

/// The function `get_register_by_name` in Rust retrieves a register by name from a Modbus device,
/// handling different cases based on the register type.
/// 
/// Arguments:
/// 
/// * `dev`: The `dev` parameter is a mutable reference to a `ModbusDeviceAsync` object, which is used
/// to interact with a Modbus device asynchronously.
/// * `name`: The `name` parameter is a string that represents
/// the name of the register you are looking for in the Modbus device.
/// 
/// Returns:
/// 
/// The function `get_register_by_name` returns a `Result` containing a tuple with a `Register` and the type of register
/// `ModBusRegisters` enum, or an `IndustrialDeviceError` in case of a register not being found.
fn get_register_by_name(
    dev: &mut ModbusDeviceAsync,
    name: &str,
) -> Result<(Register, ModBusRegisters), IndustrialDeviceError> {
    let input_reg = dev.get_input_register_by_name(name);
    let holding_reg = dev.get_holding_register_by_name(name);

    match (input_reg, holding_reg) {
        (None, None) => Err(IndustrialDeviceError::RegisterNotFoundError {
            name: name.to_string(),
        }),
        (None, Some(val)) => Ok((val, ModBusRegisters::HOLDING)),
        (Some(val), None) => Ok((val, ModBusRegisters::INPUT)),
        (Some(val), Some(_)) => {
            warn!("Found an input register and holding register with the same name, returning the input register ({name})");
            Ok((val, ModBusRegisters::INPUT))
        }
    }
}

#[async_trait]
/// This code snippet is implementing the `IndustrialDevice` trait for the `ModbusDeviceAsync` struct.
/// By implementing this trait, the `ModbusDeviceAsync` struct gains the functionality defined by the
/// trait methods.
impl IndustrialDevice for ModbusDeviceAsync {
    /// The `connect` function in Rust establishes a Modbus connection asynchronously
    /// 
    /// Returns:
    /// 
    /// a `Result` with either `Ok(())` if the connection is
    /// successful or an `IndustrialDeviceError` if there is an error during the connection process.
    async fn connect(&mut self) -> Result<(), IndustrialDeviceError> {
        ModbusConnexionAsync::connect(self).await?;
        Ok(())
    }

    /// The `dump_registers` function in Rust asynchronously retrieves input and holding registers,
    /// converts them into a HashMap of values, and returns the combined result.
    /// 
    /// Returns:
    /// 
    /// `Result` containing a `HashMap<String, Value>` if
    /// successful, or an `IndustrialDeviceError` if an error occurs during the process.
    async fn dump_registers(&mut self) -> Result<HashMap<String, Value>, IndustrialDeviceError> {
        let input: HashMap<String, RegisterValue> = self.dump_input_registers().await?;
        let holding: HashMap<String, RegisterValue> = self.dump_holding_registers().await?;

        let input_conv: HashMap<String, Value> = input
            .iter()
            .map(|(name, val)| (name.clone(), Into::<Value>::into(*val)))
            .collect();
        let holding_conv: HashMap<String, Value> = holding
            .iter()
            .map(|(name, val)| (name.clone(), Into::<Value>::into(*val)))
            .collect();

        let mut res = input_conv;
        res.extend(holding_conv);
        Ok(res)
    }

    /// The function `read_register_by_name` reads a register value by its name asynchronously in Rust.
    /// 
    /// Arguments:
    /// 
    /// * `name`:  reference to a string which represents the name of the register you want to read.
    /// 
    /// Returns:
    /// 
    /// `Result` containing a `Value` or an IndustrialDeviceError`.
    async fn read_register_by_name(&mut self, name: &str) -> Result<Value, IndustrialDeviceError> {
        let (reg, table) = get_register_by_name(self, name)?;
        let val = self.read_register(&reg, &table).await?;
        Ok(val.into())
    }

    /// The function `write_register_by_name` writes a value to a register by name asynchronously in
    /// Rust.
    /// 
    /// Arguments:
    /// 
    /// * `name`: The `name` parameter a reference to a string that represents the name of the register to write to.
    /// * `value`: The `value` a reference to a `Value` type corresponding the value we want to write in the register.
    /// 
    /// Returns:
    /// 
    ///  a `Result<(), IndustrialDeviceError>`.
    async fn write_register_by_name(
        &mut self,
        name: &str,
        value: &Value,
    ) -> Result<(), IndustrialDeviceError> {
        let val: RegisterValue = value.clone().into();
        self.write_holding_register_by_name(name, &val).await?;
        Ok(())
    }
}

impl From<ModbusError> for IndustrialDeviceError {
    fn from(value: ModbusError) -> Self {
        match value {
            ModbusError::Exception { err } => {
                IndustrialDeviceError::RequestError { err: Box::new(err) }
            }
            ModbusError::ModbusError { err } => match err {
                tokio_modbus::Error::Transport(err) => match err.kind() {
                    std::io::ErrorKind::BrokenPipe => {
                        IndustrialDeviceError::DeviceNotAccessibleError { err: Box::new(err) }
                    }
                    _ => IndustrialDeviceError::RequestError { err: Box::new(err) },
                },
                _ => IndustrialDeviceError::RequestError { err: Box::new(err) },
            },
            ModbusError::IOerror { err } => {
                IndustrialDeviceError::DeviceNotAccessibleError { err: Box::new(err) }
            }
            ModbusError::TryFromSliceError { err } => {
                IndustrialDeviceError::ConversionError { err: Box::new(err) }
            }
            ModbusError::ConversionError => IndustrialDeviceError::ConversionError {
                err: Box::new(value),
            },
            ModbusError::DeviceNotConnectedError => {
                IndustrialDeviceError::DeviceNotConnectedError {
                    err: Box::new(value),
                }
            }
            ModbusError::RegisterDoesNotExistError { name } => {
                IndustrialDeviceError::RegisterNotFoundError { name }
            }
        }
    }
}

impl From<RegisterValue> for Value {
    fn from(value: RegisterValue) -> Self {
        match value {
            RegisterValue::U16(val) => Value::U16(val),
            RegisterValue::U32(val) => Value::U32(val),
            RegisterValue::U64(val) => Value::U64(val),
            RegisterValue::U128(val) => Value::U128(val),
            RegisterValue::S32(val) => Value::S32(val),
            RegisterValue::Enum16(val) => Value::Enum16(val),
            RegisterValue::Sized(val) => Value::Sized(val),
            RegisterValue::Float32(val) => Value::Float32(val),
            RegisterValue::Boolean(val) => Value::Boolean(val),
        }
    }
}

impl From<Value> for RegisterValue {
    fn from(value: Value) -> Self {
        match value {
            Value::U16(val) => RegisterValue::U16(val),
            Value::U32(val) => RegisterValue::U32(val),
            Value::U64(val) => RegisterValue::U64(val),
            Value::U128(val) => RegisterValue::U128(val),
            Value::S16(val) => RegisterValue::S32(val as i32),
            Value::S32(val) => RegisterValue::S32(val),
            Value::Enum16(val) => RegisterValue::Enum16(val),
            Value::Sized(val) => RegisterValue::Sized(val),
            Value::Float32(val) => RegisterValue::Float32(val),
            Value::Boolean(val) => RegisterValue::Boolean(val),
        }
    }
}
