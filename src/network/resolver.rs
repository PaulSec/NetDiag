use anyhow::Result;
use std::net::IpAddr;

use trust_dns_resolver::TokioAsyncResolver;

pub async fn resolve_hostname(hostname: &str) -> Result<IpAddr> {
    // Try to parse as IP first
    if let Ok(ip) = hostname.parse::<IpAddr>() {
        return Ok(ip);
    }

    // Create resolver
    let resolver = TokioAsyncResolver::tokio_from_system_conf()
        .map_err(|e| anyhow::anyhow!("Failed to create DNS resolver: {}", e))?;

    // Try IPv4 first
    match resolver.lookup_ip(hostname).await {
        Ok(response) => {
            if let Some(ip) = response.iter().next() {
                Ok(ip)
            } else {
                Err(anyhow::anyhow!("No IP addresses found for hostname"))
            }
        }
        Err(e) => Err(anyhow::anyhow!("DNS lookup failed: {}", e)),
    }
}