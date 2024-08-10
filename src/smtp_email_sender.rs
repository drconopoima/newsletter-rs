use anyhow::Result;
use core::convert::TryInto;
use lettre::{
    message::MessageBuilder,
    transport::smtp::{authentication, SmtpTransportBuilder},
    Message, SmtpTransport,
};

pub fn new_email_builder(
    name: &str,
    from: &str,
    reply_to: &str,
) -> Result<MessageBuilder, anyhow::Error> {
    let message_builder = Message::builder();
    Ok(TryInto::<MessageBuilder>::try_into(
        message_builder
            .from(format! {"{} <{}>",name,from}.parse()?)
            .reply_to(reply_to.parse()?),
    )?)
}

pub fn new_smtp_relay_mailer(
    server: &str,
    creds: authentication::Credentials,
    port: Option<u16>,
) -> Result<SmtpTransportBuilder, anyhow::Error> {
    let mailer = if let Some(port_number) = port {
        SmtpTransport::relay(server)?.port(port_number)
    } else {
        SmtpTransport::relay(server)?
    };
    Ok(mailer.credentials(creds))
}

pub fn get_credentials(username: String, password: String) -> authentication::Credentials {
    authentication::Credentials::new(username, password)
}
