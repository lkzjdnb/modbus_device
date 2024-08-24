use std::collections::HashMap;

use crate::errors::ModbusError;
use crate::modbus_connexion_async::ModbusConnexionAsync;
use crate::modbus_device_async::ModbusDeviceAsync;
use crate::types::RegisterValue;

use industrial_device::errors::IndustrialDeviceError;
use industrial_device::types::Value;
use industrial_device::IndustrialDevice;

impl IndustrialDevice for ModbusDeviceAsync {
    async fn connect(&mut self) -> Result<(), IndustrialDeviceError> {
        ModbusConnexionAsync::connect(self).await?;
        Ok(())
    }

    async fn dump_registers(&mut self) -> Result<HashMap<String, Value>, IndustrialDeviceError> {
        todo!()
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
