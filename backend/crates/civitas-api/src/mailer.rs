//! Outbound email.
//!
//! `VerificationProvider::initiate` returns a plaintext token and leaves
//! transmission to the caller — this module is that caller's tool. Two
//! [`Mailer`] implementations exist: [`SmtpMailer`] for real deployments
//! and [`LogMailer`], which writes mail to the server log so local
//! development works without an SMTP server (pair with Mailpit via
//! `make mail-up` for a real inbox).
//!
//! Every message is `multipart/alternative`: a plain-text part that reads
//! well on its own, and an HTML part rendered from the same [`Template`]
//! so the two never drift apart.

use std::sync::Arc;

use async_trait::async_trait;
use lettre::message::{Mailbox, MultiPart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use crate::config::{MailConfig, MailTls, SmtpConfig};

/// A fully rendered outbound message.
#[derive(Debug, Clone)]
pub struct Mail {
    pub to: String,
    pub subject: String,
    /// The `text/plain` part.
    pub text: String,
    /// The `text/html` part: the same content, laid out for mail clients.
    pub html: String,
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
            .multipart(MultiPart::alternative_plain_html(mail.text, mail.html))?;
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
            body = %mail.text,
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
    Template {
        subject: "Verify your email — Civitas",
        heading: "Confirm your email address",
        intro: "Welcome to Civitas. Confirm this address to start voting, \
                delegating, and taking part in deliberation.",
        action: "Verify email",
        link: &link(public_base_url, "/auth/verify-email", token),
        validity: "The link is valid for 24 hours.",
        not_you: "If you did not create this account, ignore this message and \
                  the account will remain unverified.",
    }
    .render(to)
}

/// Password-reset message.
#[must_use]
pub fn password_reset_mail(public_base_url: &str, to: &str, token: &str) -> Mail {
    Template {
        subject: "Reset your password — Civitas",
        heading: "Choose a new password",
        intro: "A password reset was requested for this address. Choosing a \
                new password signs you out everywhere else.",
        action: "Reset password",
        link: &link(public_base_url, "/auth/reset-password", token),
        validity: "The link is valid for 1 hour.",
        not_you: "If you did not request this, ignore this message; your \
                  password is unchanged.",
    }
    .render(to)
}

/// Passwordless sign-in message. The link lands on a confirmation page;
/// the token is spent only when the reader presses the button there, so
/// mail scanners that prefetch links cannot use it up.
#[must_use]
pub fn login_link_mail(public_base_url: &str, to: &str, token: &str) -> Mail {
    Template {
        subject: "Your sign-in link — Civitas",
        heading: "Sign in to Civitas",
        intro: "Use this link to sign in without your password. It works once.",
        action: "Sign in",
        link: &link(public_base_url, "/auth/login-link/confirm", token),
        validity: "The link is valid for 15 minutes, and requesting another \
                   one retires it.",
        not_you: "If you did not ask to sign in, ignore this message. Nobody \
                  can use the link without access to this inbox.",
    }
    .render(to)
}

/// Token plaintexts are base64url (`civitas_auth::tokens`), so they embed
/// in a query string without escaping.
fn link(public_base_url: &str, path: &str, token: &str) -> String {
    let base = public_base_url.trim_end_matches('/');
    format!("{base}{path}?token={token}")
}

/// The one layout every transactional message shares: a heading, a short
/// explanation, a single call to action, how long it lasts, and what to do
/// if the reader did not ask for it.
struct Template<'a> {
    subject: &'a str,
    heading: &'a str,
    intro: &'a str,
    action: &'a str,
    link: &'a str,
    validity: &'a str,
    not_you: &'a str,
}

