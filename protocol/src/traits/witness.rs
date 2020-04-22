use bytes::Bytes;

pub trait SigVerify {
    fn verify(&mut self, witness: Bytes) -> bool;
}