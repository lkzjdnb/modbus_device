use modbus_device::types::TCPContext;
use modbus_device::utils;
use modbus_device::ModbusConnexion;
use modbus_device::ModbusDevice;
use std::fs::File;

fn connect() -> ModbusDevice {
    let input_registers_json = File::open("tests/input_registers.json").unwrap();
    let input_registers = utils::get_defs_from_json(input_registers_json).unwrap();
    let mut device = ModbusDevice::new(
        TCPContext {
            addr: "127.0.0.1:4502".parse().unwrap(),
        }
        .into(),
        input_registers.clone(),
        input_registers,
    );
    device.connect().unwrap();
    device
}

#[test]
fn test_read() {
    let mut device = connect();
    device
        .read_input_registers_by_name(vec!["reg3".to_string()])
        .unwrap();
    device
        .read_holding_registers_by_name(vec!["reg3".to_string()])
        .unwrap();
}

#[test]
fn test_write() {
    let mut device = connect();
    let reg = device
        .get_holding_register_by_name("reg7".to_string())
        .unwrap()
        .clone();
    device
        .write_holding_register(reg.clone(), (0.52 as f32).into())
        .unwrap();
}
