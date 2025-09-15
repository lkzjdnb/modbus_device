use std::fmt::Debug;

#[derive(Debug, Copy, Clone)]

pub enum DataType {
    UInt16,
    UInt32,
    UInt64,
    UInt128,
    Int32,
    Enum16,
    Sized,
    Float32,
    Boolean,
}

#[derive(Clone)]
/// The `Register` struct in Rust represents the structure of a register with a name, address, length, data type, and
/// read flag.
/// 
/// Properties:
/// 
/// * `name`: the name of the register.
/// 
/// * `addr`: the address of the register, typically a 16-bit unsigned integer (u16) indicating the memory location where the register is
/// located.
/// 
/// * `len`: the length of the register data in
/// 16-bit units. This means that if `len` is 2, for example, the register data will be 32 bits long (2 x 16 bits).
/// 
/// * `data_type`: the type of data that the register holds.
/// 
/// * `read`:  boolean value that indicates whether the register is readable only or not
pub struct Register {
    pub name: String,
    pub addr: u16,
    pub len: u16, // in 16bits
    pub data_type: DataType,
    pub read: bool,
}

impl Debug for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Registre")
            .field("name", &self.name)
            .field("addr", &self.addr)
            .field("len", &self.len)
            .field("data_type", &self.data_type)
            .field("read", &self.read)
            .finish()
    }
}
