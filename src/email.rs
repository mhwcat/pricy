use crate::config::EmailConfiguration;
use crate::error::PricyResult;

pub fn send_price_update_email_notification(
    url: &str,
    price_old: f32,
    price_new: f32,
    modified_at_str: &str,
    email_config: &Option<EmailConfiguration>,
) -> PricyResult<()> {
    use crate::error::PricyError;
    use lettre::transport::smtp::authentication::Credentials;
    use lettre::{Message, SmtpTransport, Transport};

    let email_config = email_config.as_ref().ok_or_else(|| PricyError {
        msg: "Missing email configuration".to_string(),
    })?;

    let creds = Credentials::new(
        email_config.smtp_username.clone(),
        email_config.smtp_password.clone(),
    );
    let mailer = SmtpTransport::relay(&email_config.smtp_host)?
        .credentials(creds)
        .build();

    for recipient in &email_config.recipients {
        let email = Message::builder()
            .from(email_config.sender.parse()?)
            .to(recipient.parse()?)
            .subject("Price update alert")
            .body(format!(
                "Price updated: {:.2} -> {:.2} for {} at {}",
                price_old, price_new, url, modified_at_str
            ))?;

        mailer.send(&email)?;

        println!("Sent email notification to {}", recipient);
    }

    Ok(())
}
