//! The crate that *defines* the object. crate_b passes it through an `Option` in its exported
//! API, and the library that bindgen reads contains both crates.

use std::sync::Arc;

uniffi::setup_scaffolding!();

/// An object. When another crate reads or writes it through a RustBuffer (e.g. inside an
/// `Option`), uniffi-bindgen-cs hands that crate's `BigEndianStream` to this crate's converter.
#[derive(uniffi::Object)]
pub struct Widget {
    id: u32,
}

#[uniffi::export]
impl Widget {
    #[uniffi::constructor]
    pub fn new(id: u32) -> Arc<Self> {
        Arc::new(Self { id })
    }

    pub fn id(&self) -> u32 {
        self.id
    }
}
