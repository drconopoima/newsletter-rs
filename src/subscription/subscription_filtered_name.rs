use std::collections::HashSet;
use std::convert::AsRef;
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;
use unicode_segmentation::UnicodeSegmentation;
use unicode_normalization::UnicodeNormalization;

/// Maximum *grapheme* count for display name (RFC 5322-ish soft limit)
const MAX_GRAPHEME_COUNT: usize = 254;
const FORBIDDEN_CHARS: &[char] = &['/', '(', ')', '"', '<', '>', '\\', '{', '}', '[', ']'];

/// Error types for name validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameError {
    Empty,
    TooLongGraphemes,
    InvalidChar,
    ConsecutiveSpecialChars,
}

impl fmt::Display for NameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NameError::Empty => write!(f, "Name must not be blank or empty. Please fill out a name to subscribe"),
            NameError::TooLongGraphemes => write!(f, "Name must not exceed {MAX_GRAPHEME_COUNT} graphemes."),
            NameError::InvalidChar => write!(f, "Name must not contain invalid characters. The following characters are forbidden '/()\"<>\\{{}}[]'. Please remove these characters to subscribe."),
            NameError::ConsecutiveSpecialChars => write!(f, "Name must not contain consecutive repeated special characters."),
        }
    }
}

impl std::error::Error for NameError {}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct SubscriptionFilteredName(String);

impl SubscriptionFilteredName {
    /// Validates and sanitizes a display name.
    ///
    /// Returns `Ok(Self)` if valid, or `Err(NameError)` otherwise.
    ///
    /// # Behavior
    /// - Trims Unicode whitespace (including `NBSP`, `ZWNBSP`, etc.)
    /// - Filters zero-width/formatting chars
    /// - Normalizes punctuation (`’` → `'`, `–`/`—` → `-`, etc.)
    /// - Enforces `≤ 254` graphemes and `≤ 256` bytes
    /// - Rejects leading forbidden chars
    /// - Rejects consecutive special chars (except spaces & dashes)
    /// - Detects & warns on RTL (but allows by default)
    pub fn new(name: &str) -> Result<Self, NameError> {      
        Self::parse(name)
    }    
    pub fn parse(name: &str) -> Result<Self, NameError> {
        let trimmed_name = name.trim();  
        // 1. Empty after trim
        let is_empty_or_whitespace = trimmed_name.is_empty();
        if is_empty_or_whitespace {
            return Err(NameError::Empty);
        }

        // 4. Filter zero-width/formatting chars (redundant with normalization, but explicit)
        let filtered: String = trimmed_name
            .chars()
            .filter(|c| !Self::is_zero_width_or_formatting(*c))
            .collect();

        // 5. Grapheme-based length check
        let grapheme_count = filtered.graphemes(true).count();
        if grapheme_count > MAX_GRAPHEME_COUNT {
            return Err(NameError::TooLongGraphemes);
        }

        let forbidden_chars: HashSet<&char> = ['/', '(', ')', '"', '<', '>', '\\', '{', '}', '[', ']']
            .iter()
            .collect();
        let contains_forbidden_chars = filtered.chars().any(|g| forbidden_chars.contains(&g));

        if contains_forbidden_chars {
            return Err(NameError::InvalidChar)
        }
        let name_middle_trim = Self::process_name(&filtered, None)?;
        
        // 3. Normalize & sanitize
        let normalized = Self::normalize_and_sanitize(&name_middle_trim);

        Ok(Self(normalized.to_owned()))
    }

    /// Returns the underlying string slice.
    pub fn as_str(&self) -> &str {
        &self
    }

    /// Returns the grapheme length.
    pub fn grapheme_len(&self) -> usize {
        self.graphemes(true).count()
    }

