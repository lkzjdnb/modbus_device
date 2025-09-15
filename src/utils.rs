use std::{collections::HashMap, fs::File};

use serde::{Deserialize, Serialize};

use crate::{register::Register, types::DataType};

fn return_true() -> bool {
    true
}

#[derive(Serialize, Deserialize)]
/// The `RawRegister` struct in Rust represents a register from the json file with specific properties such as ID, name,
/// data type, length, and read permission.
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
/// The struct `RegistersFormat` contains metadata, result, and a vector of raw registers for the extraction of the file json .
/// 
/// Properties:
/// 
/// * It represents the identifier or metadata associated with the registers file.
/// * `result`: The `result` property in the `RegistersFormat` struct likely represents the outcome or
/// status of the opening of the file.
/// 
/// * `registers` : the list of register on the file
struct RegistersFormat {
    metaid: String,
    result: String,
    registers: Vec<RawRegister>,
}

/// The function `get_defs_from_json` reads JSON data from a file and converts it into a HashMap of
/// Register objects in Rust.
/// 
/// Arguments:
/// 
/// * `input`:  the file that is being read to extract register definitions in JSON format.
/// 
/// Returns:
/// 
/// The function `get_defs_from_json` returns a `Result` containing a `HashMap` with keys of type
/// `String` and values of type `Register`, or an error of type `serde_json::Error`.
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