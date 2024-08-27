use custom_error::custom_error;

use std::array::TryFromSliceError;
use tokio_modbus::Exception;

custom_error! {pub ModbusError
    Exception{ err: Exception} = "Modbus exception : {err}",
    IOerror {err: std::io::Error} = "IOError : {err}",
    ModbusError {err: tokio_modbus::Error} = "Modbus error : {err}",
    TryFromSliceError {err: TryFromSliceError} = "Try from slice error : {err}",
    ConversionError = "Conversion error",
    DeviceNotConnectedError = "Device is not connected",
    RegisterDoesNotExistError{ name: String } = "Register {name} was not found",
}

impl From<Exception> for ModbusError {
    fn from(value: Exception) -> Self {
        ModbusError::Exception { err: value }
    }
}
impl From<std::io::Error> for ModbusError {
    fn from(value: std::io::Error) -> Self {
        ModbusError::IOerror { err: value }
    }
}
impl From<tokio_modbus::Error> for ModbusError {
    fn from(value: tokio_modbus::Error) -> Self {
        ModbusError::ModbusError { err: value }
    }
}
impl From<TryFromSliceError> for ModbusError {
    fn from(value: TryFromSliceError) -> Self {
        ModbusError::TryFromSliceError { err: value }
    }
}
