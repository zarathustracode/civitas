//! Caller metadata recorded alongside a new session: the `User-Agent` and
//! the client IP address.
//!
//! The IP follows the rate limiter's trust rule: forwarded headers are read
//! only when `TRUST_PROXY` is set, because without a trusted proxy in front
//! the client controls them. Otherwise the socket peer address is used.

use std::convert::Infallible;
use std::net::{IpAddr, SocketAddr};

use async_trait::async_trait;
use axum::extract::{ConnectInfo, FromRequestParts};
use axum::http::request::Parts;
use axum::http::{header, HeaderMap};

use crate::state::AppState;

/// Longest `User-Agent` stored; longer values are truncated.
const MAX_USER_AGENT_LEN: usize = 512;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClientInfo {
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
}

#[async_trait]
impl FromRequestParts<AppState> for ClientInfo {
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let peer = parts
            .extensions
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ConnectInfo(addr)| addr.ip());
        Ok(Self::from_parts(
            &parts.headers,
            peer,
            state.config().rate_limit.trust_proxy,
        ))
    }
}

impl ClientInfo {
    fn from_parts(headers: &HeaderMap, peer: Option<IpAddr>, trust_proxy: bool) -> Self {
        let user_agent = headers
            .get(header::USER_AGENT)
            .and_then(|v| v.to_str().ok())
            .map(str::trim)
            .filter(|ua| !ua.is_empty())
            // `to_str` only admits visible ASCII, so any byte index is a
            // char boundary.
            .map(|ua| ua[..ua.len().min(MAX_USER_AGENT_LEN)].to_string());

        let forwarded = if trust_proxy {
            forwarded_ip(headers)
        } else {
            None
        };

        Self {
            user_agent,
            ip_address: forwarded.or(peer).map(|ip| ip.to_string()),
        }
    }
}

/// Same precedence as the rate limiter's `SmartIpKeyExtractor`:
/// `X-Forwarded-For`, then `X-Real-IP`, then `Forwarded`.
fn forwarded_ip(headers: &HeaderMap) -> Option<IpAddr> {
    let header_str = |name: &str| headers.get(name).and_then(|v| v.to_str().ok());

    header_str("x-forwarded-for")
        .and_then(|v| v.split(',').find_map(|node| parse_node(node.trim())))
        .or_else(|| header_str("x-real-ip").and_then(|v| parse_node(v.trim())))
        .or_else(|| header_str("forwarded").and_then(parse_forwarded_for))
}

/// Extract the first `for=` node from an RFC 7239 `Forwarded` header.
fn parse_forwarded_for(value: &str) -> Option<IpAddr> {
    value
        .split(',')
        .flat_map(|element| element.split(';'))
        .find_map(|pair| {
            let (key, node) = pair.split_once('=')?;
            if !key.trim().eq_ignore_ascii_case("for") {
                return None;
            }
            parse_node(node.trim().trim_matches('"'))
        })
}

/// Parse an IP that may carry a port: `192.0.2.1`, `192.0.2.1:4711`,
/// `2001:db8::1`, or `[2001:db8::1]:4711`.
fn parse_node(node: &str) -> Option<IpAddr> {
    if let Ok(ip) = node.parse() {
        return Some(ip);
    }
    if let Some(rest) = node.strip_prefix('[') {
        return rest.split_once(']')?.0.parse().ok();
    }
    node.parse::<SocketAddr>().ok().map(|addr| addr.ip())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn headers(pairs: &[(&'static str, &str)]) -> HeaderMap {
        let mut map = HeaderMap::new();
        for (name, value) in pairs {
            map.insert(*name, HeaderValue::from_str(value).unwrap());
        }
        map
    }

    const PEER: &str = "10.0.0.7";

    fn peer() -> Option<IpAddr> {
        PEER.parse().ok()
    }

    #[test]
    fn peer_address_is_used_without_proxy_trust() {
        let info = ClientInfo::from_parts(
            &headers(&[("x-forwarded-for", "203.0.113.9")]),
            peer(),
            false,
        );
        assert_eq!(info.ip_address.as_deref(), Some("10.0.0.7"));
    }

    #[test]
    fn forwarded_for_wins_when_proxy_is_trusted() {
        let info = ClientInfo::from_parts(
            &headers(&[("x-forwarded-for", "203.0.113.9, 10.0.0.1")]),
            peer(),
            true,
        );
        assert_eq!(info.ip_address.as_deref(), Some("203.0.113.9"));
    }

    #[test]
    fn unparseable_forwarded_values_fall_back_to_peer() {
        let info =
            ClientInfo::from_parts(&headers(&[("x-forwarded-for", "not-an-ip")]), peer(), true);
        assert_eq!(info.ip_address.as_deref(), Some("10.0.0.7"));
    }

    #[test]
    fn x_real_ip_and_rfc7239_forwarded_are_honoured() {
        let real = ClientInfo::from_parts(&headers(&[("x-real-ip", "198.51.100.4")]), None, true);
        assert_eq!(real.ip_address.as_deref(), Some("198.51.100.4"));

        let rfc = ClientInfo::from_parts(
            &headers(&[("forwarded", r#"proto=https;for="[2001:db8:cafe::17]:4711""#)]),
            None,
            true,
        );
        assert_eq!(rfc.ip_address.as_deref(), Some("2001:db8:cafe::17"));
    }

    #[test]
    fn missing_peer_and_headers_yield_no_ip() {
        let info = ClientInfo::from_parts(&HeaderMap::new(), None, true);
        assert_eq!(info, ClientInfo::default());
    }

    #[test]
    fn user_agent_is_trimmed_and_truncated() {
        let info = ClientInfo::from_parts(
            &headers(&[("user-agent", "  Mozilla/5.0 (X11)  ")]),
            None,
            false,
        );
        assert_eq!(info.user_agent.as_deref(), Some("Mozilla/5.0 (X11)"));

        let long = "a".repeat(MAX_USER_AGENT_LEN + 50);
        let info = ClientInfo::from_parts(&headers(&[("user-agent", &long)]), None, false);
        assert_eq!(info.user_agent.map(|ua| ua.len()), Some(MAX_USER_AGENT_LEN));
    }

    #[test]
    fn blank_user_agent_is_dropped() {
        let info = ClientInfo::from_parts(&headers(&[("user-agent", "   ")]), None, false);
        assert_eq!(info.user_agent, None);
    }
}
