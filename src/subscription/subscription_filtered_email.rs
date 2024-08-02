use regex::Regex;
use std::convert::AsRef;
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug)]
pub struct SubscriptionFilteredEmail(String);

impl SubscriptionFilteredEmail {
    pub fn new(email: &str) -> Result<Self, String> {
        Self::parse(email)
    }
    pub fn parse(email: &str) -> Result<Self, String> {
        let lowercase_email = email.to_lowercase().trim().to_owned();
        let is_empty_or_whitespace = lowercase_email.is_empty();
        if is_empty_or_whitespace {
            return Err(format!(
                "Provided email '{}' appears to be blank or empty which is invalid.",
                email
            ));
        }
        let contains_intermediate_whitespace = Regex::new(r"^\s+|\s+$|\s+").unwrap();
        if contains_intermediate_whitespace.is_match(&lowercase_email) {
            return Err(format!(
                "Provided email '{}' appears to contain intermediate whitespace which is invalid.",
                email
            ));
        }
        // MDN web docs provide a regular expression matching emails
        // https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input/email#validation
        let email_format = Regex::new(r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").unwrap();
        if email_format.is_match(&lowercase_email) {
            Ok(Self(lowercase_email.to_owned()))
        } else {
            Err(format!(
                "Provided email '{}' has invalid formatting.",
                email
            ))
        }
    }
}

impl AsRef<str> for SubscriptionFilteredEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// Antipattern Deref polymorphism to emulate inheritance. Read https://github.com/rust-unofficial/patterns/blob/main/anti_patterns/deref.md
impl Deref for SubscriptionFilteredEmail {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl fmt::Display for SubscriptionFilteredEmail {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl FromStr for SubscriptionFilteredEmail {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

/* #[cfg(test)]
mod tests {
    #[test]
    fn email_rejects_empty_input() {
        unimplemented!()
    }
} */
