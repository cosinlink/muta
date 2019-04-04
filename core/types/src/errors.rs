use std::error;
use std::fmt;

use numext_fixed_hash::FixedHashError;

use core_serialization::CodecError;

#[derive(Debug)]
pub enum TypesError {
    ParseHexError(FixedHashError),
    Serialization(CodecError),
    AddressLenInvalid,
    HashLenInvalid,
}

impl error::Error for TypesError {}
impl fmt::Display for TypesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            TypesError::ParseHexError(ref err) => format!("parse hex error: {:?}", err),
            TypesError::Serialization(ref err) => format!("serialization error: {:?}", err),
            TypesError::AddressLenInvalid => "address len invalid".to_owned(),
            TypesError::HashLenInvalid => "hash len invalid".to_owned(),
        };
        write!(f, "{}", printable)
    }
}

impl From<FixedHashError> for TypesError {
    fn from(err: FixedHashError) -> Self {
        TypesError::ParseHexError(err)
    }
}

impl From<CodecError> for TypesError {
    fn from(err: CodecError) -> Self {
        TypesError::Serialization(err)
    }
}