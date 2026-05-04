//! Helpers for the session cookie.
//!
//! All session cookies are `HttpOnly`, `SameSite=Strict`, and (in production)
//! `Secure`. The value is the opaque session token; the database stores only
//! its hash.

use axum_extra::extract::cookie::{Cookie, SameSite};

use crate::config::CookieConfig;

/// Build a fresh session cookie for `value`.
#[must_use]
pub fn session_cookie(cfg: &CookieConfig, value: String, max_age_days: i64) -> Cookie<'static> {
    let mut cookie = Cookie::new(cfg.session_name.clone(), value);
    cookie.set_http_only(true);
    cookie.set_secure(cfg.secure);
    cookie.set_same_site(SameSite::Strict);
    cookie.set_path("/");
    if let Some(d) = &cfg.domain {
        cookie.set_domain(d.clone());
    }
    cookie.set_max_age(time::Duration::days(max_age_days));
    cookie
}

/// Build a cookie that, when set, expires the session immediately.
#[must_use]
pub fn clear_session_cookie(cfg: &CookieConfig) -> Cookie<'static> {
    let mut cookie = Cookie::new(cfg.session_name.clone(), String::new());
    cookie.set_http_only(true);
    cookie.set_secure(cfg.secure);
    cookie.set_same_site(SameSite::Strict);
    cookie.set_path("/");
    if let Some(d) = &cfg.domain {
        cookie.set_domain(d.clone());
    }
    cookie.set_max_age(time::Duration::seconds(0));
    cookie
}
