use crate::subscription::subscription_filtered_email::SubscriptionFilteredEmail;
use crate::subscription::subscription_filtered_name::SubscriptionFilteredName;

#[derive(serde::Deserialize)]
pub struct SubscriptionFormData {
    pub email: SubscriptionFilteredEmail,
    pub name: SubscriptionFilteredName,
}
