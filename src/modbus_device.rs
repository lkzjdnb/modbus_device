use std::{collections::HashMap, net::SocketAddr};
use tokio::runtime;
use tokio_modbus::{Address, Quantity};

pub mod errors;
pub mod modbus_device_async;
pub mod register;
pub mod types;
pub mod utils;

use errors::ModbusError;
use modbus_device_async::{ModbusConnexionAsync, ModbusDeviceAsync};
use register::Register;
use types::{ModBusContext, ModBusRegisters, RegisterValue};

pub struct ModbusDevice {
    device: ModbusDeviceAsync,
    runtime: runtime::Runtime,
}

pub trait ModbusConnexion {
    fn connect(&mut self) -> Result<(), std::io::Error>;
    fn read_raw_input_registers(
        &mut self,
        addr: Address,
        nb: Quantity,
    ) -> Result<Vec<u16>, ModbusError>;

    fn read_input_registers_by_name(
        &mut self,
        names: Vec<String>,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError>;
    fn read_input_registers(
        &mut self,
        regs: Vec<Register>,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError>;

    fn read_register(
        &mut self,
        regs: Vec<Register>,
        source: ModBusRegisters,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError>;

    fn dump_input_registers(&mut self) -> Result<HashMap<String, RegisterValue>, ModbusError>;

    fn read_raw_holding_registers(
        &mut self,
        addr: Address,
        nb: Quantity,
    ) -> Result<Vec<u16>, ModbusError>;
    fn read_holding_registers_by_name(
        &mut self,
        names: Vec<String>,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError>;
    fn read_holding_registers(
        &mut self,
        regs: Vec<Register>,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError>;
    fn read_holding_register(&mut self, regs: Register) -> Result<RegisterValue, ModbusError>;

    fn dump_holding_registers(&mut self) -> Result<HashMap<String, RegisterValue>, ModbusError>;

    fn get_holding_register_by_name(&mut self, name: String) -> Option<&Register>;
    fn write_raw_holding_registers(
        &mut self,
        addr: Address,
        data: Vec<u16>,
    ) -> Result<(), ModbusError>;
    fn write_holding_register(
        &mut self,
        reg: Register,
        val: RegisterValue,
    ) -> Result<(), ModbusError>;
}

impl ModbusDevice {
    pub fn new(
        context: ModBusContext,
        input_registers: HashMap<String, Register>,
        holding_registers: HashMap<String, Register>,
    ) -> Self {
        ModbusDevice {
            device: ModbusDeviceAsync::new(context, input_registers, holding_registers),
            runtime: runtime::Runtime::new().unwrap(),
        }
    }
}

impl ModbusConnexion for ModbusDevice {
    // read input registers by address
    fn read_raw_input_registers(
        &mut self,
        addr: Address,
        nb: Quantity,
    ) -> Result<Vec<u16>, ModbusError> {
        self.runtime
            .block_on(self.device.read_raw_input_registers(addr, nb))
    }

    fn read_input_registers_by_name(
        &mut self,
        names: Vec<String>,
    ) -> Result<HashMap<std::string::String, RegisterValue>, ModbusError> {
        self.runtime
            .block_on(self.device.read_input_registers_by_name(names))
    }

    fn read_register(
        &mut self,
        regs: Vec<Register>,
        source: ModBusRegisters,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError> {
        self.runtime
            .block_on(self.device.read_register(regs, source))
    }

    fn read_input_registers(
        &mut self,
        regs: Vec<Register>,
    ) -> Result<HashMap<std::string::String, RegisterValue>, ModbusError> {
        self.runtime
            .block_on(self.device.read_input_registers(regs))
    }

    fn dump_input_registers(
        &mut self,
    ) -> Result<HashMap<std::string::String, RegisterValue>, ModbusError> {
        self.runtime.block_on(self.device.dump_input_registers())
    }

    fn read_raw_holding_registers(
        &mut self,
        addr: Address,
        nb: Quantity,
    ) -> Result<Vec<u16>, ModbusError> {
        self.runtime
            .block_on(self.device.read_raw_holding_registers(addr, nb))
    }

    fn read_holding_registers_by_name(
        &mut self,
        names: Vec<String>,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError> {
        self.runtime
            .block_on(self.device.read_holding_registers_by_name(names))
    }
    fn read_holding_registers(
        &mut self,
        regs: Vec<Register>,
    ) -> Result<HashMap<String, RegisterValue>, ModbusError> {
        self.runtime
            .block_on(self.device.read_holding_registers(regs))
    }
    fn read_holding_register(&mut self, reg: Register) -> Result<RegisterValue, ModbusError> {
        self.runtime
            .block_on(self.device.read_holding_register(reg))
    }

    fn dump_holding_registers(&mut self) -> Result<HashMap<String, RegisterValue>, ModbusError> {
        self.runtime.block_on(self.device.dump_holding_registers())
    }

    fn get_holding_register_by_name(&mut self, name: String) -> Option<&Register> {
        self.device.get_holding_register_by_name(name)
    }

    fn write_raw_holding_registers(
        &mut self,
        addr: Address,
        data: Vec<u16>,
    ) -> Result<(), ModbusError> {
        self.runtime
            .block_on(self.device.write_raw_holding_registers(addr, data))
    }

    fn write_holding_register(
        &mut self,
        reg: Register,
        val: RegisterValue,
    ) -> Result<(), ModbusError> {
        self.runtime
            .block_on(self.device.write_holding_register(reg, val))
    }

    fn connect(&mut self) -> Result<(), std::io::Error> {
        self.runtime.block_on(self.device.connect())
    }
}
