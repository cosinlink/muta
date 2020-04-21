use bytes::Bytes;

use crate::types::primitive::{Address, Hash, JsonString};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawTransaction {
    pub chain_id:     Hash,
    pub nonce:        Hash,
    pub timeout:      u64,
    pub cycles_price: u64,
    pub cycles_limit: u64,
    pub request:      TransactionRequest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransactionRequest {
    pub service_name: String,
    pub method:       String,
    pub payload:      JsonString,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignedTransaction {
    pub raw:            RawTransaction,
    pub tx_hash:        Hash,
    pub pubkey:         Bytes,
    pub signature:      Bytes,
    pub sender:         Address,
    pub signature_type: u8,
}
