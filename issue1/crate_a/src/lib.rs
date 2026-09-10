//! The crate that *defines* the custom type. crate_b uses it in its exported API, and the
//! library that bindgen reads contains both crates.

uniffi::setup_scaffolding!();

/// A custom type backed by `Vec<u8>`.
///
/// uniffi-bindgen-cs represents custom types as a C# `using` alias in the file of the crate
/// that defines them. A crate that only *uses* the type gets no alias.
pub struct Blob(pub Vec<u8>);

uniffi::custom_type!(Blob, Vec<u8>, {
    lower: |blob| blob.0,
    try_lift: |bytes| Ok(Blob(bytes)),
});

#[uniffi::export]
pub fn empty_blob() -> Blob {
    Blob(Vec::new())
}
