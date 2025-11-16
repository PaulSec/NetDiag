use anyhow::Result;
use std::net::IpAddr;
use std::time::{Duration, Instant};
use tokio::time;

pub struct IcmpPinger {
    target: IpAddr,
}

impl IcmpPinger {
    pub fn new(target: IpAddr) -> Result<Self> {
        Ok(Self { target })
    }

    pub async fn ping(&self, sequence: u16) -> Result<Duration> {
        let start_time = Instant::now();
        
        // Create ICMP echo request packet
        let _packet = create_icmp_packet(sequence);
        
        // For this simplified implementation, we'll just test TCP connectivity
        // In a real implementation, you'd send actual ICMP packets
        self.test_connectivity().await?;
        
        Ok(start_time.elapsed())
    }

    async fn test_connectivity(&self) -> Result<()> {
        // Simplified connectivity test using TCP connection
        let addr = match self.target {
            IpAddr::V4(ipv4) => std::net::SocketAddr::V4(std::net::SocketAddrV4::new(ipv4, 80)),
            IpAddr::V6(ipv6) => std::net::SocketAddr::V6(std::net::SocketAddrV6::new(ipv6, 80, 0, 0)),
        };

        match time::timeout(Duration::from_secs(1), tokio::net::TcpStream::connect(addr)).await {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(_)) => {
                // Try port 443 if 80 fails
                let addr_443 = match self.target {
                    IpAddr::V4(ipv4) => std::net::SocketAddr::V4(std::net::SocketAddrV4::new(ipv4, 443)),
                    IpAddr::V6(ipv6) => std::net::SocketAddr::V6(std::net::SocketAddrV6::new(ipv6, 443, 0, 0)),
                };
                
                match time::timeout(Duration::from_secs(1), tokio::net::TcpStream::connect(addr_443)).await {
                    Ok(Ok(_)) => Ok(()),
                    _ => Err(anyhow::anyhow!("Host unreachable"))
                }
            }
            Err(_) => Err(anyhow::anyhow!("Timeout")),
        }
    }
}

fn create_icmp_packet(sequence: u16) -> Vec<u8> {
    let mut packet = vec![0u8; 8];
    
    // ICMP Header: Type (1 byte) + Code (1 byte) + Checksum (2 bytes) + ID (2 bytes) + Sequence (2 bytes)
    packet[0] = 8; // Echo Request
    packet[1] = 0; // Code
    packet[2] = 0; // Checksum (will be calculated)
    packet[3] = 0;
    packet[4] = (std::process::id() & 0xff) as u8; // ID (low byte)
    packet[5] = ((std::process::id() >> 8) & 0xff) as u8; // ID (high byte)
    packet[6] = (sequence & 0xff) as u8; // Sequence (low byte)
    packet[7] = ((sequence >> 8) & 0xff) as u8; // Sequence (high byte)
    
    // Calculate checksum
    let checksum = calculate_checksum(&packet);
    packet[2] = (checksum & 0xff) as u8;
    packet[3] = ((checksum >> 8) & 0xff) as u8;
    
    packet
}

fn calculate_checksum(data: &[u8]) -> u16 {
    let mut sum = 0u32;
    
    // Sum all 16-bit words
    for chunk in data.chunks(2) {
        let word = if chunk.len() == 2 {
            u16::from_be_bytes([chunk[0], chunk[1]])
        } else {
            u16::from_be_bytes([chunk[0], 0])
        };
        sum += word as u32;
    }
    
    // Add carry bits
    while (sum >> 16) != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    
    // One's complement
    !sum as u16
}