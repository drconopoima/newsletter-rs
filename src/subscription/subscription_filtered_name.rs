use regex::Regex;
use std::convert::AsRef;
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug)]
pub struct SubscriptionFilteredName(String);

impl SubscriptionFilteredName {
    pub fn new(name: &str) -> Result<Self, String> {
        Self::parse(name)
    }
    pub fn parse(name: &str) -> Result<Self, String> {
        let trimmed_name = name.trim();
        let is_empty_or_whitespace = trimmed_name.is_empty();
        if is_empty_or_whitespace {
            return Err(format!("Provided name '{}' appears to be blank or empty which is invalid. Please fill out a name to subscribe", name));
        }
        let intermediate_whitespace = Regex::new(r"^\s+|\s+$|\s+").unwrap();
        let name_middle_trim = intermediate_whitespace
            .replace_all(trimmed_name, " ")
            .into_owned();

        let forbidden_chars = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_chars = name_middle_trim
            .chars()
            .any(|g| forbidden_chars.contains(&g));

        if contains_forbidden_chars {
            return Err(format!("Provided name '{}' contains one or more characters from the following forbidden list '/()\"<>\\{{}}'. Please remove these characters to subscribe.", name_middle_trim));
        }

        let is_too_long = name_middle_trim.len() > 254;

        if !is_too_long {
            Ok(Self(name_middle_trim))
        } else {
            Err(format!("Provided name '{}' is longer than the limit of 254 characters. Please provide a nickname to subscribe.", name_middle_trim))
        }
    }
}

impl AsRef<str> for SubscriptionFilteredName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// Antipattern Deref polymorphism to emulate inheritance. Read https://github.com/rust-unofficial/patterns/blob/main/anti_patterns/deref.md
impl Deref for SubscriptionFilteredName {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl FromStr for SubscriptionFilteredName {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl fmt::Display for SubscriptionFilteredName {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[cfg(test)]
mod tests {
    use crate::subscription::SubscriptionFilteredName;
    use claims::{assert_err, assert_ok};
    use std::str::FromStr;
    use rand::{prelude::*,distributions::WeightedIndex};

    #[test]
    fn rejects_255_characters_input() {
        let name = "n".repeat(255);
        assert_err!(SubscriptionFilteredName::new(&name));
    }
    #[test]
    fn accepts_254_characters_input() {
        let name = "y".repeat(254);
        assert_ok!(SubscriptionFilteredName::from_str(&name));
    }

    #[test]
    // At least while it isn't mandated otherwise
    fn accepts_cancelled_celebrities() {
        let tests = vec!(
            "J. K. Rowling",
            "Matt Damon",
            "Jimmy Donaldson",
            "Ye West",
            "Logan Paul",
            "boogie2988",
            "SSSniperWolf"
        );
        for input in tests {
            assert_ok!(
                SubscriptionFilteredName::new(&input)
            );
        }
    }

    #[test]
    fn accepts_input_needing_trimming() {
        let tests = vec!(
            "We are anonymous!\n",
            "\n \tWe know exactly who they are \t",
            "\nRyan Sees Through Copper\t \n"
        );
        for input in tests {
            assert_ok!(
                SubscriptionFilteredName::new(&input)
            );
        };
    }

    #[test]
    fn accepts_intermediate_whitespace(){
        let tests = vec!(
            "Jose   Felix \t \n \
                Ribas",
            "This \t    \n keyboard\t \
            jumps \t \t\n    around   a lot"
        );
        for input in tests {
            assert_ok!(
                SubscriptionFilteredName::new(&input)
            );
        };
    }

    #[test]
    fn rejects_empty_blank_whitespace() {
        let tests = vec!(
            "",
            " \t",
            "\n\t \n"
        );
        let mut rng = thread_rng();
        let methods_weights = [("new", 1), ("parse", 1), ("from_str", 1)];
        let sampling_methods = WeightedIndex::new(methods_weights.iter().map(|weight| weight.1)).unwrap();
        let results: Vec<Result<SubscriptionFilteredName, String>> = tests.into_iter().map(|input| {
            let method = methods_weights[sampling_methods.sample(&mut rng)].0;
            if method.eq("new") {
                SubscriptionFilteredName::new(&input)
            } else if method.eq("from_str") {
                SubscriptionFilteredName::from_str(&input)
            } else {
                SubscriptionFilteredName::parse(&input)
            }
        }).collect();
        println!("{:?}",&results);
        for result in results {
            assert_err!(result);
        }
    }
}
