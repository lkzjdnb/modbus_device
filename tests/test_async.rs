use core::panic;
use modbus_device::modbus_connexion_async::ModbusConnexionAsync;
use modbus_device::types::TCPContext;
use modbus_device::{utils, ModbusDeviceAsync};
use std::fs::File;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::thread;
use std::time::Duration;
use testcontainers::runners::AsyncRunner;
use testcontainers::{core::WaitFor, ContainerAsync, GenericImage};
use tokio;

fn create_server() -> GenericImage {
    let server = GenericImage::new("modbus-test-server", "1")
        .with_exposed_port(4502.into())
        .with_wait_for(WaitFor::message_on_stdout("Started on port 4502"));

    server
}

async fn start_server(container: GenericImage) -> ContainerAsync<GenericImage> {
    let server = container.start().await;
    match server {
        Ok(val) => Ok(val),
        Err(err) => match &err {
            testcontainers::TestcontainersError::Client(client_err) => match client_err {
                testcontainers::core::error::ClientError::PullImage { descriptor: _, err: _pull_err } => panic!("Could not access docker image. Tests need the test server docker image. See https://github.com/lkzjdnb/modbus_device/blob/master/Testing.md \n {err}"),
                _ => Err(err)
            },
            _ => Err(err),
        },
    }
    .unwrap()
}

async fn connect(server: &ContainerAsync<GenericImage>) -> ModbusDeviceAsync {
    let port = server.get_host_port_ipv4(4502_u16).await.unwrap();

    let addr: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);

    let input_registers_json = File::open("tests/input_registers.json").unwrap();
    let input_registers = utils::get_defs_from_json(input_registers_json).unwrap();

    let holding_registers_json = File::open("tests/holding_registers.json").unwrap();
    let holding_registers = utils::get_defs_from_json(holding_registers_json).unwrap();

    let device = ModbusDeviceAsync::new(
        TCPContext { addr }.into(),
        input_registers,
        holding_registers,
    );
    device
}

#[tokio::test]
async fn test_read() {
    let container = create_server();
    let server = start_server(container).await;
    let mut device = connect(&server).await;

    thread::sleep(Duration::from_secs(15));
    device.connect().await.unwrap();

    device
        .read_input_registers_by_name(&["ProjectId".to_string()])
        .await
        .unwrap();
    device
        .read_holding_registers_by_name(&["Version".to_string()])
        .await
        .unwrap();
}

#[tokio::test]
async fn test_write() {
    let container = create_server();
    let server = start_server(container).await;
    let mut device = connect(&server).await;

    thread::sleep(Duration::from_secs(15));
    device.connect().await.unwrap();

    let reg = device
        .get_holding_register_by_name("ProductionRate[%]")
        .unwrap()
        .clone();
    device
        .write_holding_register(&reg, &(0.52 as f32).into())
        .await
        .unwrap();
}
