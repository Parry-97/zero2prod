use validator::validate_email;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<SubscriberEmail, String> {
        if validate_email(&s) {
            Ok(SubscriberEmail(s))
        } else {
            Err("Invalid subscriber email".to_string())
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use claim::{assert_err, assert_ok};
    use fake::{faker::internet::en::SafeEmail, Fake};
    use quickcheck::Arbitrary;

    use super::*;

    #[derive(Clone, Debug)]
    struct ValidEmailFixture(pub String);

    impl Arbitrary for ValidEmailFixture {
        fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
            let email = SafeEmail().fake_with_rng(g);
            Self(email)
        }
    }

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!(SubscriberEmail::parse(email), "Email is too short");
    }

    #[test]
    fn email_missing_at_symbol() {
        let email = "somethingdomain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn valid_email_is_parsed_succesfully() {
        //NOTE: Every time we run our test suite we will get a different email to test our parsing
        //logic. This a major improvement over the previous test where we had to hardcode the email
        let email = SafeEmail().fake();
        assert_ok!(SubscriberEmail::parse(email));
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_succesfully(valid: ValidEmailFixture) -> bool {
        SubscriberEmail::parse(valid.0).is_ok()
    }
}
