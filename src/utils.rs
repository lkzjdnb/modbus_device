use std::{collections::HashMap, fs::File};

use serde::{Deserialize, Serialize};

use crate::{register::Register, types::DataType};

fn return_true() -> bool {
    true
}

#[derive(Serialize, Deserialize)]
struct RawRegister {
    id: u16,
    name: String,
    #[serde(rename = "type")]
    type_: DataType,
    len: u16,
    #[serde(default = "return_true")]
    read: bool,
}

#[derive(Serialize, Deserialize)]
struct RegistersFormat {
    metaid: String,
    result: String,
    registers: Vec<RawRegister>,
}

pub fn get_defs_from_json(input: File) -> Result<HashMap<String, Register>, serde_json::Error> {
    let raw: RegistersFormat = serde_json::from_reader(input)?;
    let mut m = HashMap::<String, Register>::new();
    for f in raw.registers {
        m.insert(
            f.name.clone(),
            Register {
                name: f.name,
                addr: f.id,
                len: f.len / 16,
                data_type: f.type_.into(),
                read: f.read,
            },
        );
    }
    return Ok(m);
}
