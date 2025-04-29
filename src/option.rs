//! Convenience module to allow serialization via `humantime_serde` for `Option`
//!
//! # Example
//!
//! ```
//! use serde::{Serialize, Deserialize};
//! use core::time::Duration;
//!
//! use chrono::{DateTime, Utc};
//!
//! #[derive(Serialize, Deserialize)]
//! struct Foo {
//!     #[serde(default)]
//!     #[serde(with = "humantime_serde::option")]
//!     timeout: Option<Duration>,
//!     #[serde(default)]
//!     #[serde(with = "humantime_serde::option")]
//!     time: Option<DateTime<Utc>>,
//! }
//! ```

use super::Serde;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Serializes an `Option<Duration>` or `Option<chrono::DateTime>`
///
/// This function can be used with `serde_derive`'s `with` and
/// `deserialize_with` annotations.
pub fn serialize<T, S>(d: &Option<T>, s: S) -> Result<S::Ok, S::Error>
where
    for<'a> Serde<&'a T>: Serialize,
    S: Serializer,
{
    let nested: Option<Serde<&T>> = d.as_ref().map(Into::into);
    nested.serialize(s)
}

/// Deserialize an `Option<Duration>` or `Option<chrono::DateTime>`
///
/// This function can be used with `serde_derive`'s `with` and
/// `deserialize_with` annotations.
pub fn deserialize<'a, T, D>(d: D) -> Result<Option<T>, D::Error>
where
    Serde<T>: Deserialize<'a>,
    D: Deserializer<'a>,
{
    let got: Option<Serde<T>> = Deserialize::deserialize(d)?;
    Ok(got.map(Serde::into_inner))
}
