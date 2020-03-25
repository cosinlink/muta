use bytes::{Bytes, BytesMut};
use fixed_codec_derive::FixedCodec;
use rand::random;

const HASH_LEN: usize = 32;

type JsonString = String;

#[derive(Clone, Debug, FixedCodec)]
struct Hash([u8; HASH_LEN]);

impl Hash {
    fn new() -> Self {
        let bytes = (0..1024).map(|_| random::<u8>()).collect::<Vec<_>>();
        let mut out = [0u8; HASH_LEN];
        out.copy_from_slice(&bytes);
        Hash(out)
    }
}

#[derive(Clone, Debug, FixedCodec)]
pub struct SignedTransaction {
    pub raw:       JsonString,
    pub tx_hash:   Hash,
    pub pubkey:    Bytes,
    pub signature: Bytes,
}

#[test]
fn test_hash() {
    let data = Hash::new();
    assert_eq!(
        Hash::decode_fixed(data.encode_fixed().unwrap()).unwrap(),
        data
    );
}
