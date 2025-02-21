// src/utils/email.rs
use std::env;
use lettre::message::Message;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{SmtpTransport, Transport};
use crate::utils::error::AppError;
use dotenvy::dotenv;
use chrono::NaiveDateTime;

pub async fn send_password_reset_email(
    to_email: &str,
    reset_token: &str,
    token_expiration: NaiveDateTime
) -> Result<(), AppError> {
    dotenv().ok();  // Load .env file
    
    let smtp_username = env::var("SMTP_USERNAME")
        .map_err(|_| AppError::InternalServerError("SMTP username not configured".to_string()))?;
    let smtp_password = env::var("SMTP_PASSWORD")
        .map_err(|_| AppError::InternalServerError("SMTP password not configured".to_string()))?;
    let smtp_host = env::var("SMTP_HOST")
        .map_err(|_| AppError::InternalServerError("SMTP host not configured".to_string()))?;
    let frontend_url = env::var("FRONTEND_URL")
        .map_err(|_| AppError::InternalServerError("Frontend URL not configured".to_string()))?;

    let email = Message::builder()
        .from(smtp_username.parse().unwrap())
        .to(to_email.parse().unwrap())
        .subject("Password Reset Request")
        .body(format!(
            "Click the following link to reset your password:\n{}/reset-password/{} \n\nThis link will expire on {}.",
            frontend_url,
            reset_token,
            token_expiration.format("%Y-%m-%d %H:%M:%S")
        ))
        .unwrap();

    let creds = Credentials::new(smtp_username, smtp_password);

    let mailer = SmtpTransport::relay(&smtp_host).unwrap()
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => return Err(AppError::InternalServerError(format!("Could not send email: {}", e))),
    }
    Ok(())
}