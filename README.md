# Modbus device
Rust library to provide high level access to a device over the ModBus protocol.
Device definition can be provided directly as a HashMap<String, Register> or through a JSON file (using the get_defs_from_json helper function). Using this definition all conversions from name->address and values->register ([u16]) is handled transparently.

Ex : 
```rust
let addr: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 502);

let input_registers_json = File::open("input_registers.json").unwrap();
let input_registers = utils::get_defs_from_json(input_registers_json).unwrap();

let holding_registers_json = File::open("holding_registers.json").unwrap();
let holding_registers = utils::get_defs_from_json(holding_registers_json).unwrap();

let device = ModbusDeviceAsync::new(
    TCPContext { addr }.into(),
    input_registers,
    holding_registers,
);

device.connect().await.unwrap();

device
    .read_input_registers_by_name(&["ProjectId".to_string()])
    .await
    .unwrap();
```
