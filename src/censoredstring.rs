use serde::{de, ser, Deserialize, Serialize};
use std::convert::AsRef;
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

pub static CENSOR_STRING: &str = "***REMOVED***";

pub struct CensoredString {
    data: String,
    pub representation: String,
}

impl CensoredString {
    /// Take ownership of a secret value
    pub fn new<T: AsRef<str> + ToString>(secret: &T, representation: Option<&T>) -> Self {
        match representation {
            Some(value) => Self {
                data: secret.to_string(),
                representation: value.to_string(),
            },
            None => Self {
                data: secret.to_string(),
                representation: CENSOR_STRING.to_owned(),
            },
        }
    }
}

impl AsRef<str> for CensoredString {
    fn as_ref(&self) -> &str {
        &self.data
    }
}

// Antipattern Deref polymorphism to emulate inheritance. Read https://github.com/rust-unofficial/patterns/blob/main/anti_patterns/deref.md
impl Deref for CensoredString {
    type Target = String;
    fn deref(&self) -> &String {
        &self.data
    }
}

impl Serialize for CensoredString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        self.as_ref().serialize(serializer)
    }
}

impl FromStr for CensoredString {
    type Err = core::convert::Infallible;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            data: src.to_owned(),
            representation: CENSOR_STRING.to_owned(),
        })
    }
}

impl From<String> for CensoredString {
    fn from(src: String) -> Self {
        Self {
            data: src,
            representation: CENSOR_STRING.to_owned(),
        }
    }
}

impl<'de> Deserialize<'de> for CensoredString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(CensoredString::from)
    }
}

impl fmt::Debug for CensoredString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let debug_repr = format! {"newsletter-rs::CensoredString{{pub representation: String::from({})}}",&self.representation.as_str()};
        fmt::Display::fmt(&debug_repr, f)
    }
}

impl fmt::Display for CensoredString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.representation, f)
    }
}

#[cfg(test)]
mod tests {
    use crate::censoredstring::{CensoredString, CENSOR_STRING};
    use arbtest::arbtest;
    use std::str::FromStr;

    #[test]
    fn random_fuzz_censoredstring_display() {
        arbtest(|u| {
            let value = u.arbitrary::<String>()?;
            let censored_string: CensoredString = CensoredString::new(&value, None);
            assert_eq!(format! {"{}",censored_string}, CENSOR_STRING);
            Ok(())
        })
        .budget_ms(500)
        .run();
    }

    #[test]
    fn random_fuzz_censoredstring_debug() {
        arbtest(|u| {
            let value = u.arbitrary::<String>()?;
            // dbg!(&value);   // cargo test random_fuzz_censoredstring_debug -- --nocapture
            let censored_string: CensoredString = CensoredString::from_str(&value).unwrap();
            let debug_repr = format!{"newsletter-rs::CensoredString{{pub representation: String::from({})}}",CENSOR_STRING};
            assert_eq!(format!{"{:?}",censored_string}, debug_repr);
            Ok(())
        }).budget_ms(500).run();
    }
}
