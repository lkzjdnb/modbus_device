use std::array::TryFromSliceError;

use crate::register;
use serde::{Deserialize, Serialize};

pub enum ModBusRegisters {
    INPUT,
    HOLDING,
}

#[derive(Debug, Clone, Copy)]
pub enum RegisterValue {
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    S32(i32),
    Enum16(u16),
    Sized([u8; 66]),
    Float32(f32),
    Boolean(bool),
}

#[derive(Serialize, Deserialize)]
pub enum DataType {
    #[serde(alias = "Uint16")]
    UInt16,
    #[serde(alias = "Uint32")]
    UInt32,
    UInt64,
    UInt128,
    Int32,
    Enum16,
    #[serde(rename = "Sized+Uint16[31]")]
    Sized,
    #[serde(rename = "IEEE-754 float32")]
    Float32,
    #[serde(rename = "boolean")]
    Boolean,
}

impl Into<register::DataType> for DataType {
    fn into(self) -> register::DataType {
        match self {
            Self::UInt16 => register::DataType::UInt16,
            Self::UInt32 => register::DataType::UInt32,
            Self::UInt64 => register::DataType::UInt64,
            Self::UInt128 => register::DataType::UInt128,
            Self::Int32 => register::DataType::Int32,
            Self::Enum16 => register::DataType::Enum16,
            Self::Sized => register::DataType::Sized,
            Self::Float32 => register::DataType::Float32,
            Self::Boolean => register::DataType::Boolean,
        }
    }
}

impl From<f32> for RegisterValue {
    fn from(value: f32) -> Self {
        RegisterValue::Float32(value)
    }
}

impl TryFrom<(Vec<u16>, register::DataType)> for RegisterValue {
    fn try_from((raw, kind): (Vec<u16>, register::DataType)) -> Result<Self, Self::Error> {
        let raw_b: Vec<u8> = raw
            .iter()
            .map(|v| v.to_be_bytes())
            .flatten()
            .rev()
            .collect();
        match kind {
            register::DataType::UInt16 => Ok(RegisterValue::U16(raw[0])),
            register::DataType::UInt32 => match raw_b.try_into() {
                Ok(res) => Ok(RegisterValue::U32(u32::from_le_bytes(res))),
                Err(err) => Err(err),
            },
            register::DataType::UInt64 => match raw_b.try_into() {
                Ok(res) => Ok(RegisterValue::U64(u64::from_le_bytes(res))),
                Err(err) => Err(err),
            },
            register::DataType::UInt128 => match raw_b.try_into() {
                Ok(res) => Ok(RegisterValue::U128(u128::from_le_bytes(res))),
                Err(err) => Err(err),
            },
            register::DataType::Int32 => match raw_b.try_into() {
                Ok(res) => Ok(RegisterValue::S32(i32::from_le_bytes(res))),
                Err(err) => Err(err),
            },
            register::DataType::Enum16 => Ok(RegisterValue::Enum16(raw[0])),
            register::DataType::Sized => match raw_b.try_into() {
                Ok(res) => Ok(RegisterValue::Sized(res)),
                Err(err) => Err(err),
            },
            register::DataType::Float32 => match raw_b.try_into() {
                Ok(res) => Ok(RegisterValue::Float32(f32::from_le_bytes(res))),
                Err(err) => Err(err),
            },
            register::DataType::Boolean => Ok(RegisterValue::Boolean(!raw[0] == 0)),
        }
    }

    type Error = Vec<u8>;
}

impl TryInto<Vec<u16>> for RegisterValue {
    type Error = TryFromSliceError;

    fn try_into(self) -> Result<Vec<u16>, Self::Error> {
        let bytearray = match self {
            RegisterValue::U16(val) => val.to_le_bytes().to_vec(),
            RegisterValue::U32(val) => val.to_le_bytes().to_vec(),
            RegisterValue::U64(val) => val.to_le_bytes().to_vec(),
            RegisterValue::U128(val) => val.to_le_bytes().to_vec(),
            RegisterValue::S32(val) => val.to_le_bytes().to_vec(),
            RegisterValue::Enum16(val) => val.to_le_bytes().to_vec(),
            RegisterValue::Sized(val) => val.to_vec(),
            RegisterValue::Float32(val) => val.to_le_bytes().to_vec(),
            RegisterValue::Boolean(val) => match val {
                true => 1 as u16,
                false => 0,
            }
            .to_le_bytes()
            .to_vec(),
        };

        bytearray
            .chunks(2)
            .map(|v| match v.try_into() {
                Ok(arr) => Ok(u16::from_le_bytes(arr)),
                Err(err) => Err(err),
            })
            .rev()
            .collect()
    }
}
