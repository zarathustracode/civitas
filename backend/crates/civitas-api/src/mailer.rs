//! Outbound email.
//!
//! `VerificationProvider::initiate` returns a plaintext token and leaves
//! transmission to the caller — this module is that caller's tool. Two
//! [`Mailer`] implementations exist: [`SmtpMailer`] for real deployments
//! and [`LogMailer`], which writes mail to the server log so local
//! development works without an SMTP server (pair with Mailpit via
//! `make mail-up` for a real inbox).
//!
//! Messages are plain text by design; HTML templating is roadmap v0.2.

use std::sync::Arc;

use async_trait::async_trait;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use crate::config::{MailConfig, MailTls, SmtpConfig};

/// A fully rendered outbound message. Plain text only.
#[derive(Debug, Clone)]
pub struct Mail {
    pub to: String,
    pub subject: String,
    pub body: String,
}

#[derive(Debug, thiserror::Error)]
pub enum MailerError {
    #[error("invalid mail address: {0}")]
    Address(#[from] lettre::address::AddressError),
    #[error("building message: {0}")]
    Build(#[from] lettre::error::Error),
    #[error("smtp: {0}")]
    Smtp(#[from] lettre::transport::smtp::Error),
}

/// Pluggable mail transport. Object-safe so [`crate::AppState`] can hold
/// whichever driver the config selected.
#[async_trait]
pub trait Mailer: Send + Sync {
    async fn send(&self, mail: Mail) -> Result<(), MailerError>;
}

/// Build the mailer the config asks for. Fails fast on an unparseable
/// `SMTP_FROM` or host so a misconfigured deployment never boots.
pub fn build_mailer(config: &MailConfig) -> Result<Arc<dyn Mailer>, MailerError> {
    match config {
        MailConfig::Smtp(smtp) => Ok(Arc::new(SmtpMailer::new(smtp)?)),
        MailConfig::Log => {
            tracing::warn!(
                "SMTP_HOST not set — outbound mail will be written to the log, not delivered"
            );
            Ok(Arc::new(LogMailer))
        }
    }
}

/// Fire-and-forget delivery. Handlers must not fail or block on SMTP: the
/// state change (user row, token row) is already committed, and every mail
/// can be re-requested (resend-verification, password-reset request).
pub fn send_in_background(mailer: Arc<dyn Mailer>, mail: Mail) {
    tokio::spawn(async move {
        let subject = mail.subject.clone();
        if let Err(error) = mailer.send(mail).await {
            // Recipient address deliberately omitted: PII stays out of logs.
            tracing::error!(%error, %subject, "failed to send mail");
        }
    });
}

/// Production driver: SMTP via `lettre`, pooled connections.
pub struct SmtpMailer {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: Mailbox,
}

impl SmtpMailer {
    pub fn new(config: &SmtpConfig) -> Result<Self, MailerError> {
        let from: Mailbox = config.from.parse()?;

        let mut builder = match config.tls {
            MailTls::Implicit => AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)?,
            MailTls::StartTls => {
                AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host)?
            }
            MailTls::None => AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.host),
        }
        .port(config.port);

        if let (Some(user), Some(pass)) = (&config.username, &config.password) {
            builder = builder.credentials(Credentials::new(user.clone(), pass.clone()));
        }

        Ok(Self {
            transport: builder.build(),
            from,
        })
    }
}

#[async_trait]
impl Mailer for SmtpMailer {
    async fn send(&self, mail: Mail) -> Result<(), MailerError> {
        let message = Message::builder()
            .from(self.from.clone())
            .to(mail.to.parse()?)
            .subject(mail.subject)
            .body(mail.body)?;
        self.transport.send(message).await?;
        Ok(())
    }
}

/// Dev fallback: write the whole mail to the log. The verification and
/// reset links land in console output, which is where local development
/// copied tokens from before SMTP existed.
pub struct LogMailer;

#[async_trait]
impl Mailer for LogMailer {
    async fn send(&self, mail: Mail) -> Result<(), MailerError> {
        tracing::info!(
            to = %mail.to,
            subject = %mail.subject,
            body = %mail.body,
            "outbound mail (log driver — not delivered)"
        );
        Ok(())
    }
}

// ── messages ───────────────────────────────────────────────────────────────

/// Email-verification message. The link targets the frontend page that
/// pre-fills the token from the query string.
#[must_use]
pub fn verification_mail(public_base_url: &str, to: &str, token: &str) -> Mail {
    let link = link(public_base_url, "/auth/verify-email", token);
    Mail {
        to: to.to_string(),
        subject: "Verify your email — Civitas".to_string(),
        body: format!(
            "Welcome to Civitas.\n\n\
             Confirm this email address by opening the link below. \
             The link is valid for 24 hours.\n\n\
             {link}\n\n\
             If you did not create this account, ignore this message and \
             the account will remain unverified.\n"
        ),
    }
}

/// Password-reset message.
#[must_use]
pub fn password_reset_mail(public_base_url: &str, to: &str, token: &str) -> Mail {
    let link = link(public_base_url, "/auth/reset-password", token);
    Mail {
        to: to.to_string(),
        subject: "Reset your password — Civitas".to_string(),
        body: format!(
            "A password reset was requested for this address.\n\n\
             Open the link below to choose a new password. \
             The link is valid for 1 hour.\n\n\
             {link}\n\n\
             If you did not request this, ignore this message; your \
             password is unchanged.\n"
        ),
    }
}

/// Token plaintexts are base64url (`civitas_auth::tokens`), so they embed
/// in a query string without escaping.
fn link(public_base_url: &str, path: &str, token: &str) -> String {
    let base = public_base_url.trim_end_matches('/');
    format!("{base}{path}?token={token}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verification_link_embeds_token_and_targets_verify_page() {
        let mail = verification_mail("https://example.org", "a@b.c", "tok123");
        assert_eq!(mail.to, "a@b.c");
        assert!(mail
            .body
            .contains("https://example.org/auth/verify-email?token=tok123"));
    }

    #[test]
    fn reset_link_targets_reset_page() {
        let mail = password_reset_mail("https://example.org", "a@b.c", "tok456");
        assert!(mail
            .body
            .contains("https://example.org/auth/reset-password?token=tok456"));
    }

    #[test]
    fn trailing_slash_on_base_url_does_not_double_up() {
        let mail = verification_mail("https://example.org/", "a@b.c", "t");
        assert!(mail
            .body
            .contains("https://example.org/auth/verify-email?token=t"));
        assert!(!mail.body.contains("org//auth"));
    }

    #[test]
    fn smtp_mailer_rejects_unparseable_from() {
        let config = SmtpConfig {
            host: "localhost".to_string(),
            port: 1025,
            username: None,
            password: None,
            from: "not an address".to_string(),
            tls: MailTls::None,
        };
        assert!(SmtpMailer::new(&config).is_err());
    }

    // The pooled transport's Drop needs a tokio reactor, hence the
    // async test for the constructing (success) case.
    #[tokio::test]
    async fn smtp_mailer_accepts_display_name_mailbox() {
        let config = SmtpConfig {
            host: "localhost".to_string(),
            port: 1025,
            username: None,
            password: None,
            from: "Civitas <noreply@example.org>".to_string(),
            tls: MailTls::None,
        };
        assert!(SmtpMailer::new(&config).is_ok());
    }
}