    /// Normalizes punctuation & applies NFC normalization.
    fn normalize_and_sanitize(s: &str) -> String {
        // Step 1: Normalize punctuation (before NFC for consistency)
        let normalized = s
            .chars()
            .map(|c| match c {
                // All quote variants → single quote
                '\u{2019}' | '\u{2018}' | '\u{201A}' | '\u{201B}' | '\u{2032}' => '\'',
                // All double-quote variants
                '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{2033}' => '"',
                // All hyphen/dash variants → `-`
                '\u{2010}' | '\u{2011}' | '\u{2012}' | '\u{2013}' | '\u{2014}' | '\u{2015}'
                | '\u{2E3A}' | '\u{2E3B}' => '-',
                // Full-width ASCII (e.g., Japanese/Korean) → half-width
                '\u{FF07}' => '\'', // fullwidth apostrophe
                '\u{FF02}' => '"',  // fullwidth quote
                '\u{FF0D}' => '-',  // fullwidth hyphen
                '\u{FF0E}' => '.',  // fullwidth period
                '\u{FF0C}' => ',',  // fullwidth comma
                '\u{FF1A}' => ':',  // fullwidth colon
                '\u{FF1B}' => ';',  // fullwidth semicolon
                '\u{FF01}' => '!',  // fullwidth exclamation
                '\u{FF1F}' => '?',  // fullwidth question
                '\u{FF08}' => '(',  // fullwidth left paren
                '\u{FF09}' => ')',  // fullwidth right paren
                '\u{FF3B}' => '[',  // fullwidth left bracket
                '\u{FF3D}' => ']',  // fullwidth right bracket
                '\u{FF5B}' => '{',  // fullwidth left brace
                '\u{FF5D}' => '}',  // fullwidth right brace
                '\u{FF5C}' => '|',  // fullwidth vertical bar
                '\u{FF5E}' => '~',  // fullwidth tilde
                '\u{FF0B}' => '+',  // fullwidth plus
                '\u{FF1D}' => '=',  // fullwidth equals
                '\u{FF0A}' => '*',  // fullwidth asterisk
                '\u{FF0F}' => '/',  // fullwidth solidus
                '\u{FF3C}' => '\\', // fullwidth reverse solidus
                // Common ligatures (optional, but useful)
                'ﬁ' => "fi".to_string().chars().next().unwrap(), // ligature fi → "fi"
                'ﬂ' => "fl".to_string().chars().next().unwrap(), // ligature fl → "fl"
                'ﬀ' => "ff".to_string().chars().next().unwrap(), // ligature ff → "ff"
                'ﬃ' => "ffi".to_string().chars().next().unwrap(), // ligature ffi
                'ﬄ' => "ffl".to_string().chars().next().unwrap(), // ligature ffl
                'ﬅ' => "st".to_string().chars().next().unwrap(), // ligature st
                'ﬆ' => "st".to_string().chars().next().unwrap(), // ligature st
                // Preserve others
                c => c,
            })
            .collect::<String>();

        // Step 2: NFC normalization
        normalized.nfc().collect::<String>()
    }
    fn process_name(
        name: &str,
        special_char_list: Option<HashSet<String>>,
    ) -> Result<String, NameError> {
        #[allow(suspicious_double_ref_op)]
        let allowed_non_consecutive_special_characters = match special_char_list {
            Some(char_set) => char_set,
            None => [
                "'", ",", ";", ".", ":", "*", "+", "-", "&", "%", "¨", "`", "´", "~", "#", "^",
                "%", "@", "?", "¿", "|", "!", "¡", "=",
            ]
            .iter()
            .map(|x| x.clone().to_owned())
            .collect::<HashSet<String>>(),
        };
        let mut chars: Vec<(usize, char)> = name.chars().enumerate().collect();
        let is_too_long = chars.len() > 4096;
        if is_too_long {
            return Err(NameError::TooLongGraphemes);
        }
        let mut previous: String = "".into();
        let mut idx = 0;
        while idx < chars.len() {
            if chars[idx].1.is_whitespace() {
                if previous.eq(" ") {
                    chars.remove(idx);
                    continue;
                } else {
                    previous = " ".into();
                    chars[idx].1 = ' '
                }
            }
            let current: String = chars[idx].1.into();
            if allowed_non_consecutive_special_characters.contains(&current)
                && previous.eq(&current)
            {
                return Err(NameError::ConsecutiveSpecialChars);
            }
            previous = current;
            idx += 1
        }
        Ok(chars.into_iter().map(|(_, y)| y).collect())
    }
    
    /// Returns `true` if char is zero-width or formatting (invisible)
    fn is_zero_width_or_formatting(c: char) -> bool {
        matches!(
            c,
            '\u{200B}'  // ZWSP
            | '\u{200C}' // ZWNJ
            | '\u{200D}' // ZWJ
            | '\u{2060}' // Word Joiner
            | '\u{FEFF}' // BOM / ZWNBSP
            | '\u{00AD}' // Soft Hyphen
            | '\u{034F}' // Combining Grapheme Joiner
            | '\u{180E}' // Mongolian Vowel Separator
            | '\u{202F}' // Narrow NBSP (but already trimmed)
            | '\u{2060}' // Word Joiner
            | '\u{2061}' // Function Application
            | '\u{2062}' // Invisible Times
            | '\u{2063}' // Invisible Separator
            | '\u{2064}' // Invisible Plus
            | '\u{2066}' // LRI
            | '\u{2067}' // RLI
            | '\u{2068}' // FSI
            | '\u{2069}' // PDI
            | '\u{202A}' // LRE
            | '\u{202B}' // RLE
            | '\u{202C}' // PDF
            | '\u{202D}' // LRO
            | '\u{202E}' // RLO
            | '\u{206A}' // SSCI
            | '\u{206B}' // ACSC
            | '\u{206C}' // ACSCB
            | '\u{206D}' // ATSC
            | '\u{206E}' // NDS
            | '\u{206F}' // CN
            | '\u{FFF0}' // Specials: Replacement Character
            | '\u{FFF1}' | '\u{FFF2}' | '\u{FFF3}' | '\u{FFF4}' // Specials
        )
    }

}

