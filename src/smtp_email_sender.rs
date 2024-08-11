use crate::censoredstring::CensoredString;
use anyhow::Result;
use core::convert::TryInto;
use lettre::{message::MessageBuilder, transport::smtp, Message, SmtpTransport, Transport};
use std::str::FromStr;

#[tracing::instrument(name = "Generating email builder.")]
pub fn new_email_builder(
    name: Option<&str>,
    from: &str,
    reply_to: &str,
) -> Result<MessageBuilder, anyhow::Error> {
    let message_builder = Message::builder();
    Ok(TryInto::<MessageBuilder>::try_into(
        message_builder
            .from(get_mailbox(name, from).parse()?)
            .reply_to(reply_to.parse()?),
    )?)
}

#[tracing::instrument(name = "Generating SmtpTransport Pool.")]
pub fn new_smtp_relay_mailer(
    server: &str,
    creds: smtp::authentication::Credentials,
    port: Option<u16>,
) -> Result<SmtpTransport, anyhow::Error> {
    let mailer = if let Some(port_number) = port {
        SmtpTransport::relay(server)?.port(port_number)
    } else {
        SmtpTransport::relay(server)?
    };
    Ok(mailer.credentials(creds).build())
}

#[tracing::instrument(name = "Generating SMTP Credentials.")]
pub fn get_smtp_credentials(
    username: &str,
    password: &CensoredString,
) -> smtp::authentication::Credentials {
    smtp::authentication::Credentials::new(
        std::string::String::from_str(username).unwrap(),
        std::string::String::from_str(password).unwrap(),
    )
}

#[tracing::instrument(name = "Sending email.")]
pub fn send_email(
    to_name: Option<&str>,
    to_email: &str,
    message: MessageBuilder,
    smtp_mailer: &smtp::SmtpTransport,
    subject: &str,
    body: &str,
) -> Result<smtp::response::Response, anyhow::Error> {
    let email = message
        .to(get_mailbox(to_name, to_email).parse()?)
        .subject(subject)
        .body(body.to_string())?;
    Ok(TryInto::<smtp::response::Response>::try_into(
        smtp_mailer.send(&email)?,
    )?)
}

pub fn get_mailbox(name: Option<&str>, address: &str) -> String {
    let receiver = if let Some(inner_name) = name {
        format! {"{} ", inner_name}
    } else {
        "".to_owned()
    };
    format! {"{}<{}>",receiver,address}
}
