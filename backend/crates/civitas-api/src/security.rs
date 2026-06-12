//! Request hardening: browser-origin verification for state-changing
//! requests.
//!
//! Defense in depth against CSRF. The primary defenses already exist by
//! construction — `SameSite=Strict` session cookies and JSON-only request
//! bodies — this guard additionally rejects any POST/PUT/PATCH/DELETE whose
//! `Origin` header disagrees with `PUBLIC_BASE_URL`. Requests *without* an
//! `Origin` header pass: server-to-server calls (the `SvelteKit` backend) and
//! non-browser clients don't send one, and a browser cross-site attack
//! cannot remove it.

use axum::extract::{Request, State};
use axum::http::{header, Method};
use axum::middleware::Next;
use axum::response::Response;

use crate::error::ApiError;
use crate::state::AppState;

pub async fn verify_origin(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let safe_method = matches!(*req.method(), Method::GET | Method::HEAD | Method::OPTIONS);
    if !safe_method {
        if let Some(origin) = req.headers().get(header::ORIGIN) {
            let allowed = origin
                .to_str()
                .is_ok_and(|o| origin_allowed(&state.config().public_base_url, o));
            if !allowed {
                return Err(ApiError::Forbidden);
            }
        }
    }
    Ok(next.run(req).await)
}

/// Compare a browser `Origin` header against the configured public base
/// URL: scheme, host, and port must all match. Loopback spellings
/// (`localhost`, `127.0.0.1`, `[::1]`) are treated as equivalent so local
/// development works regardless of which one the browser used.
fn origin_allowed(public_base_url: &str, origin: &str) -> bool {
    let Some(expected) = split_origin(public_base_url) else {
        return false;
    };
    let Some(got) = split_origin(origin) else {
        return false;
    };

    expected.scheme.eq_ignore_ascii_case(got.scheme)
        && hosts_equivalent(expected.host, got.host)
        && expected.port == got.port
}

struct OriginParts<'a> {
    scheme: &'a str,
    host: &'a str,
    port: Option<&'a str>,
}

fn split_origin(url: &str) -> Option<OriginParts<'_>> {
    let url = url.trim_end_matches('/');
    let scheme_end = url.find("://")?;
    let scheme = &url[..scheme_end];
    let authority = &url[scheme_end + 3..];
    if authority.is_empty() || authority.contains('/') {
        return None;
    }
    // IPv6 literals carry brackets: [::1]:5173
    let (host, port) = if let Some(rest) = authority.strip_prefix('[') {
        let close = rest.find(']')?;
        let host = &authority[..close + 2];
        let port = rest[close + 1..].strip_prefix(':');
        (host, port)
    } else {
        match authority.rsplit_once(':') {
            Some((h, p)) => (h, Some(p)),
            None => (authority, None),
        }
    };
    Some(OriginParts { scheme, host, port })
}

fn hosts_equivalent(a: &str, b: &str) -> bool {
    if a.eq_ignore_ascii_case(b) {
        return true;
    }
    is_loopback(a) && is_loopback(b)
}

fn is_loopback(host: &str) -> bool {
    host.eq_ignore_ascii_case("localhost") || host == "127.0.0.1" || host == "[::1]"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match_passes() {
        assert!(origin_allowed(
            "https://civitas.example.org",
            "https://civitas.example.org"
        ));
    }

    #[test]
    fn trailing_slash_on_config_is_tolerated() {
        assert!(origin_allowed(
            "https://civitas.example.org/",
            "https://civitas.example.org"
        ));
    }

    #[test]
    fn foreign_origin_is_rejected() {
        assert!(!origin_allowed(
            "https://civitas.example.org",
            "https://evil.example.com"
        ));
    }

    #[test]
    fn null_origin_is_rejected() {
        // Sandboxed iframes and some redirects send the literal "null".
        assert!(!origin_allowed("https://civitas.example.org", "null"));
    }

    #[test]
    fn scheme_downgrade_is_rejected() {
        assert!(!origin_allowed(
            "https://civitas.example.org",
            "http://civitas.example.org"
        ));
    }

    #[test]
    fn port_mismatch_is_rejected() {
        assert!(!origin_allowed(
            "http://localhost:5173",
            "http://localhost:4173"
        ));
    }

    #[test]
    fn loopback_spellings_are_equivalent() {
        assert!(origin_allowed(
            "http://localhost:5173",
            "http://127.0.0.1:5173"
        ));
        assert!(origin_allowed("http://localhost:5173", "http://[::1]:5173"));
    }

    #[test]
    fn subdomain_is_not_the_same_origin() {
        assert!(!origin_allowed(
            "https://civitas.example.org",
            "https://api.civitas.example.org"
        ));
    }
}
