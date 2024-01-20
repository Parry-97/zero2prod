use core::panic;

use unicode_segmentation::UnicodeSegmentation;

pub struct NewSubscriber {
    pub email: String,
    pub name: SubscriberName,
}
pub struct SubscriberName(String);

impl SubscriberName {
    /// Returns an instance of [`SubscriberName`] if the input satisfies all our validation
    /// constraints on subscriber name
    ///
    /// # Panics
    ///
    /// Panics if the input string is either empty, longer than 256 graphemes or contains
    /// forbidden_chars chars
    pub fn parse(s: String) -> Self {
        let is_empty_or_whitespace = s.trim().is_empty();

        //NOTE: A grapheme is defined by the Unicode standard as a "user-perceived" character: an
        //example would be an Umlaut letter, but it is composed of two characters
        //`graphemes` returns an iterator over the graphemes in the input `s`. `true` specifies that we
        //want to use the extended grapheme definition set, the recommended one
        let is_too_long = s.graphemes(true).count() > 256;

        let forbidden_chars = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_chars = s.chars().any(|g| forbidden_chars.contains(&g));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_chars {
            panic!("{} is not a valid subscriber name. ", s)
        } else {
            Self(s)
        }
    }

    /// Returns a reference to the inner ref of this [`SubscriberName`]
    pub fn inner_ref(&self) -> &str {
        &self.0
    }
}