impl AsRef<str> for SubscriptionFilteredName {
    fn as_ref(&self) -> &str {
        &self
    }
}

// Antipattern Deref polymorphism to emulate inheritance. Read https://github.com/rust-unofficial/patterns/blob/main/anti_patterns/deref.md
impl Deref for SubscriptionFilteredName {
    type Target = String;
    fn deref(&self) -> &String {
        &self
    }
}

impl FromStr for SubscriptionFilteredName {
    type Err = NameError;

    fn from_str(s: &str) -> Result<Self, NameError> {
        Self::new(s)
    }
}

impl fmt::Display for SubscriptionFilteredName {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}

#[cfg(test)]
mod tests {
    use crate::subscription::SubscriptionFilteredName;
    use super::NameError;
    use arbtest::arbtest;
    use claims::{assert_err, assert_ok};
    use rand::{distributions::WeightedIndex, prelude::*};
    use std::str::FromStr;

    #[test]
    fn test_empty_input() {
        assert_eq!(SubscriptionFilteredName::new(""), Err(NameError::Empty));
    }

    #[test]
    fn test_trimmed_empty() {
        assert_eq!(SubscriptionFilteredName::new("   "), Err(NameError::Empty));
    }

    #[test]
    fn test_valid_name() {
        let name = SubscriptionFilteredName::new("Alice Smith").unwrap();
        assert_eq!(name.as_str(), "Alice Smith");
        assert_eq!(name.grapheme_len(), 11);
    }

    #[test]
    fn test_unicode_whitespace() {
        let name = SubscriptionFilteredName::new("\u{A0}Alice\u{A0}").unwrap(); // NBSP
        assert_eq!(name.as_str(), "Alice");
    }

    #[test]
    fn test_zero_width_filtering() {
        let name = SubscriptionFilteredName::new("A\u{200B}lice").unwrap();
        assert_eq!(name.as_str(), "Alice"); // ZWSP removed
    }

    #[test]
    fn test_punctuation_normalization() {
        let name = SubscriptionFilteredName::new("O’Connor").unwrap();
        assert_eq!(name.as_str(), "O'Connor");

        let name = SubscriptionFilteredName::new("Jean–Luc").unwrap();
        assert_eq!(name.as_str(), "Jean-Luc");

        let name = SubscriptionFilteredName::new("100 %").unwrap(); // narrow NBSP
        assert_eq!(name.as_str(), "100%");
    }

    #[test]
    fn test_fullwidth_normalization() {
        let name = SubscriptionFilteredName::new("Ａｌｉｃｅ").unwrap();
        assert_eq!(name.as_str(), "Alice");
    }

    #[test]
    fn test_consecutive_special_chars() {
        assert!(matches!(
            SubscriptionFilteredName::new("Alice..Bob"),
            Err(NameError::ConsecutiveSpecialChars)
        ));
        assert!(matches!(
            SubscriptionFilteredName::new("Alice--Bob"),
            Err(NameError::ConsecutiveSpecialChars)
        ));
        assert!(matches!(
            SubscriptionFilteredName::new("Alice__Bob"),
            Err(NameError::ConsecutiveSpecialChars)
        ));
        assert!(matches!(
            SubscriptionFilteredName::new("Alice__Bob"),
            Err(NameError::ConsecutiveSpecialChars)
        ));
    }

    // ✅ Allow spaces/dashes between specials (e.g., "O'Connor", "Jean-Luc")
    #[test]
    fn test_valid_special_chars() {
        let name = SubscriptionFilteredName::new("O'Connor").unwrap();
        assert_eq!(name.as_str(), "O'Connor");

        let name = SubscriptionFilteredName::new("Jean-Luc").unwrap();
        assert_eq!(name.as_str(), "Jean-Luc");

        let name = SubscriptionFilteredName::new("Jean-Luc O'Connor").unwrap();
        assert_eq!(name.as_str(), "Jean-Luc O'Connor");
    }

    #[test]
    fn test_from_str() {
        let name = SubscriptionFilteredName::from_str("Bob").unwrap();
        assert_eq!(name.as_str(), "Bob");
    }

    #[test]
    fn test_as_ref() {
        let name = SubscriptionFilteredName::new("Bob").unwrap();
        assert_eq!(name.as_ref(), "Bob");
    }

    #[test]
    fn test_ligature_expansion() {
        let name = SubscriptionFilteredName::new("ﬁle").unwrap(); // U+FB01
        assert_eq!(name.as_str(), "file");
    }

