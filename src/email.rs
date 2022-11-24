use lettre::message::{header, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use crate::config::{EmailConfiguration, ProductConfiguration};
use crate::error::PricyError;
use crate::error::PricyResult;

pub fn send_price_update_email_notification(
    prod_conf: &ProductConfiguration,
    name: &str,
    price_old: f32,
    price_new: f32,
    modified_at_str: &str,
    email_config: &Option<EmailConfiguration>,
) -> PricyResult<()> {
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

    let recipients = if let Some(prod_recipients) = &prod_conf.notification_email_recipients {
        prod_recipients
    } else {
        &email_config.recipients
    };

    for recipient in recipients {
        let email = Message::builder()
            .from(email_config.sender.parse()?)
            .to(recipient.parse()?)
            .subject(format!("Price update alert ({})", name))
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_PLAIN)
                            .body(format!(
                                "Price updated: {:.2} -> {:.2} for {} at {}",
                                price_old, price_new, prod_conf.url, modified_at_str
                            )),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_HTML)
                            .body(format_email_html(
                                &prod_conf.url,
                                price_old,
                                price_new,
                                modified_at_str,
                            )),
                    ),
            )?;

        mailer.send(&email)?;

        println!("Sent email notification to {}", recipient);
    }

    Ok(())
}

#[inline]
fn format_email_html(url: &str, price_old: f32, price_new: f32, modified_at_str: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Price update alert</title>
            </head>
            <body>
                <strong>Price update alert!</strong>
                <div style="margin-top: 10px;">
                    <p>{}</p>
                    <p>{:.2} &rarr; {:.2}</p>
                    <br>
                    <p>Checked at {}</p>
                </div>
            </body>
        </html>"#,
        url, price_old, price_new, modified_at_str
    )
}
