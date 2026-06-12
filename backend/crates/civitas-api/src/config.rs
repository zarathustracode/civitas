//! Configuration loaded from the process environment.
//!
//! Loading is fail-fast: any missing required variable aborts startup with a
//! clear message, so a misconfigured deployment never silently runs with
//! defaults. Optional variables have documented defaults.

use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub database_max_connections: u32,
    pub http_listen_addr: SocketAddr,
    pub public_base_url: String,
    pub cookie: CookieConfig,
    /// When true, the register response includes the verification token
    /// so the dev frontend can pre-fill it. Never enable in production —
    /// it short-circuits the email-verification ceremony.
    pub dev_return_verification_token: bool,
    pub mail: MailConfig,
}

/// Outbound-mail driver. `Smtp` when `SMTP_HOST` is set; otherwise `Log`,
/// which writes mail to the server log so local development works without
/// an SMTP server.
#[derive(Debug, Clone)]
pub enum MailConfig {
    Smtp(SmtpConfig),
    Log,
}

#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    /// RFC 5322 mailbox, e.g. `Civitas <noreply@example.org>`.
    pub from: String,
    pub tls: MailTls,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MailTls {
    /// Plaintext SMTP. Only for local catchers like Mailpit.
    None,
    StartTls,
    Implicit,
}

#[derive(Debug, Clone)]
pub struct CookieConfig {
    pub domain: Option<String>,
    pub secure: bool,
    pub session_name: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing required environment variable: {0}")]
    Missing(&'static str),
    #[error("invalid value for {var}: {msg}")]
    Invalid { var: &'static str, msg: String },
}

impl Config {
    /// Load config from the environment. Required: `DATABASE_URL`,
    /// `PUBLIC_BASE_URL`. Everything else has a sensible default.
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url = required("DATABASE_URL")?;
        let public_base_url = required("PUBLIC_BASE_URL")?;

        let database_max_connections = optional_u32("DATABASE_MAX_CONNECTIONS", 20)?;
        let http_listen_addr = optional("HTTP_LISTEN_ADDR")
            .unwrap_or_else(|| "127.0.0.1:8080".to_string())
            .parse()
            .map_err(|e: std::net::AddrParseError| ConfigError::Invalid {
                var: "HTTP_LISTEN_ADDR",
                msg: e.to_string(),
            })?;

        let cookie = CookieConfig {
            domain: optional("COOKIE_DOMAIN"),
            secure: optional_bool("COOKIE_SECURE", true)?,
            session_name: optional("COOKIE_SESSION_NAME")
                .unwrap_or_else(|| "civitas_session".to_string()),
        };
        let dev_return_verification_token = optional_bool("DEV_RETURN_VERIFICATION_TOKEN", false)?;
        let mail = mail_from_env()?;

        Ok(Self {
            database_url,
            database_max_connections,
            http_listen_addr,
            public_base_url,
            cookie,
            dev_return_verification_token,
            mail,
        })
    }
}

fn mail_from_env() -> Result<MailConfig, ConfigError> {
    let Some(host) = optional("SMTP_HOST") else {
        return Ok(MailConfig::Log);
    };

    let port = optional_u16("SMTP_PORT", 587)?;
    let tls = match optional("SMTP_TLS") {
        Some(s) => match s.to_ascii_lowercase().as_str() {
            "none" => MailTls::None,
            "starttls" => MailTls::StartTls,
            "implicit" => MailTls::Implicit,
            other => {
                return Err(ConfigError::Invalid {
                    var: "SMTP_TLS",
                    msg: format!("expected none|starttls|implicit, got {other:?}"),
                })
            }
        },
        // Secure by default: implicit TLS on the SMTPS port, STARTTLS
        // everywhere else. Plaintext must be requested explicitly.
        None => {
            if port == 465 {
                MailTls::Implicit
            } else {
                MailTls::StartTls
            }
        }
    };

    Ok(MailConfig::Smtp(SmtpConfig {
        host,
        port,
        username: optional("SMTP_USER"),
        password: optional("SMTP_PASS"),
        from: required("SMTP_FROM")?,
        tls,
    }))
}

fn required(var: &'static str) -> Result<String, ConfigError> {
    std::env::var(var).map_err(|_| ConfigError::Missing(var))
}

fn optional(var: &str) -> Option<String> {
    std::env::var(var).ok().filter(|s| !s.is_empty())
}

fn optional_u16(var: &'static str, default: u16) -> Result<u16, ConfigError> {
    match optional(var) {
        Some(s) => s
            .parse()
            .map_err(|e: std::num::ParseIntError| ConfigError::Invalid {
                var,
                msg: e.to_string(),
            }),
        None => Ok(default),
    }
}

fn optional_u32(var: &'static str, default: u32) -> Result<u32, ConfigError> {
    match optional(var) {
        Some(s) => s
            .parse()
            .map_err(|e: std::num::ParseIntError| ConfigError::Invalid {
                var,
                msg: e.to_string(),
            }),
        None => Ok(default),
    }
}

fn optional_bool(var: &'static str, default: bool) -> Result<bool, ConfigError> {
    match optional(var) {
        Some(s) => match s.to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Ok(true),
            "false" | "0" | "no" | "off" => Ok(false),
            other => Err(ConfigError::Invalid {
                var,
                msg: format!("expected boolean, got {other:?}"),
            }),
        },
        None => Ok(default),
    }
}
