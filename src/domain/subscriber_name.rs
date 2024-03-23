use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    /// Returns an instance of [`SubscriberName`] if the input satisfies all our validation
    /// constraints on subscriber name
    ///
    /// # Panics
    ///
    /// Panics if the input string is either empty, longer than 256 graphemes or contains
    /// forbidden_chars chars
    pub fn parse(s: String) -> Result<Self, String> {
        let is_empty_or_whitespace = s.trim().is_empty();

        //NOTE: A grapheme is defined by the Unicode standard as a "user-perceived" character: an
        //example would be an Umlaut letter, but it is composed of two characters
        //`graphemes` returns an iterator over the graphemes in the input `s`. `true` specifies that we
        //want to use the extended grapheme definition set, the recommended one
        let is_too_long = s.graphemes(true).count() > 256;

        let forbidden_chars = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_chars = s.chars().any(|g| forbidden_chars.contains(&g));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_chars {
            Err(format!("{} is not a valid subscriber name. ", s))
        } else {
            Ok(Self(s))
        }
    }

    // pub fn inner_ref(&self) -> &str {
    //     &self.0
    // }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
