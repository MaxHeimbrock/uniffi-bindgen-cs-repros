//! Uses crate_a's custom type `Blob` in its exported API; compiled into libcrate_b.dylib.
//!
//! In the generated C#, `Blob` is a `using` alias that only crate_a.cs declares. crate_b.cs
//! uses the name in its signatures without declaring it, and C# aliases do not cross files.

use crate_a::Blob;

uniffi::setup_scaffolding!();

#[uniffi::export]
pub fn make_blob(len: u32) -> Blob {
    Blob(vec![0xAB; len as usize])
}

#[uniffi::export]
pub fn blob_len(blob: Blob) -> u32 {
    blob.0.len() as u32
}
