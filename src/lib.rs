#![no_std]
// #![doc(html_root_url = "https://docs.rs/humantime-serde/1.0")]
#![forbid(unsafe_code)]

//! Serde support for the `humantime` crate.
//!
//! Based on [this fork](https://github.com/tailhook/serde-humantime/tree/serde_wrapper).
//!
//! Currently `std::time::Duration` and `chrono::DateTime` are supported.
//!
//! # Example
//! ```
//! use serde::{Serialize, Deserialize};
//! use std::time::Duration;
//! use chrono::{DateTime, Utc};
//!
//! #[derive(Serialize, Deserialize)]
//! struct Foo {
//!     #[serde(with = "humantime_serde")]
//!     timeout: Duration,
//!     #[serde(default)]
//!     #[serde(with = "humantime_serde")]
//!     time: Option<DateTime<Utc>>,
//! }
//! ```
//!
//! Or use the `Serde` wrapper type:
//!
//! ```
//! use serde::{Serialize, Deserialize};
//! use humantime_serde::Serde;
//! use chrono::{DateTime, Utc};
//!
//! #[derive(Serialize, Deserialize)]
//! struct Foo {
//!     timeout: Vec<Serde<DateTime<Utc>>>,
//! }
//! ```

extern crate alloc;

/// Reexport module.
pub mod re {
    pub use humantime;
}

pub mod option;

use alloc::string::ToString;
use core::fmt;
use core::ops::{Deref, DerefMut};
use core::time::{Duration};

use serde::{de, ser, Deserialize, Deserializer, Serialize, Serializer};
use chrono::{DateTime, FixedOffset, SecondsFormat, Utc};

/// Deserializes a `Duration` or `DateTime<Tz>` via the humantime crate.
///
/// This function can be used with `serde_derive`'s `with` and
/// `deserialize_with` annotations.
pub fn deserialize<'a, T, D>(d: D) -> Result<T, D::Error>
where
    Serde<T>: Deserialize<'a>,
    D: Deserializer<'a>,
{
    Serde::deserialize(d).map(Serde::into_inner)
}

/// Serializes a `Duration` or `DateTime<Tz>` via the humantime crate.
///
/// This function can be used with `serde_derive`'s `with` and
/// `serialize_with` annotations.
pub fn serialize<T, S>(d: &T, s: S) -> Result<S::Ok, S::Error>
where
    for<'a> Serde<&'a T>: Serialize,
    S: Serializer,
{
    Serde::from(d).serialize(s)
}

/// A wrapper type which implements `Serialize` and `Deserialize` for
/// types involving `DateTime<Tz>` and `Duration`.
#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct Serde<T>(T);

impl<T> fmt::Debug for Serde<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.0.fmt(formatter)
    }
}

impl<T> Deref for Serde<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Serde<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> Serde<T> {
    /// Consumes the `De`, returning the inner value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> From<T> for Serde<T> {
    fn from(val: T) -> Serde<T> {
        Serde(val)
    }
}

impl<'de> Deserialize<'de> for Serde<Duration> {
    fn deserialize<D>(d: D) -> Result<Serde<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct V;

        impl de::Visitor<'_> for V {
            type Value = Duration;

            fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt.write_str("a duration")
            }

            fn visit_str<E>(self, v: &str) -> Result<Duration, E>
            where
                E: de::Error,
            {
                humantime::parse_duration(v).map_err(|_| {
                    E::invalid_value(de::Unexpected::Str(v), &self)
                })
            }
        }

        d.deserialize_str(V).map(Serde)
    }
}

impl<'de> Deserialize<'de> for Serde<DateTime<Utc>> {
    fn deserialize<D>(d: D) -> Result<Serde<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct V;

        impl de::Visitor<'_> for V {
            type Value = DateTime<Utc>;

            fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt.write_str("a timestamp")
            }

            fn visit_str<E>(self, v: &str) -> Result<DateTime<Utc>, E>
            where
                E: de::Error,
            {
                Ok(
                    DateTime::parse_from_rfc3339(v).map_err(
                        |_| E::invalid_value(de::Unexpected::Str(v), &self)
                    )?.to_utc()
                )
            }
        }

        d.deserialize_str(V).map(Serde)
    }
}

impl<'de> Deserialize<'de> for Serde<DateTime<FixedOffset>> {
    fn deserialize<D>(d: D) -> Result<Serde<DateTime<FixedOffset>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct V;

        impl de::Visitor<'_> for V {
            type Value = DateTime<FixedOffset>;

            fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt.write_str("a timestamp")
            }

            fn visit_str<E>(self, v: &str) -> Result<DateTime<FixedOffset>, E>
            where
                E: de::Error,
            {
                DateTime::parse_from_rfc3339(v).map_err(
                    |_| E::invalid_value(de::Unexpected::Str(v), &self)
                )
            }
        }

        d.deserialize_str(V).map(Serde)
    }
}

impl<'de> Deserialize<'de> for Serde<Option<Duration>> {
    fn deserialize<D>(d: D) -> Result<Serde<Option<Duration>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match Option::<Serde<Duration>>::deserialize(d)? {
            Some(Serde(dur)) => Ok(Serde(Some(dur))),
            None => Ok(Serde(None)),
        }
    }
}