impl Template<'_> {
    fn render(&self, to: &str) -> Mail {
        Mail {
            to: to.to_string(),
            subject: self.subject.to_string(),
            text: self.text(),
            html: self.html(),
        }
    }

    fn text(&self) -> String {
        let Self {
            heading,
            intro,
            action,
            link,
            validity,
            not_you,
            ..
        } = self;
        format!(
            "{heading}\n\n\
             {intro}\n\n\
             {action}:\n{link}\n\n\
             {validity}\n\n\
             {not_you}\n\n\
             — Civitas\n"
        )
    }

    /// Table layout with inline styles: the subset every mail client
    /// renders. Colours follow the web app's paper, ink, and accent.
    fn html(&self) -> String {
        let subject = escape_html(self.subject);
        let heading = escape_html(self.heading);
        let intro = escape_html(self.intro);
        let action = escape_html(self.action);
        let link = escape_html(self.link);
        let validity = escape_html(self.validity);
        let not_you = escape_html(self.not_you);
        format!(
            r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="color-scheme" content="light">
<title>{subject}</title>
</head>
<body style="margin:0;padding:0;background:#f4f1ea;color:#1b1a14;">
<table role="presentation" width="100%" cellpadding="0" cellspacing="0" style="background:#f4f1ea;">
<tr><td align="center" style="padding:32px 16px;">
<table role="presentation" width="100%" cellpadding="0" cellspacing="0" style="max-width:520px;background:#fbfaf6;border:1px solid #dcd8cd;border-radius:6px;">
<tr><td style="padding:32px 32px 8px;font-family:Georgia,'Times New Roman',serif;font-size:13px;letter-spacing:0.12em;text-transform:uppercase;color:#908d80;">Civitas</td></tr>
<tr><td style="padding:0 32px;font-family:Georgia,'Times New Roman',serif;font-size:26px;line-height:1.2;font-weight:bold;color:#1b1a14;">{heading}</td></tr>
<tr><td style="padding:16px 32px 0;font-family:Arial,Helvetica,sans-serif;font-size:16px;line-height:1.55;color:#2c2a22;">{intro}</td></tr>
<tr><td style="padding:24px 32px 0;">
<a href="{link}" style="display:inline-block;background:#2b3a8c;color:#ffffff;font-family:Arial,Helvetica,sans-serif;font-size:16px;font-weight:bold;text-decoration:none;padding:12px 22px;border-radius:999px;">{action}</a>
</td></tr>
<tr><td style="padding:20px 32px 0;font-family:Arial,Helvetica,sans-serif;font-size:14px;line-height:1.5;color:#56544a;">{validity}</td></tr>
<tr><td style="padding:12px 32px 24px;font-family:Arial,Helvetica,sans-serif;font-size:13px;line-height:1.5;color:#56544a;">If the button does not work, copy this address into your browser:<br><a href="{link}" style="color:#2b3a8c;word-break:break-all;">{link}</a></td></tr>
<tr><td style="padding:24px 32px 32px;font-family:Arial,Helvetica,sans-serif;font-size:13px;line-height:1.5;color:#908d80;border-top:1px solid #dcd8cd;">{not_you}</td></tr>
</table>
</td></tr>
</table>
</body>
</html>
"#
        )
    }
}

fn escape_html(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for c in raw.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verification_link_embeds_token_and_targets_verify_page() {
        let mail = verification_mail("https://example.org", "a@b.c", "tok123");
        assert_eq!(mail.to, "a@b.c");
        let link = "https://example.org/auth/verify-email?token=tok123";
        assert!(mail.text.contains(link));
        assert!(mail.html.contains(&format!("href=\"{link}\"")));
    }

    #[test]
    fn reset_link_targets_reset_page() {
        let mail = password_reset_mail("https://example.org", "a@b.c", "tok456");
        assert!(mail
            .text
            .contains("https://example.org/auth/reset-password?token=tok456"));
    }

    #[test]
    fn login_link_targets_the_confirmation_page() {
        let mail = login_link_mail("https://example.org", "a@b.c", "tok789");
        let link = "https://example.org/auth/login-link/confirm?token=tok789";
        assert!(mail.text.contains(link));
        assert!(mail.html.contains(link));
        assert!(mail.text.contains("15 minutes"));
    }

    #[test]
    fn trailing_slash_on_base_url_does_not_double_up() {
        let mail = verification_mail("https://example.org/", "a@b.c", "t");
        assert!(mail
            .text
            .contains("https://example.org/auth/verify-email?token=t"));
        assert!(!mail.text.contains("org//auth"));
    }

    #[test]
    fn text_and_html_parts_carry_the_same_content() {
        let mail = password_reset_mail("https://example.org", "a@b.c", "t");
        for fragment in [
            "Choose a new password",
            "The link is valid for 1 hour.",
            "your password is unchanged",
        ] {
            assert!(mail.text.contains(fragment), "text lacks {fragment:?}");
            assert!(mail.html.contains(fragment), "html lacks {fragment:?}");
        }
        assert!(mail.html.starts_with("<!doctype html>"));
    }

    #[test]
    fn html_escapes_interpolated_values() {
        let mail = verification_mail("https://example.org/\"><script>", "a@b.c", "t");
        assert!(!mail.html.contains("<script>"));
        assert!(mail.html.contains("&quot;&gt;&lt;script&gt;"));
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
