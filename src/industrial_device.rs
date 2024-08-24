use std::collections::HashMap;

use crate::modbus_connexion_async::ModbusConnexionAsync;
use crate::modbus_device_async::ModbusDeviceAsync;
use crate::types::RegisterValue;

use industrial_device::types::Value;
use industrial_device::IndustrialDevice;

impl IndustrialDevice for ModbusDeviceAsync {
    async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error + Send>> {
        match ModbusConnexionAsync::connect(self).await {
            Ok(_) => Ok(()),
            Err(err) => Err(Box::new(err)),
        }
    }

    async fn dump_registers(
        &mut self,
    ) -> Result<HashMap<String, Value>, Box<dyn std::error::Error + Send>> {
        todo!()
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
