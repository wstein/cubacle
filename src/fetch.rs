//! One HTTP GET with the safety rules of `cubacle patterns update`.
//!
//! This is the only network code of cubacle. It runs only for a URL the user
//! listed in the config, and it hands back bytes: nothing it receives is ever
//! executed, the caller parses it as a tab table.
//!
//! Rules: https only (plain http only to the loopback host, which the tests
//! use), at most [`MAX_BYTES`], a connect and a total timeout, at most
//! [`MAX_REDIRECTS`] redirects and each of them to https (or to the same
//! loopback host). Proxies come from `HTTPS_PROXY`, `https_proxy` and
//! `ALL_PROXY` through ureq.

#[cfg(feature = "fetch")]
use std::time::Duration;

/// The largest download accepted.
pub const MAX_BYTES: u64 = 50 * 1024 * 1024;
pub const MAX_REDIRECTS: usize = 5;
#[cfg(feature = "fetch")]
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
#[cfg(feature = "fetch")]
const TOTAL_TIMEOUT: Duration = Duration::from_secs(120);

/// What the server told us about the file, to ask "changed?" next time.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Validators {
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

#[derive(Debug)]
pub enum Fetched {
    NotModified,
    Body {
        bytes: Vec<u8>,
        validators: Validators,
    },
}

/// Scheme and host of an `http(s)://` URL; `None` if it has no host or
/// carries credentials.
fn split_url(url: &str) -> Option<(&str, &str)> {
    let (scheme, rest) = url.split_once("://")?;
    let authority = rest.split(['/', '?', '#']).next()?;
    if authority.is_empty() || authority.contains('@') {
        return None;
    }
    let host = match authority.strip_prefix('[') {
        Some(v6) => v6.split(']').next()?,
        None => authority.split(':').next()?,
    };
    Some((scheme, host))
}

fn is_loopback(host: &str) -> bool {
    matches!(host, "127.0.0.1" | "localhost" | "::1")
}

/// Whether `url` may be fetched at all.
pub fn check_url(url: &str) -> Result<(), String> {
    match split_url(url) {
        Some(("https", _)) => Ok(()),
        Some(("http", host)) if is_loopback(host) => Ok(()),
        Some(("http", _)) => Err(
            "plain http is refused; use an https URL (http is allowed only for 127.0.0.1 and localhost)"
                .to_owned(),
        ),
        _ => Err("not an https URL (and no credentials in the URL, please)".to_owned()),
    }
}

/// The URL a redirect leads to, if it is allowed from `from`.
pub fn redirect_target(from: &str, location: &str) -> Result<String, String> {
    let target = if location.contains("://") {
        location.to_owned()
    } else if location.starts_with('/') {
        let (scheme, rest) = from.split_once("://").ok_or("bad URL")?;
        let host = rest.split(['/', '?', '#']).next().unwrap_or(rest);
        format!("{scheme}://{host}{location}")
    } else {
        return Err(format!("unsupported redirect to {location}"));
    };
    match (split_url(from), split_url(&target)) {
        (_, Some(("https", _))) => Ok(target),
        (Some((_, a)), Some(("http", b))) if is_loopback(a) && a == b => Ok(target),
        _ => Err(format!(
            "redirect to {target} refused (only https is followed)"
        )),
    }
}

#[cfg(feature = "fetch")]
pub fn get(url: &str, have: &Validators) -> Result<Fetched, String> {
    use std::io::Read;

    check_url(url)?;
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_connect(Some(CONNECT_TIMEOUT))
        .timeout_global(Some(TOTAL_TIMEOUT))
        .max_redirects(0)
        .http_status_as_error(false)
        .user_agent(concat!("cubacle/", env!("CARGO_PKG_VERSION")))
        .build()
        .into();
    let mut current = url.to_owned();
    for _ in 0..=MAX_REDIRECTS {
        let mut request = agent.get(&current);
        if let Some(etag) = &have.etag {
            request = request.header("If-None-Match", etag);
        }
        if let Some(date) = &have.last_modified {
            request = request.header("If-Modified-Since", date);
        }
        let response = request.call().map_err(|e| e.to_string())?;
        let status = response.status().as_u16();
        let header = |name: &str| {
            response
                .headers()
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(str::to_owned)
        };
        match status {
            304 => return Ok(Fetched::NotModified),
            301 | 302 | 303 | 307 | 308 => {
                let location = header("location").ok_or("redirect without Location")?;
                current = redirect_target(&current, &location)?;
            }
            200 => {
                if let Some(length) = header("content-length").and_then(|v| v.parse::<u64>().ok()) {
                    if length > MAX_BYTES {
                        return Err(format!(
                            "{length} bytes announced, more than the limit of {MAX_BYTES}"
                        ));
                    }
                }
                let validators = Validators {
                    etag: header("etag"),
                    last_modified: header("last-modified"),
                };
                let mut bytes = Vec::new();
                response
                    .into_body()
                    .into_reader()
                    .take(MAX_BYTES + 1)
                    .read_to_end(&mut bytes)
                    .map_err(|e| format!("download interrupted: {e}"))?;
                if bytes.len() as u64 > MAX_BYTES {
                    return Err(format!("more than the limit of {MAX_BYTES} bytes"));
                }
                return Ok(Fetched::Body { bytes, validators });
            }
            other => return Err(format!("HTTP status {other}")),
        }
    }
    Err(format!("more than {MAX_REDIRECTS} redirects"))
}

#[cfg(not(feature = "fetch"))]
pub fn get(url: &str, _have: &Validators) -> Result<Fetched, String> {
    check_url(url)?;
    Err("this cubacle was built without the `fetch` feature".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_https_and_loopback_http() {
        assert!(check_url("https://example.org/x").is_ok());
        assert!(check_url("http://127.0.0.1:8080/x").is_ok());
        assert!(check_url("http://localhost/x").is_ok());
        assert!(check_url("http://example.org/x")
            .unwrap_err()
            .contains("plain http"));
        assert!(check_url("http://127.0.0.1@evil.org/x").is_err());
        assert!(check_url("ftp://example.org/x").is_err());
    }

    #[test]
    fn redirects_stay_on_https() {
        let from = "https://a.org/x";
        assert_eq!(redirect_target(from, "/y").unwrap(), "https://a.org/y");
        assert!(redirect_target(from, "https://b.org/z").is_ok());
        assert!(redirect_target(from, "http://b.org/z").is_err());
        let local = "http://127.0.0.1:9/x";
        assert!(redirect_target(local, "http://127.0.0.1:9/y").is_ok());
        assert!(redirect_target(local, "http://localhost:9/y").is_err());
    }
}
