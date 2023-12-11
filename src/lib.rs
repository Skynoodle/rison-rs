//! Rison is a data serialization format based on JSON, optimized for
//! compactness in URIs.
//!
//! The format supported by this implementation is based on the documentation
//! and implementations found below:
//! - <https://github.com/Nanonid/rison>
//! - <https://github.com/w33ble/rison-node>
//!
//! The deserializer implementation is broadly inspired by the existing
//! `serde_json` library which provides a `serde` serializer and
//! deserializer for the standard JSON format.

pub mod de;
pub mod error;

#[doc(inline)]
pub use error::{Error, Result};

#[doc(inline)]
pub use de::{from_reader, from_slice, from_str, Deserializer};
