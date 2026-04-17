//! SSRF (Server-Side Request Forgery) protection

use crate::error::Result;
use std::net::{IpAddr, ToSocketAddrs};

/// Check if an IP address is private or loopback
pub fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let octets = v4.octets();
            matches!(
                octets,
                [10, ..] |           // 10.0.0.0/8
                [172, 16..=31, ..] |  // 172.16.0.0/12
                [192, 168, ..] |      // 192.168.0.0/16
                [169, 254, ..] |      // 169.254.0.0/16 (link-local)
                [127, ..] |           // 127.0.0.0/8
                [0, 0, 0, 0] |        // 0.0.0.0
                [255, 255, 255, 255] // Broadcast
            )
        }
        IpAddr::V6(v6) => {
            let segments = v6.segments();
            (segments[0] & 0xfe00) == 0xfc00 ||  // Unique Local Address (fc00::/7)
            (segments[0] & 0xffc0) == 0xfe80 ||  // Link-local (fe80::/10)
            segments == [0, 0, 0, 0, 0, 0, 0, 1] // Loopback
        }
    }
}

/// Check if a hostname is in the blocklist
pub fn is_hostname_blocked(hostname: &str) -> bool {
    let blocked = [
        "localhost",
        "metadata.google.internal",
        "metadata.aws.internal",
        "instance-data",
        "169.254.169.254",
        "169.254.170.2",
    ];

    blocked.iter().any(|&b| hostname.eq_ignore_ascii_case(b))
}

/// Validate a URL against SSRF attacks
pub fn validate_url(url: &str) -> Result<()> {
    // Only allow http and https schemes
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(crate::error::Error::PermissionDenied(format!(
            "SSRF blocked: Only http:// and https:// URLs are allowed, got: {}",
            url
        )));
    }

    // Extract hostname
    let after_scheme = url.split("://").nth(1).unwrap_or("");
    let host_part = after_scheme.split('/').next().unwrap_or("");
    let hostname = host_part.split(':').next().unwrap_or("");

    // Check hostname blocklist
    if is_hostname_blocked(hostname) {
        return Err(crate::error::Error::PermissionDenied(format!(
            "SSRF blocked: Hostname '{}' is in blocklist",
            hostname
        )));
    }

    // Resolve hostname to IP addresses and check EVERY ONE
    // This protects against DNS rebinding attacks
    if let Ok(addrs) = host_part.to_socket_addrs() {
        for addr in addrs {
            let ip = addr.ip();
            if is_private_ip(&ip) {
                return Err(crate::error::Error::PermissionDenied(format!(
                    "SSRF blocked: Hostname '{}' resolves to private IP {}",
                    hostname, ip
                )));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn test_is_private_ip() {
        assert!(is_private_ip(&IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
        assert!(is_private_ip(&IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1))));
        assert!(is_private_ip(&IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))));
        assert!(is_private_ip(&IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))));
        assert!(is_private_ip(&IpAddr::V4(Ipv4Addr::new(
            169, 254, 169, 254
        ))));

        assert!(is_private_ip(&IpAddr::V6(Ipv6Addr::new(
            0xfc00, 0, 0, 0, 0, 0, 0, 1
        ))));
        assert!(is_private_ip(&IpAddr::V6(Ipv6Addr::new(
            0xfe80, 0, 0, 0, 0, 0, 0, 1
        ))));
        assert!(is_private_ip(&IpAddr::V6(Ipv6Addr::LOCALHOST)));

        assert!(!is_private_ip(&IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))));
        assert!(!is_private_ip(&IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1))));
    }

    #[test]
    fn test_is_hostname_blocked() {
        assert!(is_hostname_blocked("localhost"));
        assert!(is_hostname_blocked("169.254.169.254"));
        assert!(is_hostname_blocked("metadata.google.internal"));

        assert!(!is_hostname_blocked("google.com"));
        assert!(!is_hostname_blocked("github.com"));
    }

    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://google.com").is_ok());
        assert!(validate_url("https://github.com").is_ok());

        assert!(validate_url("file:///etc/passwd").is_err());
        assert!(validate_url("gopher://localhost").is_err());
        assert!(validate_url("http://localhost").is_err());
        assert!(validate_url("http://169.254.169.254/latest/meta-data/").is_err());
    }
}
