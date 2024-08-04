use lettre::transport::smtp::{response,authentication,Error};
use lettre::{Message, SmtpTransport, Transport};

pub fn send_email() -> Result<response::Response, Error> {
    let email = Message::builder()
    .from(
        "newsletter-rs <postmaster@email.tld>"
            .parse()
            .unwrap(),
    )
    .reply_to("no-reply@example.com".parse().unwrap())
    .to("example <email@example.com>".parse().unwrap())
    .subject("Rust Email")
    .body(String::from("Hello, this is a test email from Rust!"))
    .unwrap();
    let creds = authentication::Credentials::new(
        "ausernamefromansmtp".to_string(),
        "apasswordfromansmtp".to_string(),
    );
    let mailer = SmtpTransport::relay("ansmtp.server.tld")?
    .credentials(creds)
    .build();
    mailer.send(&email)
}