use std::collections::HashMap;

use tokio_modbus::{Address, Quantity};

use crate::{
    errors::ModbusError,
    register::Register,
    types::{ModBusRegisters, RegisterValue},
};

#[trait_variant::make(ModbusConnexionAsync: Send)]
pub trait LocalModbusConnexionAsync {
    async fn connect(&mut self) -> Result<(), std::io::Error>;
    async fn read_raw_input_registers(
        &mut self,
        addr: Address,
        nb: Quantity,
    ) -> Result<Vec<u16>, ModbusError>;

    async fn read_input_registers_by_name(
        &mut self,
        names: Vec<String>,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError>;
    async fn read_input_registers(
        &mut self,
        regs: Vec<Register>,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError>;

    async fn read_range(
        &mut self,
        regs: Vec<Register>,
        source: ModBusRegisters,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError>;
    async fn read_register(
        &mut self,
        regs: Vec<Register>,
        source: ModBusRegisters,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError>;

    async fn dump_input_registers(&mut self)
        -> Result<HashMap<String, RegisterValue>, ModbusError>;

    async fn read_raw_holding_registers(
        &mut self,
        addr: Address,
        nb: Quantity,
    ) -> Result<Vec<u16>, ModbusError>;
    async fn read_holding_registers_by_name(
        &mut self,
        names: Vec<String>,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError>;
    async fn read_holding_registers(
        &mut self,
        regs: Vec<Register>,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError>;
    async fn read_holding_register(&mut self, regs: Register)
        -> Result<RegisterValue, ModbusError>;

    async fn dump_holding_registers(
        &mut self,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError>;

    fn get_holding_register_by_name(&mut self, name: String) -> Option<&Register>;
    async fn write_raw_holding_registers(
        &mut self,
        addr: Address,
        data: Vec<u16>,
    ) -> Result<(), ModbusError>;
    async fn write_holding_register(
        &mut self,
        reg: Register,
        val: RegisterValue,
    ) -> Result<(), ModbusError>;
}
