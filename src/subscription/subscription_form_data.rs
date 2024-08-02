use crate::subscription::subscription_filtered_email::SubscriptionFilteredEmail;
use crate::subscription::subscription_filtered_name::SubscriptionFilteredName;
use std::convert::TryFrom;

pub struct SubscriptionFormData {
    pub email: SubscriptionFilteredEmail,
    pub name: SubscriptionFilteredName,
}

impl TryFrom<FormData> for SubscriptionFormData {
    type Error = String;

    fn try_from(form: FormData) -> Result<Self, Self::Error> {
        let name = SubscriptionFilteredName::new(&form.name)?;
        let email = SubscriptionFilteredEmail::new(&form.email)?;
        Ok(Self { name, email })
    }
}

#[derive(serde::Deserialize)]
pub struct FormData {
    pub email: String,
    pub name: String,
}
