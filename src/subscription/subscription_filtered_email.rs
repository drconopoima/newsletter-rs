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
        if !email_format.is_match(&lowercase_email) {
            return Err(format!(
                "Provided email '{}' has invalid formatting.",
                email
            ));
        }
        Ok(Self(lowercase_email.to_owned()))
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

#[cfg(test)]
mod tests {
    use crate::subscription::SubscriptionFilteredEmail;
    use arbtest::arbtest;
    use claims::{assert_err, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;
    use quickcheck::{Arbitrary, Gen};
    use quickcheck_macros;
    use rand::{distributions::WeightedIndex, prelude::*};
    use rand::{rngs::StdRng, SeedableRng};
    use std::str::FromStr;

    #[test]
    fn random_fuzz_email_nopanic() {
        arbtest(|u| {
            let fuzz = u.arbitrary().expect("");
            // dbg!(&fuzz);   // cargo test random_fuzz_email_nopanic -- --nocapture
            let _ = SubscriptionFilteredEmail::new(fuzz);
            Ok(())
        })
        .budget_ms(1_250)
        .run();
    }

    #[test]
    fn rejects_missing_at_symbol() {
        let tests = vec!["email.drconopoima.com", "[::1].127.0.0.1"];
        for input in tests {
            assert_err!(SubscriptionFilteredEmail::parse(&input));
        }
    }

    #[derive(Clone, Debug)]
    struct ValidEmail(String);
    impl Arbitrary for ValidEmail {
        fn arbitrary(g: &mut Gen) -> Self {
            let mut rng = StdRng::seed_from_u64(u64::arbitrary(g));
            let email = SafeEmail().fake_with_rng(&mut rng);
            ValidEmail(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn accepts_random_faked_email(valid_email: ValidEmail) -> bool {
        // dbg!(&valid_email.0);   // cargo test accepts_random_faked_email -- --nocapture
        SubscriptionFilteredEmail::parse(&valid_email.0).is_ok()
    }

    #[test]
    fn rejects_missing_subject_address() {
        let tests = vec!["@drconopoima.com", "@127.0.0.1"];
        for input in tests {
            assert_err!(SubscriptionFilteredEmail::from_str(&input));
        }
    }

    #[test]
    fn accepts_standard_looking_cases() {
        let tests = vec![
            "email@drconopoima.com",
            "hyphenated-email@here.and.there.com",
            "email@127.0.0.1",
            "email@localhost",
        ];
        for input in tests {
            assert_ok!(SubscriptionFilteredEmail::new(&input));
        }
    }

    #[test]
    fn accepts_input_needing_trimming() {
        let tests = vec![
            "address@host.local\n",
            " \tthisgotin@byaccidentaltypi.ng",
            "\nsomescript@unintended.input\t \n",
        ];
        for input in tests {
            assert_ok!(SubscriptionFilteredEmail::parse(&input));
        }
    }

    #[test]
    fn rejects_empty_blank_whitespace() {
        let tests = vec!["", " \t", "\n\t \n"];
        let mut rng = thread_rng();
        let methods_weights = [("new", 1), ("parse", 1), ("from_str", 1)];
        let sampling_methods =
            WeightedIndex::new(methods_weights.iter().map(|weight| weight.1)).unwrap();
        let results: Vec<Result<SubscriptionFilteredEmail, String>> = tests
            .into_iter()
            .map(|input| {
                let method = methods_weights[sampling_methods.sample(&mut rng)].0;
                if method.eq("new") {
                    SubscriptionFilteredEmail::new(&input)
                } else if method.eq("from_str") {
                    SubscriptionFilteredEmail::from_str(&input)
                } else {
                    SubscriptionFilteredEmail::parse(&input)
                }
            })
            .collect();
        for result in results {
            assert_err!(result);
        }
    }

    #[test]
    fn rejects_missing_tld() {
        let tests = vec!["abc", "abc@"];
        for input in tests {
            assert_err!(SubscriptionFilteredEmail::new(&input));
        }
    }

    #[test]
    fn rejects_intermediate_whitespace() {
        let tests = vec!["a @x.yz", "a\n@b.net"];
        for input in tests {
            assert_err!(SubscriptionFilteredEmail::from_str(&input));
        }
    }

    #[test]
    fn accepts_domain_label_63_characters() {
        let mut long_tld = "admin@local.".to_owned();
        long_tld.extend("y".repeat(63).chars());
        let mut long_domain_label = "email@".to_owned();
        long_domain_label.extend("y".repeat(63).chars());
        let long_domain = format!("{}.com", long_domain_label);
        let mut long_sub_domain_label = "anonymous@".to_owned();
        long_sub_domain_label.extend("x".repeat(63).chars());
        long_sub_domain_label.extend(".".repeat(1).chars());
        long_sub_domain_label.extend("z".repeat(63).chars());
        let long_sub_domain = format!("{}.net", long_sub_domain_label);
        let tests = vec![long_tld, long_domain, long_sub_domain];
        for input in tests {
            assert_ok!(SubscriptionFilteredEmail::from_str(&input));
        }
    }

    #[test]
    fn rejects_domain_label_64_characters() {
        let mut long_tld = "admin@abc.".to_owned();
        long_tld.extend("n".repeat(64).chars());
        let mut long_domain_label = "email@".to_owned();
        long_domain_label.extend("y".repeat(64).chars());
        let long_domain = format!("{}.com", long_domain_label);
        let mut long_sub_domain_label = "anonymous@".to_owned();
        long_sub_domain_label.extend("x".repeat(63).chars());
        long_sub_domain_label.extend(".".repeat(1).chars());
        long_sub_domain_label.extend("y".repeat(64).chars());
        long_sub_domain_label.extend(".".repeat(1).chars());
        long_sub_domain_label.extend("z".repeat(63).chars());
        let long_sub_domain = format!("{}.net", long_sub_domain_label);
        let tests = vec![long_tld, long_domain, long_sub_domain];
        for input in tests {
            assert_err!(SubscriptionFilteredEmail::parse(&input));
        }
    }

    #[test]
    fn django_non_ipv6() {
        // A few Django test cases
        // https://github.com/django/django/blob/master/tests/validators/tests.py#L48
        let tests =
            vec![
            (r#"!def!xyz%abc@example.com"#, true),
            ("example@valid-----hyphens.com", true),
            ("example@valid-with-hyphens.com", true),
            (r#""test@test"@example.com"#, false),
            // domain name labels up to 63 characters per RFC 1034
            ("a@atm.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", true),
            ("a@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.atm", true),
            (
                "a@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.bbbbbbbbbb.atm",
                true,
            ),
            // 64 * a
            ("a@atm.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", false),
            ("abc@.com", false),
            ("something@@somewhere.com", false),
            ("email@[127.0.0.256]", false),
            ("example@invalid-.com", false),
            ("example@-invalid.com", false),
            ("example@invalid.com-", false),
            ("example@inv-.alid-.com", false),
            ("example@inv-.-alid.com", false),
            (r#"test@example.com\n\n<script src="x.js">"#, false),
            (r#""\\\011"@here.com"#, false),
            (r#""\\\012"@here.com"#, false),
            ("trailingdot@shouldfail.com.", false),
            (r#""test@test"\n@example.com"#, false),
            // underscores are not allowed
            ("John.Doe@exam_ple.com", false),
        ];
        let mut rng = thread_rng();
        let methods_weights = [("new", 1), ("parse", 1), ("from_str", 1)];
        let sampling_methods =
            WeightedIndex::new(methods_weights.iter().map(|weight| weight.1)).unwrap();
        for (input, expected) in tests {
            let method = methods_weights[sampling_methods.sample(&mut rng)].0;
            let result: Result<SubscriptionFilteredEmail, String> = {
                if method.eq("new") {
                    SubscriptionFilteredEmail::new(&input)
                } else if method.eq("from_str") {
                    SubscriptionFilteredEmail::from_str(&input)
                } else {
                    SubscriptionFilteredEmail::parse(&input)
                }
            };
            if expected {
                assert_ok!(result);
            } else {
                assert_err!(result);
            };
        }
    }
}
