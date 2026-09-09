//! What counts as a server address, and whether the thing at it is WaterForm.
//!
//! Both halves are here rather than in the shell pages because the shell is the
//! only side that can ask without a browser's cross-origin rules in the way, and
//! because an address the customer types once, badly, is the difference between
//! "the application is broken" and a support call nobody needed.

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::time::Duration;
use url::Url;

/// Where the shell points when nobody has said otherwise: the common door every
/// customer of ours signs in at. An on-prem install replaces it from the menu.
pub const DEFAULT_SERVER: &str = "https://app.waterform.fabrikode.com";

const PROBE_TIMEOUT: Duration = Duration::from_secs(6);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressError {
    /// Nothing was typed.
    Empty,
    /// Not something `Url` can read at all.
    Malformed,
    /// A scheme we will not open, such as `file:` or `ftp:`.
    UnsupportedScheme,
    /// Plain http to a public address. Allowed on a private network, never over
    /// the internet: session cookies would travel in the clear.
    InsecurePublicHost,
}

/// Turns what the customer typed into the one address we will store.
///
/// Everything after the host is dropped. A server address is an origin; keeping
/// a path would send every later request somewhere it was never meant to go, and
/// people paste the address of the page they happen to be looking at.
pub fn normalise(input: &str) -> Result<String, AddressError> {
    let raw = input.trim();
    if raw.is_empty() {
        return Err(AddressError::Empty);
    }

    // `Url::parse("localhost:3000")` reads "localhost" as the scheme, so the
    // scheme has to be recognised by the separator rather than by the colon.
    let with_scheme = if raw.contains("://") {
        raw.to_string()
    } else {
        format!("https://{raw}")
    };

    let url = Url::parse(&with_scheme).map_err(|_| AddressError::Malformed)?;

    let scheme = url.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(AddressError::UnsupportedScheme);
    }

    let host = url.host_str().unwrap_or_default().to_ascii_lowercase();
    if host.is_empty() {
        return Err(AddressError::Malformed);
    }

    if scheme == "http" && !is_private_host(&host) {
        return Err(AddressError::InsecurePublicHost);
    }

    let mut out = format!("{scheme}://{host}");
    if let Some(port) = url.port() {
        // A default port is noise; anything else is part of the address.
        out.push_str(&format!(":{port}"));
    }
    Ok(out)
}

/// Addresses that cannot leave the building, where plain http is a reasonable
/// choice a factory network actually makes.
fn is_private_host(host: &str) -> bool {
    if host == "localhost" || host.ends_with(".localhost") || host.ends_with(".local") {
        return true;
    }
    // Url keeps IPv6 hosts in brackets.
    let bare = host.trim_start_matches('[').trim_end_matches(']');
    match bare.parse::<IpAddr>() {
        Ok(IpAddr::V4(ip)) => ip.is_private() || ip.is_loopback() || ip.is_link_local(),
        Ok(IpAddr::V6(ip)) => {
            ip.is_loopback()
                || (ip.segments()[0] & 0xfe00) == 0xfc00
                || (ip.segments()[0] & 0xffc0) == 0xfe80
        }
        Err(_) => false,
    }
}

/// What `/api/health` says about itself. Only the fields the shell acts on.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    #[serde(default)]
    pub app: String,
    #[serde(default)]
    pub ok: bool,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub database: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProbeError {
    /// Nothing answered: no route, no DNS, refused, or too slow.
    Unreachable,
    /// The certificate was refused. Kept apart from the above because the fix is
    /// completely different, and on-prem is where it happens.
    Certificate,
    /// Something answered and it was not WaterForm.
    NotWaterForm,
    /// WaterForm answered and said it is not ready (503: its database is down).
    NotReady,
}

