use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use url::Url;

use openissa_core::error::{OpenIssaError, Result};

pub struct NetworkFirewall;

impl NetworkFirewall {
    /// Verify that an IP address is a safe public routable address (not internal or cloud metadata).
    pub fn is_ip_allowed(ip: &IpAddr) -> bool {
        match ip {
            IpAddr::V4(ipv4) => Self::is_ipv4_allowed(ipv4),
            IpAddr::V6(ipv6) => Self::is_ipv6_allowed(ipv6),
        }
    }

    fn is_ipv4_allowed(ip: &Ipv4Addr) -> bool {
        let octets = ip.octets();

        // 127.0.0.0/8 (Loopback)
        if octets[0] == 127 {
            return false;
        }

        // 10.0.0.0/8 (Private)
        if octets[0] == 10 {
            return false;
        }

        // 172.16.0.0/12 (Private)
        if octets[0] == 172 && (16..=31).contains(&octets[1]) {
            return false;
        }

        // 192.168.0.0/16 (Private)
        if octets[0] == 192 && octets[1] == 168 {
            return false;
        }

        // 169.254.0.0/16 (Link-Local & Cloud Metadata AWS/GCP 169.254.169.254)
        if octets[0] == 169 && octets[1] == 254 {
            return false;
        }

        // 0.0.0.0/8 (Current network)
        if octets[0] == 0 {
            return false;
        }

        true
    }

    fn is_ipv6_allowed(ip: &Ipv6Addr) -> bool {
        // ::1 (Loopback)
        if ip.is_loopback() {
            return false;
        }

        let segments = ip.segments();

        // fe80::/10 (Link-local)
        if (segments[0] & 0xffc0) == 0xfe80 {
            return false;
        }

        // fc00::/7 (Unique local address)
        if (segments[0] & 0xfe00) == 0xfc00 {
            return false;
        }

        true
    }

    /// Check a URL target before connecting.
    pub fn validate_url(raw_url: &str) -> Result<Url> {
        let parsed = Url::parse(raw_url).map_err(|e| OpenIssaError::InvalidUrl(e.to_string()))?;

        if let Some(host) = parsed.host_str() {
            if host == "localhost" || host.ends_with(".local") || host.ends_with(".internal") {
                return Err(OpenIssaError::SsrfBlocked {
                    ip: host.to_string(),
                });
            }

            if let Ok(ip) = host.parse::<IpAddr>() {
                if !Self::is_ip_allowed(&ip) {
                    return Err(OpenIssaError::SsrfBlocked { ip: ip.to_string() });
                }
            }
        }

        Ok(parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_firewall_blocks_private_ips() {
        assert!(!NetworkFirewall::is_ip_allowed(
            &"127.0.0.1".parse().unwrap()
        ));
        assert!(!NetworkFirewall::is_ip_allowed(
            &"10.0.1.5".parse().unwrap()
        ));
        assert!(!NetworkFirewall::is_ip_allowed(
            &"172.16.0.1".parse().unwrap()
        ));
        assert!(!NetworkFirewall::is_ip_allowed(
            &"192.168.1.1".parse().unwrap()
        ));
        assert!(!NetworkFirewall::is_ip_allowed(
            &"169.254.169.254".parse().unwrap()
        )); // AWS metadata
        assert!(NetworkFirewall::is_ip_allowed(&"1.1.1.1".parse().unwrap())); // Public Cloudflare DNS
        assert!(NetworkFirewall::is_ip_allowed(&"8.8.8.8".parse().unwrap())); // Public Google DNS
    }

    #[test]
    fn test_firewall_blocks_localhost_url() {
        assert!(NetworkFirewall::validate_url("http://localhost:8080/admin").is_err());
        assert!(NetworkFirewall::validate_url("http://127.0.0.1/").is_err());
        assert!(NetworkFirewall::validate_url("https://api.github.com/users").is_ok());
    }
}
