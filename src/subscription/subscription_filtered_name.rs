use std::convert::AsRef;
use std::str::FromStr;
use regex::Regex;
use std::fmt;

#[derive(Debug, serde::Deserialize)]
pub struct SubscriptionFilteredName(String);

impl SubscriptionFilteredName {
    pub fn new(name: &str) -> Result<Self, String> {
        Self::parse(name)
    }
    pub fn parse(name: &str) -> Result<Self, String> {
        let trimmed_name = name.trim();
        let is_empty_or_whitespace = trimmed_name.is_empty();
        if is_empty_or_whitespace {
            panic!("Provided name '{}' appears to be blank or empty which is invalid. Please fill out a name to subscribe", name)
        }
        let intermediate_whitespace = Regex::new(r"^\s+|\s+$|\s+").unwrap();
        let name_middle_trim = intermediate_whitespace.replace_all(trimmed_name, " ").into_owned();
        
        let forbidden_chars = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_chars = name_middle_trim.chars().any(|g| forbidden_chars.contains(&g));
        
        if contains_forbidden_chars {
            panic!("Provided name '{}' contains one or more characters from the following forbidden list '/()\"<>\\{{}}'. Please remove these characters to subscribe.", name_middle_trim)
        }

        let is_too_long = name_middle_trim.len() > 254;

        if !is_too_long {
            Ok(Self(name_middle_trim))
        } else {
            panic!("Provided name '{}' is longer than the limit of 254 characters. Please provide a nickname to subscribe.", name_middle_trim)
        }
    }
}


impl AsRef<str> for SubscriptionFilteredName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromStr for SubscriptionFilteredName {
    type Err = String;

    fn from_str(s: &str) -> Result<Self,Self::Err> {
        Self::new(s)
    }
}

impl fmt::Display for SubscriptionFilteredName {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/* #[cfg(test)]
mod tests {
    use crate::subscription::SubscriptionFilteredName;
    use std::str::FromStr;
    use claims::{assert_err, assert_ok};

    #[test]
    fn name_rejects_255_characters_input() {
        let name = "n".repeat(255);
        assert_err!(SubscriptionFilteredName::new(&name));
    }
    #[test]
    fn name_accepts_254_characters_input() {
        let name = "y".repeat(254);
        assert_ok!(SubscriptionFilteredName::from_str(&name));
    }
} */