/// Asks the address whether it is a WaterForm server that is ready to work.
pub async fn probe(base: &str) -> Result<Health, ProbeError> {
    let client = reqwest::Client::builder()
        .timeout(PROBE_TIMEOUT)
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|_| ProbeError::Unreachable)?;

    let response = client
        .get(format!("{base}/api/health"))
        .header("accept", "application/json")
        .send()
        .await
        .map_err(classify)?;

    let status = response.status();
    let health: Health = response
        .json()
        .await
        .map_err(|_| ProbeError::NotWaterForm)?;

    if health.app != "waterform" {
        return Err(ProbeError::NotWaterForm);
    }
    if !health.ok || status.is_server_error() {
        return Err(ProbeError::NotReady);
    }
    Ok(health)
}

/// A transport failure has to be told apart from a certificate one, because the
/// person reading the message can only act on the second.
fn classify(error: reqwest::Error) -> ProbeError {
    let mut source: Option<&(dyn std::error::Error + 'static)> = Some(&error);
    while let Some(err) = source {
        let text = err.to_string().to_ascii_lowercase();
        if text.contains("certificate")
            || text.contains("unknownissuer")
            || text.contains("self-signed")
            || text.contains("tls")
            || text.contains("handshake")
        {
            return ProbeError::Certificate;
        }
        source = err.source();
    }
    ProbeError::Unreachable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_the_scheme_nobody_types() {
        assert_eq!(
            normalise("app.waterform.fabrikode.com").unwrap(),
            "https://app.waterform.fabrikode.com"
        );
    }

    #[test]
    fn keeps_only_the_origin() {
        assert_eq!(
            normalise("https://app.waterform.fabrikode.com/engineering?job=4#top").unwrap(),
            "https://app.waterform.fabrikode.com"
        );
    }

    #[test]
    fn drops_the_trailing_slash_and_lowercases_the_host() {
        assert_eq!(
            normalise("  HTTPS://App.Waterform.Fabrikode.Com/  ").unwrap(),
            "https://app.waterform.fabrikode.com"
        );
    }

    #[test]
    fn keeps_a_port() {
        assert_eq!(
            normalise("depo.sirket.local:8443").unwrap(),
            "https://depo.sirket.local:8443"
        );
    }

    #[test]
    fn a_bare_host_and_port_is_not_a_scheme() {
        assert_eq!(
            normalise("localhost:3000").unwrap(),
            "https://localhost:3000"
        );
    }

    #[test]
    fn plain_http_is_for_the_building_only() {
        assert_eq!(
            normalise("http://192.168.1.40:3000").unwrap(),
            "http://192.168.1.40:3000"
        );
        assert_eq!(normalise("http://10.0.0.5").unwrap(), "http://10.0.0.5");
        assert_eq!(
            normalise("http://depo.sirket.local").unwrap(),
            "http://depo.sirket.local"
        );
        assert_eq!(
            normalise("http://localhost:3000").unwrap(),
            "http://localhost:3000"
        );
        assert_eq!(
            normalise("http://waterform.fabrikode.com"),
            Err(AddressError::InsecurePublicHost)
        );
        assert_eq!(normalise("http://172.16.3.9").unwrap(), "http://172.16.3.9");
        assert_eq!(
            normalise("http://172.32.3.9"),
            Err(AddressError::InsecurePublicHost)
        );
    }

    #[test]
    fn refuses_what_is_not_an_address() {
        assert_eq!(normalise("   "), Err(AddressError::Empty));
        assert_eq!(
            normalise("file:///Users/eally/waterform"),
            Err(AddressError::UnsupportedScheme)
        );
        assert_eq!(
            normalise("ftp://depo"),
            Err(AddressError::UnsupportedScheme)
        );
        // `url` refuses a scheme with nothing behind it, which is the same answer.
        assert_eq!(normalise("https://"), Err(AddressError::Malformed));
        // `https:///depo` is not refused: `url` reads the extra slash as noise and
        // "depo" as the host, which is what the person meant. It fails at the
        // probe instead, and says so.
        assert_eq!(normalise("https:///depo").unwrap(), "https://depo");
    }

    #[test]
    fn strips_credentials_someone_pasted() {
        assert_eq!(
            normalise("https://admin:sifre@depo.sirket.com/x").unwrap(),
            "https://depo.sirket.com"
        );
    }
}
