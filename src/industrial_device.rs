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
impl IndustrialDevice for ModbusDeviceAsync {
    async fn connect(&mut self) -> Result<(), IndustrialDeviceError> {
        ModbusConnexionAsync::connect(self).await?;
        Ok(())
    }

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

    async fn read_register_by_name(&mut self, name: &str) -> Result<Value, IndustrialDeviceError> {
        let (reg, table) = get_register_by_name(self, name)?;
        let val = self.read_register(&reg, &table).await?;
        Ok(val.into())
    }

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