    #[test]
    fn test_max_grapheme_limit() {
        let long_name = "A".repeat(255);
        assert!(matches!(
            SubscriptionFilteredName::new(&long_name),
            Err(NameError::TooLongGraphemes)
        ));
    }

    #[test]
    fn test_invalid_char() {
        assert!(matches!(
            SubscriptionFilteredName::new("Alice\u{0007}"),
            Err(NameError::InvalidChar)
        ));
    }
    #[test]
    fn random_fuzz_name_nopanic() {
        arbtest(|u| {
            let fuzz = u.arbitrary().expect("");
            // dbg!(&fuzz); // cargo test random_fuzz_name_nopanic -- --nocapture
            let _ = SubscriptionFilteredName::new(fuzz);
            Ok(())
        })
        .budget_ms(1_250)
        .run();
    }

    #[test]
    fn rejects_255_characters_input() {
        let name = "n".repeat(255);
        assert_err!(SubscriptionFilteredName::new(&name));
    }

    #[test]
    fn accepts_254_characters_input() {
        let name = "y".repeat(254);
        assert_ok!(SubscriptionFilteredName::parse(&name));
    }

    #[test]
    // At least while it isn't mandated otherwise
    fn accepts_cancelled_celebrities() {
        let tests = vec![
            "J. K. Rowling",
            "Matt Damon",
            "Jimmy Donaldson",
            "Ye West",
            "Logan Paul",
            "boogie2988",
            "SSSniperWolf",
        ];
        for input in tests {
            assert_ok!(SubscriptionFilteredName::from_str(&input));
        }
    }

    #[test]
    fn accepts_special_characters() {
        let tests = vec![
            "O'Yeah",
            "Graham-Cumming ",
            "X Æ A-12 Musk",
            "Nsĩã́",
            "Horáčková",
            "Rômulo",
            "Yaʻªqōḇ",
            "Dr. Conopoima",
            "Gordon Freeman, MSc;MBA;PhD,PMP®",
        ];
        for input in tests {
            assert_ok!(SubscriptionFilteredName::new(&input));
        }
    }

    #[test]
    fn rejects_repeated_special_characters() {
        let tests = vec![
            "O''Nah",
            "Column--Delimiter",
            "Likely++AnError",
            "Missing titles, MSc;;PhD,®",
        ];
        for input in tests {
            assert_err!(SubscriptionFilteredName::from_str(&input));
        }
    }

    #[test]
    fn accepts_input_needing_trimming() {
        let tests = vec![
            "We are anonymous!\n",
            "\n \tWe know exactly who they are \t",
            "\nRyan Sees Through Copper\t \n",
        ];
        for input in tests {
            assert_ok!(SubscriptionFilteredName::new(&input));
        }
    }

    #[test]
    fn rejects_forbidden_characters() {
        let tests = vec![
            "<MyNameIsARustTypeAnnotation>\n",
            "MyName?ReturnsResultAutomatically//ButErrorVariant",
            "Rust[1]ndexLik{3}TheFirst(0)ne",
        ];
        for input in tests {
            assert_err!(SubscriptionFilteredName::parse(&input));
        }
    }

    #[test]
    fn accepts_intermediate_whitespace() {
        let tests = vec![
            "Jose   Felix \t \n \
                Ribas",
            "This \t    \n keyboard\t \
            jumps \t \t\n    around   a lot",
        ];
        for input in tests {
            assert_ok!(SubscriptionFilteredName::from_str(&input));
        }
    }

    #[test]
    fn accepts_longer_than_254_chars_by_trimming() {
        let name = "  \ty\n".repeat(127); // Intermediate trimming 1 space after each "y" brings it to 253
        assert_ok!(SubscriptionFilteredName::parse(&name));
    }

    #[test]
    fn rejects_longer_than_254_chars_after_trimming() {
        let name = "  \tn\n".repeat(128); // Intermediate trimming 1 space after each "y" brings it to 255
        assert_err!(SubscriptionFilteredName::from_str(&name));
    }

    #[test]
    fn rejects_empty_blank_whitespace() {
        let tests = vec!["", " \t", "\n\t \n"];
        let mut rng = thread_rng();
        let methods_weights = [("new", 1), ("parse", 1), ("from_str", 1)];
        let sampling_methods =
            WeightedIndex::new(methods_weights.iter().map(|weight| weight.1)).unwrap();
        let results: Vec<Result<SubscriptionFilteredName, NameError>> = tests
            .into_iter()
            .map(|input| {
                let method = methods_weights[sampling_methods.sample(&mut rng)].0;
                if method.eq("new") {
                    SubscriptionFilteredName::new(&input)
                } else if method.eq("from_str") {
                    SubscriptionFilteredName::from_str(&input)
                } else {
                    SubscriptionFilteredName::parse(&input)
                }
            })
            .collect();
        for result in results {
            assert_err!(result);
        }
    }
}
