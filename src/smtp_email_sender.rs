use anyhow::Result;
use core::convert::TryInto;
use lettre::{
    message::MessageBuilder,
    transport::smtp::{
        authentication::Credentials,
        SmtpTransport,
        response
    },
    Message, Transport
};
use secrecy::{ExposeSecret, SecretString};
use std::{str::FromStr};

#[tracing::instrument(name = "Generating email builder.", skip(name, from, reply_to))]
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

#[tracing::instrument(name = "Generating SmtpTransport Pool.", skip(creds))]
pub fn new_smtp_relay_mailer(
    server: &str,
    creds: Credentials,
    port: Option<u16>,
) -> Result<SmtpTransport, anyhow::Error> {
    let port = port.unwrap_or(587);
    // Build transport based on port:
    // - Port 465 → implicit TLS (SMTPS)
    // - Port 587 or other → explicit STARTTLS (opportunistic)
    let transport = if port == 465 {
        SmtpTransport::relay(server)?
            .credentials(creds)
            .build()
    } else {
        SmtpTransport::starttls_relay(server)?
            .port(port)
            .credentials(creds)
            .build()
    };

    Ok(transport)
}


#[tracing::instrument(name = "Generating SMTP Credentials.")]
pub fn get_smtp_credentials(
    username: &str,
    password: &SecretString,
) -> Credentials {
    Credentials::new(
        std::string::String::from_str(username).unwrap(),
        std::string::String::from_str(password.expose_secret()).unwrap(),
    )
}

#[tracing::instrument(name = "Sending email.")]
pub fn send_email(
    to_name: Option<&str>,
    to_email: &str,
    message: MessageBuilder,
    smtp_mailer: SmtpTransport,
    subject: &str,
    body: &str,
) -> Result<response::Response, anyhow::Error> {
    let email = message
        .to(get_mailbox(to_name, to_email).parse()?)
        .subject(subject)
        .body(body.to_string())?;
    Ok(TryInto::<response::Response>::try_into(
        smtp_mailer.send(&email)?,
    )?)
}

pub fn get_mailbox(name: Option<&str>, address: &str) -> String {
    let receiver = if let Some(inner_name) = name {
        format! {"{inner_name} "}
    } else {
        "".to_owned()
    };
    format! {"{receiver}<{address}>"}
}

