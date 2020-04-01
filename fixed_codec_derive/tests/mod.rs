use bytes::BytesMut;
use fixed_codec_derive::MutaFixedCodec;
use rand::random;

use protocol::fixed_codec::FixedCodec;

const HASH_LEN: usize = 32;

type JsonString = String;

#[derive(Clone, Debug, MutaFixedCodec, PartialEq, Eq, Copy)]
struct Hash([u8; HASH_LEN]);

impl Hash {
    fn new() -> Self {
        let bytes = (0..32).map(|_| random::<u8>()).collect::<Vec<_>>();
        let mut out = [0u8; HASH_LEN];
        out.copy_from_slice(&bytes);
        Hash(out)
    }
}

// #[derive(Clone, Debug, FixedCodec)]
// pub struct SignedTransaction {
//     pub raw:       JsonString,
//     pub tx_hash:   Hash,
//     pub pubkey:    Bytes,
//     pub signature: Bytes,
// }

#[derive(Clone, Debug, MutaFixedCodec, PartialEq, Eq)]
pub struct Foo {
    pub a: u64,
    pub b: u64,
}

#[test]
fn test_hash() {
    let data = Hash::new();
    // let data = Foo { a: 1, b: 2 };
    assert_eq!(
        data,
        FixedCodec::decode_fixed(data.encode_fixed().unwrap()).unwrap()
    );

    // assert_eq!(
    //     Foo::decode_fixed(data.encode_fixed().unwrap()).unwrap(),
    //     data
    // );
}
