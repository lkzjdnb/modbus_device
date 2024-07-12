use std::array::TryFromSliceError;
use tokio_modbus::Exception;

#[derive(Debug)]
pub struct ConversionError;
#[derive(Debug)]
pub struct DeviceNotConnectedError;

#[derive(Debug)]
pub enum ModbusError {
    Exception(Exception),
    IOerror(std::io::Error),
    ModbusError(tokio_modbus::Error),
    TryFromSliceError(TryFromSliceError),
    ConversionError(ConversionError),
    DeviceNotConnectedError(DeviceNotConnectedError),
}
impl From<Exception> for ModbusError {
    fn from(value: Exception) -> Self {
        ModbusError::Exception(value)
    }
}
impl From<std::io::Error> for ModbusError {
    fn from(value: std::io::Error) -> Self {
        ModbusError::IOerror(value)
    }
}
impl From<tokio_modbus::Error> for ModbusError {
    fn from(value: tokio_modbus::Error) -> Self {
        ModbusError::ModbusError(value)
    }
}
impl From<TryFromSliceError> for ModbusError {
    fn from(value: TryFromSliceError) -> Self {
        ModbusError::TryFromSliceError(value)
    }
}
impl From<ConversionError> for ModbusError {
    fn from(value: ConversionError) -> Self {
        ModbusError::ConversionError(value)
    }
}

impl From<DeviceNotConnectedError> for ModbusError {
    fn from(value: DeviceNotConnectedError) -> Self {
        ModbusError::DeviceNotConnectedError(value)
    }
}
