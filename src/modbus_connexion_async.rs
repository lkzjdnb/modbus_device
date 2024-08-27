use std::collections::HashMap;

use tokio_modbus::{Address, Quantity};

use crate::{
    errors::ModbusError,
    register::Register,
    types::{ModBusRegisters, RegisterValue},
};

#[trait_variant::make(ModbusConnexionAsync: Send)]
pub trait LocalModbusConnexionAsync {
    async fn connect(&mut self) -> Result<(), ModbusError>;

    // Lower level utils
    async fn read_raw_registers(
        &mut self,
        addr: &Address,
        nb: &Quantity,
        source: &ModBusRegisters,
    ) -> Result<Vec<u16>, ModbusError>;
    async fn write_raw_holding_registers(
        &mut self,
        addr: &Address,
        data: &[u16],
    ) -> Result<(), ModbusError>;
    async fn read_range(
        &mut self,
        regs: &[Register],
        source: &ModBusRegisters,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError>;

    // Global access
    async fn read_registers(
        &mut self,
        regs: &[Register],
        source: &ModBusRegisters,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError>;
    async fn read_register(
        &mut self,
        reg: &Register,
        source: &ModBusRegisters,
    ) -> Result<RegisterValue, ModbusError>;
    async fn read_registers_by_name(
        &mut self,
        names: &[String],
        source: &ModBusRegisters,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError>;
    async fn dump_registers(
        &mut self,
        source: &ModBusRegisters,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError>;

    // Input register specific wrappers
    async fn read_input_registers_by_name(
        &mut self,
        names: &[String],
    ) -> Result<HashMap<String, RegisterValue>, ModbusError>;
    async fn read_input_registers(
        &mut self,
        regs: &[Register],
    ) -> Result<HashMap<String, RegisterValue>, ModbusError>;
    async fn dump_input_registers(&mut self)
        -> Result<HashMap<String, RegisterValue>, ModbusError>;

    // Holding register specific wrappers
    async fn read_holding_registers_by_name(
        &mut self,
        names: &[String],
    ) -> Result<HashMap<String, RegisterValue>, ModbusError>;
    async fn read_holding_registers(
        &mut self,
        regs: &[Register],
    ) -> Result<HashMap<String, RegisterValue>, ModbusError>;
    async fn read_holding_register(&mut self, reg: &Register)
        -> Result<RegisterValue, ModbusError>;
    async fn dump_holding_registers(
        &mut self,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError>;
    async fn write_holding_register(
        &mut self,
        reg: &Register,
        val: &RegisterValue,
    ) -> Result<(), ModbusError>;
    async fn write_holding_register_by_name(
        &mut self,
        name: &str,
        val: &RegisterValue,
    ) -> Result<(), ModbusError>;

    // Registers access utils
    fn get_holding_register_by_name(&mut self, name: &str) -> Option<Register>;
    fn get_input_register_by_name(&mut self, name: &str) -> Option<Register>;
}
