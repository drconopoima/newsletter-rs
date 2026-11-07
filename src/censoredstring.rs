use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Deref;

pub static CENSOR_STRING: &str = "***REMOVED***";

#[derive(Clone)]
pub struct CensoredString {
    data: String,
    representation: String,
}

impl CensoredString {
    pub fn new<S: Into<String>>(secret: S, representation: Option<S>) -> Self {
        let secret_str: String = secret.into();
        Self {
            data: secret_str,
            representation: representation
                .map(|r| r.into())
                .unwrap_or_else(|| CENSOR_STRING.to_string()),
        }
    }

    pub fn from_secret<S: Into<String>>(secret: S) -> Self {
        Self::new(secret, None)
    }

    pub fn as_str(&self) -> &str {
        &self.data
    }

    pub fn into_inner(self) -> String {
        self.data
    }
}

// Antipattern Deref polymorphism to emulate inheritance. Read https://github.com/rust-unofficial/patterns/blob/main/anti_patterns/deref.md
impl Deref for CensoredString {
    type Target = str;
    fn deref(&self) -> &str {
        &self.data
    }
}

impl AsRef<str> for CensoredString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<String> for CensoredString {
    fn from(s: String) -> Self {
        Self::from_secret(s)
    }
}

impl From<&str> for CensoredString {
    fn from(s: &str) -> Self {
        Self::from_secret(s)
    }
}

impl Serialize for CensoredString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.data.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CensoredString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(Self::from_secret)
    }
}

impl fmt::Display for CensoredString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.representation)
    }
}

impl fmt::Debug for CensoredString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.representation)
    }
}

pub struct ZeroizingString {
    data: Vec<u8>,
}

impl ZeroizingString {
    pub fn new(s: &str) -> Self {
        Self {
            data: s.as_bytes().to_vec(),
        }
    }

    pub fn as_str(&self) -> &str {
        // Safety: We only create from &str, which guarantees valid UTF-8
        unsafe { std::str::from_utf8_unchecked(&self.data) }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Drop for ZeroizingString {
    fn drop(&mut self) {
        self.data.iter_mut().for_each(|b| *b = 0);
    }
}

// Antipattern Deref polymorphism to emulate inheritance. Read https://github.com/rust-unofficial/patterns/blob/main/anti_patterns/deref.md
impl Deref for ZeroizingString {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for ZeroizingString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for ZeroizingString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", CENSOR_STRING)
    }
}

impl std::fmt::Debug for ZeroizingString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", CENSOR_STRING)
    }
}

impl From<String> for ZeroizingString {
    fn from(s: String) -> Self {
        Self::new(&s)
    }
}

impl From<&str> for ZeroizingString {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl Clone for ZeroizingString {
    fn clone(&self) -> Self {
        Self::new(self.as_str())
    }
}

impl Serialize for ZeroizingString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        CENSOR_STRING.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ZeroizingString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(Self::from)
    }
}