impl<'de> Deserialize<'de> for Serde<Option<DateTime<Utc>>> {
    fn deserialize<D>(d: D) -> Result<Serde<Option<DateTime<Utc>>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match Option::<Serde<DateTime<Utc>>>::deserialize(d)? {
            Some(Serde(dur)) => Ok(Serde(Some(dur))),
            None => Ok(Serde(None)),
        }
    }
}

impl<'de> Deserialize<'de> for Serde<Option<DateTime<FixedOffset>>> {
    fn deserialize<D>(d: D) -> Result<Serde<Option<DateTime<FixedOffset>>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match Option::<Serde<DateTime<FixedOffset>>>::deserialize(d)? {
            Some(Serde(dur)) => Ok(Serde(Some(dur))),
            None => Ok(Serde(None)),
        }
    }
}

impl ser::Serialize for Serde<&Duration> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        humantime::format_duration(*self.0)
            .to_string()
            .serialize(serializer)
    }
}

impl ser::Serialize for Serde<Duration> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        humantime::format_duration(self.0)
            .to_string()
            .serialize(serializer)
    }
}

impl ser::Serialize for Serde<&DateTime<Utc>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        self.0.to_rfc3339_opts(SecondsFormat::Secs, true)
            .serialize(serializer)
    }
}

impl ser::Serialize for Serde<&DateTime<FixedOffset>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        self.0.to_rfc3339_opts(SecondsFormat::Secs, true)
            .serialize(serializer)
    }
}

impl ser::Serialize for Serde<DateTime<Utc>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        self.0.to_rfc3339_opts(SecondsFormat::Secs, true)
            .serialize(serializer)
    }
}

impl ser::Serialize for Serde<DateTime<FixedOffset>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        self.0.to_rfc3339_opts(SecondsFormat::Secs, true)
            .serialize(serializer)
    }
}

impl ser::Serialize for Serde<&Option<Duration>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match *self.0 {
            Some(dur) => serializer.serialize_some(&Serde(dur)),
            None => serializer.serialize_none(),
        }
    }
}

impl ser::Serialize for Serde<Option<Duration>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        Serde(&self.0).serialize(serializer)
    }
}

impl ser::Serialize for Serde<&Option<DateTime<Utc>>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match *self.0 {
            Some(tm) => serializer.serialize_some(&Serde(tm)),
            None => serializer.serialize_none(),
        }
    }
}

impl ser::Serialize for Serde<&Option<DateTime<FixedOffset>>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match *self.0 {
            Some(tm) => serializer.serialize_some(&Serde(tm)),
            None => serializer.serialize_none(),
        }
    }
}

impl ser::Serialize for Serde<Option<DateTime<Utc>>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        Serde(&self.0).serialize(serializer)
    }
}

impl ser::Serialize for Serde<Option<DateTime<FixedOffset>>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        Serde(&self.0).serialize(serializer)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn with() {
        #[derive(Serialize, Deserialize)]
        struct Foo {
            #[serde(with = "super")]
            time: Duration,
        }

        let json = r#"{"time": "15 seconds"}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time, Duration::from_secs(15));
        let reverse = serde_json::to_string(&foo).unwrap();
        assert_eq!(reverse, r#"{"time":"15s"}"#);
    }

    #[test]
    fn with_option() {
        #[derive(Serialize, Deserialize)]
        struct Foo {
            #[serde(with = "super", default)]
            time: Option<Duration>,
        }

        let json = r#"{"time": "15 seconds"}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time, Some(Duration::from_secs(15)));
        let reverse = serde_json::to_string(&foo).unwrap();
        assert_eq!(reverse, r#"{"time":"15s"}"#);

        let json = r#"{"time": null}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time, None);
        let reverse = serde_json::to_string(&foo).unwrap();
        assert_eq!(reverse, r#"{"time":null}"#);

        let json = r#"{}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time, None);
    }

    #[test]
    fn time() {
        #[derive(Serialize, Deserialize)]
        struct Foo {
            #[serde(with = "super")]
            time: DateTime<Utc>,
        }

        let json = r#"{"time": "2018-05-11T18:28:30Z"}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time, DateTime::UNIX_EPOCH + Duration::new(1526063310, 0));
        let reverse = serde_json::to_string(&foo).unwrap();
        assert_eq!(reverse, r#"{"time":"2018-05-11T18:28:30Z"}"#);
    }

    #[test]
    fn time_with_option() {
        #[derive(Serialize, Deserialize)]
        struct Foo {
            #[serde(with = "super", default)]
            time: Option<DateTime<Utc>>,
        }

        let json = r#"{"time": "2018-05-11T18:28:30Z"}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time, Some(DateTime::UNIX_EPOCH + Duration::new(1526063310, 0)));
        let reverse = serde_json::to_string(&foo).unwrap();
        assert_eq!(reverse, r#"{"time":"2018-05-11T18:28:30Z"}"#);

        let json = r#"{"time": null}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time, None);
        let reverse = serde_json::to_string(&foo).unwrap();
        assert_eq!(reverse, r#"{"time":null}"#);

        let json = r#"{}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time, None);
    }

    #[test]
    fn test_readme_deps() {
        version_sync::assert_markdown_deps_updated!("README.md");
    }

    #[test]
    fn test_html_root_url() {
        version_sync::assert_html_root_url_updated!("src/lib.rs");
    }
}
