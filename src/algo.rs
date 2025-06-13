use blake3;
use sha2::{Digest, Sha256};

pub trait Hasher: Send {
    fn update(&mut self, data: &[u8]);
    fn finalize(self: Box<Self>) -> [u8; 32];
}

impl Hasher for Sha256 {
    fn update(&mut self, data: &[u8]) {
        Digest::update(self, data);
    }

    fn finalize(self: Box<Self>) -> [u8; 32] {
        let result = Digest::finalize(*self);
        let mut out = [0u8; 32];
        out.copy_from_slice(&result);
        out
    }
}

pub struct Blake3Wrapper(blake3::Hasher);

impl Blake3Wrapper {
    pub fn new() -> Self {
        Self(blake3::Hasher::new())
    }
}

impl Hasher for Blake3Wrapper {
    fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }

    fn finalize(self: Box<Self>) -> [u8; 32] {
        *self.0.finalize().as_bytes()
    }
}
