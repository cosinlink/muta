use bytes::{Bytes, BytesMut};
use fixed_codec_derive::MutaFixedCodec;
use rand::random;

use protocol::fixed_codec::FixedCodec;

const HASH_LEN: usize = 32;

type JsonString = String;

#[derive(Clone, Debug, MutaFixedCodec, PartialEq, Eq, Copy)]
pub struct Hash([u8; HASH_LEN]);

impl Hash {
    fn new() -> Self {
        let bytes = (0..32).map(|_| random::<u8>()).collect::<Vec<_>>();
        let mut out = [0u8; HASH_LEN];
        out.copy_from_slice(&bytes);
        Hash(out)
    }
}

#[derive(Clone, Debug, MutaFixedCodec, PartialEq, Eq)]
pub struct Hex(String);

impl Hex {
    fn new() -> Self {
        let temp = "0x".to_owned() + &String::from("muta-dev");
        Self(temp)
    }
}

#[derive(Clone, Debug, MutaFixedCodec, PartialEq, Eq)]
pub struct SignedTransaction {
    pub raw:       JsonString,
    pub tx_hash:   Hash,
    pub pubkey:    Bytes,
    pub signature: Bytes,
}

impl SignedTransaction {
    fn new() -> Self {
        SignedTransaction {
            raw:       JsonString::from("muta-dev"),
            tx_hash:   Hash::new(),
            pubkey:    random_bytes(32),
            signature: random_bytes(64),
        }
    }
}

fn random_bytes(len: usize) -> Bytes {
    let temp = (0..len).map(|_| random::<u8>()).collect::<Vec<_>>();
    Bytes::from(temp)
}

#[test]
fn test_hash() {
    let data = Hash::new();
    assert_eq!(
        data,
        FixedCodec::decode_fixed(data.encode_fixed().unwrap()).unwrap()
    );

    let data = SignedTransaction::new();
    assert_eq!(
        data,
        FixedCodec::decode_fixed(data.encode_fixed().unwrap()).unwrap()
    );

    let data = Hex::new();
    assert_eq!(
        data,
        FixedCodec::decode_fixed(data.encode_fixed().unwrap()).unwrap()
    );
}
