//! Browser, PWA, and API security hardening (`I09`, `22-pwa-api-and-abuse-security.md`,
//! `23-files-exports-and-integrations.md`).
//!
//! Enforces:
//! - Mandatory HTTP response security headers (`Cache-Control: no-store`,
//!   `Content-Security-Policy: default-src 'none'; frame-ancestors 'none'`,
//!   `X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`).
//! - Redaction of sensitive NFC tap query parameters (`d`, `c`, `pin`, `token`) before logging
//!   or tracing so single-use proofs never leak into observability sinks.
//! - Outbound destination SSRF validation blocking non-HTTPS schemes, loopback, RFC1918
//!   private ranges, link-local/cloud metadata (`169.254.169.254`), and unapproved hosts.
//! - Hospital-aware composite rate-limit keys that avoid treating a shared hospital NAT IP
//!   as a single individual identity.

use axum::{
    extract::Request,
    http::{HeaderValue, header},
    middleware::Next,
    response::Response,
};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use thiserror::Error;

/// Typed errors for browser/API hardening and outbound destination validation.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum HardeningError {
    #[error("outbound URL must use https:// scheme")]
    InsecureScheme,
    #[error("outbound URL is missing a valid host")]
    MissingHost,
    #[error("outbound IP literal or private/loopback/metadata address is forbidden")]
    ForbiddenAddressRange,
    #[error("outbound host is not in the approved partner allowlist")]
    HostNotAllowlisted,
    #[error("rate limit subject and scope must not be empty")]
    EmptyRateLimitDimension,
}

/// Axum middleware that attaches strict anti-caching, anti-framing, and content-type
/// hardening headers to every HTTP response (`22-pwa-api-and-abuse-security.md`).
pub async fn security_headers_middleware(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, no-cache, must-revalidate"),
    );
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'none'; frame-ancestors 'none'"),
    );
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    response
}

/// Redacts sensitive query parameters (`d`, `c`, `pin`, `token`, `code`) from URLs before
/// they are recorded in logs, metrics, or error traces (`22-pwa-api-and-abuse-security.md`).
pub fn redact_sensitive_url_params(raw_url: &str) -> String {
    let Some((base, query_and_fragment)) = raw_url.split_once('?') else {
        return raw_url.to_string();
    };
    let (query, fragment_opt) = match query_and_fragment.split_once('#') {
        Some((q, frag)) => (q, Some(frag)),
        None => (query_and_fragment, None),
    };

    let redacted_pairs: Vec<String> = query
        .split('&')
        .map(|pair| {
            if let Some((key, _value)) = pair.split_once('=') {
                let normalized = key.trim().to_ascii_lowercase();
                if matches!(
                    normalized.as_str(),
                    "d" | "c" | "pin" | "token" | "code" | "secret"
                ) {
                    format!("{key}=[REDACTED]")
                } else {
                    pair.to_string()
                }
            } else {
                pair.to_string()
            }
        })
        .collect();

    match fragment_opt {
        Some(_frag) => format!("{base}?{}#[REDACTED]", redacted_pairs.join("&")),
        None => format!("{base}?{}", redacted_pairs.join("&")),
    }
}

/// Validates an outbound partner or webhook URL against SSRF and allowlist rules (`22-pwa-api-and-abuse-security.md`, `23-files-exports-and-integrations.md`).
pub fn validate_outbound_https_destination(
    url: &str,
    approved_hosts: &[&str],
) -> Result<String, HardeningError> {
    let trimmed = url.trim();
    let rest = trimmed
        .strip_prefix("https://")
        .ok_or(HardeningError::InsecureScheme)?;

    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    if authority.is_empty() || authority.contains('@') {
        return Err(HardeningError::MissingHost);
    }

    let host = if authority.starts_with('[') {
        // IPv6 literal in brackets
        return Err(HardeningError::ForbiddenAddressRange);
    } else {
        authority.split(':').next().unwrap_or("")
    };

    let normalized_host = host.trim().to_ascii_lowercase();
    if normalized_host.is_empty() {
        return Err(HardeningError::MissingHost);
    }

    if normalized_host == "localhost"
        || normalized_host.ends_with(".local")
        || normalized_host.ends_with(".internal")
    {
        return Err(HardeningError::ForbiddenAddressRange);
    }

    if let Ok(ip) = normalized_host.parse::<IpAddr>() {
        if is_forbidden_ip(ip) {
            return Err(HardeningError::ForbiddenAddressRange);
        }
        return Err(HardeningError::ForbiddenAddressRange);
    }

    let is_allowed = approved_hosts
        .iter()
        .any(|allowed| allowed.trim().eq_ignore_ascii_case(&normalized_host));
    if !is_allowed {
        return Err(HardeningError::HostNotAllowlisted);
    }

    Ok(normalized_host)
}

/// Checks whether a resolved IP address falls in loopback, RFC1918 private, link-local
/// (`169.254.0.0/16` cloud metadata), unspecified, or multicast ranges.
pub fn is_forbidden_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_forbidden_ipv4(v4),
        IpAddr::V6(v6) => is_forbidden_ipv6(v6),
    }
}

fn is_forbidden_ipv4(ip: Ipv4Addr) -> bool {
    let octets = ip.octets();
    ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_broadcast()
        || ip.is_documentation()
        || ip.is_unspecified()
        || ip.is_multicast()
        // Shared address space (RFC 6598: 100.64.0.0/10)
        || (octets[0] == 100 && (octets[1] & 0b1100_0000) == 64)
        // Cloud metadata IP explicitly (169.254.169.254)
        || (octets[0] == 169 && octets[1] == 254)
}

fn is_forbidden_ipv6(ip: Ipv6Addr) -> bool {
    let segments = ip.segments();
    ip.is_loopback()
        || ip.is_unspecified()
        || ip.is_multicast()
        // Unique local (fc00::/7)
        || (segments[0] & 0xfe00) == 0xfc00
        // Link-local (fe80::/10)
        || (segments[0] & 0xffc0) == 0xfe80
}

/// Hospital-aware rate-limit key that binds `(endpoint_scope, subject_key, hospital_id, ip_bucket)`
/// so multiple doctors behind the same hospital NAT IP do not share a single lock-out bucket (`22-pwa-api-and-abuse-security.md`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositeRateLimitKey {
    canonical_key: String,
}

impl CompositeRateLimitKey {
    pub fn new(
        endpoint_scope: &str,
        subject_id: &str,
        hospital_id: Option<&str>,
        ip_bucket: &str,
    ) -> Result<Self, HardeningError> {
        let scope = endpoint_scope.trim();
        let subject = subject_id.trim();
        let ip = ip_bucket.trim();
        if scope.is_empty() || subject.is_empty() || ip.is_empty() {
            return Err(HardeningError::EmptyRateLimitDimension);
        }
        let hosp = hospital_id.map(str::trim).filter(|s| !s.is_empty()).unwrap_or("public");
        Ok(Self {
            canonical_key: format!("rl:{scope}:hosp:{hosp}:subj:{subject}:ip:{ip}"),
        })
    }

    pub fn as_str(&self) -> &str {
        &self.canonical_key
    }
}
