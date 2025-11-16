use anyhow::Result;
use std::collections::HashSet;

pub fn parse_port_range(port_spec: &str) -> Result<Vec<u16>> {
    let mut ports = HashSet::new();
    
    for part in port_spec.split(',') {
        let part = part.trim();
        
        if part.contains('-') {
            // Range like "80-443"
            let range_parts: Vec<&str> = part.split('-').collect();
            if range_parts.len() != 2 {
                return Err(anyhow::anyhow!("Invalid port range format: {}", part));
            }
            
            let start: u16 = range_parts[0].parse()
                .map_err(|_| anyhow::anyhow!("Invalid start port: {}", range_parts[0]))?;
            let end: u16 = range_parts[1].parse()
                .map_err(|_| anyhow::anyhow!("Invalid end port: {}", range_parts[1]))?;
            
            if start > end {
                return Err(anyhow::anyhow!("Start port {} is greater than end port {}", start, end));
            }
            
            for port in start..=end {
                ports.insert(port);
            }
        } else {
            // Single port like "80"
            let port: u16 = part.parse()
                .map_err(|_| anyhow::anyhow!("Invalid port number: {}", part))?;
            
            if port == 0 {
                return Err(anyhow::anyhow!("Port number cannot be 0"));
            }
            
            ports.insert(port);
        }
    }
    
    if ports.is_empty() {
        return Err(anyhow::anyhow!("No valid ports specified"));
    }
    
    let mut port_vec: Vec<u16> = ports.into_iter().collect();
    port_vec.sort();
    
    Ok(port_vec)
}

pub fn get_common_ports() -> Vec<u16> {
    vec![
        20, 21,    // FTP
        22,        // SSH
        23,        // Telnet
        25,        // SMTP
        53,        // DNS
        80,        // HTTP
        110,       // POP3
        143,       // IMAP
        443,       // HTTPS
        993,       // IMAPS
        995,       // POP3S
        1433,      // MSSQL
        3306,      // MySQL
        5432,      // PostgreSQL
        6379,      // Redis
        27017,     // MongoDB
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_port() {
        assert_eq!(parse_port_range("80").unwrap(), vec![80]);
    }

    #[test]
    fn test_multiple_ports() {
        let mut result = parse_port_range("80,443,22").unwrap();
        result.sort();
        assert_eq!(result, vec![22, 80, 443]);
    }

    #[test]
    fn test_port_range() {
        let result = parse_port_range("80-85").unwrap();
        assert_eq!(result, vec![80, 81, 82, 83, 84, 85]);
    }

    #[test]
    fn test_mixed_format() {
        let mut result = parse_port_range("22,80-82,443").unwrap();
        result.sort();
        assert_eq!(result, vec![22, 80, 81, 82, 443]);
    }

    #[test]
    fn test_invalid_port() {
        assert!(parse_port_range("0").is_err());
        assert!(parse_port_range("65536").is_err());
        assert!(parse_port_range("abc").is_err());
    }

    #[test]
    fn test_invalid_range() {
        assert!(parse_port_range("443-80").is_err());
        assert!(parse_port_range("80-").is_err());
        assert!(parse_port_range("-80").is_err());
    }
}