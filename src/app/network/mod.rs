use crc32fast::Hasher;

mod server;
mod client;
mod networking;

fn compute_checksum(data: &[u8]) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(data);
    hasher.finalize()
}

pub use server::*;
pub use client::*;
pub use networking::*;