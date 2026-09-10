//! Uses crate_a's object `Widget` inside an `Option` in its exported API; compiled into
//! libcrate_b.dylib.
//!
//! The `Option` makes crate_b.cs generate `FfiConverterOptionalTypeWidget`, whose Read/Write
//! call crate_a's `FfiConverterTypeWidget` with crate_b's own `BigEndianStream`. A plain
//! `Arc<Widget>` argument would not trigger it: Lower/Lift only pass a `ulong` handle.

use std::sync::Arc;

use crate_a::Widget;

uniffi::setup_scaffolding!();

#[uniffi::export]
pub fn widget_id(widget: Option<Arc<Widget>>) -> u32 {
    widget.map(|w| w.id()).unwrap_or(0)
}
