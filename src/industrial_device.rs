use std::collections::HashMap;

use crate::modbus_connexion_async::ModbusConnexionAsync;
use crate::modbus_device_async::ModbusDeviceAsync;

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
