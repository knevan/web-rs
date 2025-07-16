use crate::builder::startup::Mailer;
use anyhow::Context;
use lettre::message::Mailbox;
use lettre::{AsyncTransport, Message};
use std::env;

// Function to send the password reset email
pub async fn send_password_reset_email(
    mailer: &Mailer,
    recipient_email: &str,
    recipient_username: &str,
    token: &str,
) -> anyhow::Result<()> {
    let frontend_url =
        env::var("FRONTEND_URL").expect("FRONTEND_URL must be set");
    let app_email = env::var("SMTP_USERNAME").expect("APP_EMAIL must be set");

    // Construct the password reset link
    let reset_link = format!("{}/reset-password?token={}", frontend_url, token);

    // Build email structure
    let from_mailbox: Mailbox = format!("Your App Name <{}>", app_email)
        .parse()
        .context("Failed to parse sender email")?;
    let to_mailbox: Mailbox = recipient_email
        .parse()
        .context("Failed to parse recipient email")?;

    let email = Message::builder()
        .from(from_mailbox)
        .to(to_mailbox)
        .subject("Password Reset Request")
        .body(format!(
            "Hello {}, \n\nYou requested a password reset for your account. Click the following link below:\n{}\n\nIf you did not request this, please ignore this email.",
            recipient_username,
            reset_link
        ))
        .context("Failed to build email message")?;

    // Send the email to the recipient
    mailer.send(email).await.context("Failed to send email")?;

    println!(
        "[EMAIL] Password to reset email sent to {}",
        recipient_email
    );

    Ok(())
}
