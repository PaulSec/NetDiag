use anyhow::Result;
use colored::*;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::{TcpStream, UdpSocket};
use tokio::time;

use crate::network::resolver::resolve_hostname;

pub async fn connect_command(host: String, port: u16, timeout: Duration, udp: bool) -> Result<()> {
    let protocol = if udp { "UDP" } else { "TCP" };
    println!("{} to {}:{}", 
        format!("🔌 {} CONNECTION TEST", protocol).bright_green().bold(), 
        host.bright_white().bold(),
        port.to_string().bright_cyan().bold()
    );

    // Resolve hostname to IP
    let ip = match resolve_hostname(&host).await {
        Ok(ip) => {
            if ip.to_string() != host {
                println!("Resolved {} to {}", host.bright_cyan(), ip.to_string().bright_yellow());
            }
            ip
        }
        Err(e) => {
            println!("{} Failed to resolve hostname: {}", "❌".red(), e);
            return Ok(());
        }
    };

    let addr = SocketAddr::new(ip, port);
    println!("Testing {} connection to {}...", protocol, addr.to_string().bright_yellow());
    println!();

    let start_time = std::time::Instant::now();
    
    if udp {
        test_udp_connection(addr, timeout).await
    } else {
        test_tcp_connection(addr, timeout).await
    }?;

    let elapsed = start_time.elapsed();
    println!("Connection test completed in {}ms", 
        (elapsed.as_millis()).to_string().bright_white()
    );

    Ok(())
}

async fn test_tcp_connection(addr: SocketAddr, timeout: Duration) -> Result<()> {
    match time::timeout(timeout, TcpStream::connect(addr)).await {
        Ok(Ok(stream)) => {
            let local_addr = stream.local_addr().unwrap_or_else(|_| "unknown".parse().unwrap());
            println!("{} TCP connection successful!", "✅".green());
            println!("  Local address: {}", local_addr.to_string().bright_cyan());
            println!("  Remote address: {}", addr.to_string().bright_yellow());
            println!("  Status: {}", "CONNECTED".bright_green().bold());
            
            // Get some basic socket info
            if let Ok(peer_addr) = stream.peer_addr() {
                println!("  Peer address: {}", peer_addr.to_string().bright_magenta());
            }
        }
        Ok(Err(e)) => {
            println!("{} TCP connection failed: {}", "❌".red(), e.to_string().red());
            
            println!();
            println!("{} Possible causes:", "💡".yellow());
            match e.kind() {
                std::io::ErrorKind::ConnectionRefused => {
                    println!("  • Port is closed or no service is listening");
                    println!("  • Firewall is blocking the connection");
                    println!("  • Service is not running");
                }
                std::io::ErrorKind::TimedOut => {
                    println!("  • Network timeout");
                    println!("  • Host is unreachable");
                    println!("  • Firewall is dropping packets");
                }
                std::io::ErrorKind::HostUnreachable => {
                    println!("  • Host is not reachable");
                    println!("  • Routing issues");
                    println!("  • Network interface is down");
                }
                _ => {
                    println!("  • Network connectivity issues");
                    println!("  • DNS resolution problems");
                    println!("  • Firewall restrictions");
                }
            }
        }
        Err(_) => {
            println!("{} TCP connection timed out after {}s", 
                "⏰".yellow(), 
                timeout.as_secs().to_string().yellow()
            );
            
            println!();
            println!("{} This indicates:", "💡".yellow());
            println!("  • Host may be unreachable");
            println!("  • Firewall may be filtering packets");
            println!("  • Network latency is very high");
        }
    }
    
    Ok(())
}

async fn test_udp_connection(addr: SocketAddr, timeout: Duration) -> Result<()> {
    // UDP is connectionless, so we'll send a packet and see if we get a response
    let local_addr: SocketAddr = if addr.is_ipv4() {
        "0.0.0.0:0".parse().unwrap()
    } else {
        "[::]:0".parse().unwrap()
    };

    match UdpSocket::bind(local_addr).await {
        Ok(socket) => {
            println!("{} UDP socket created", "✅".green());
            println!("  Local address: {}", socket.local_addr().unwrap().to_string().bright_cyan());
            println!("  Target address: {}", addr.to_string().bright_yellow());
            
            // Try to send a test packet
            let test_data = b"netdiag-test-packet";
            
            match time::timeout(timeout, socket.send_to(test_data, addr)).await {
                Ok(Ok(bytes_sent)) => {
                    println!("  Sent: {} bytes", bytes_sent.to_string().bright_green());
                    
                    // Try to receive a response (with a shorter timeout)
                    let mut buffer = [0u8; 1024];
                    match time::timeout(
                        Duration::from_secs(2), 
                        socket.recv_from(&mut buffer)
                    ).await {
                        Ok(Ok((bytes_received, from))) => {
                            println!("  Received: {} bytes from {}", 
                                bytes_received.to_string().bright_green(),
                                from.to_string().bright_yellow()
                            );
                            println!("  Status: {}", "RESPONSE_RECEIVED".bright_green().bold());
                        }
                        Ok(Err(e)) => {
                            println!("  Receive error: {}", e.to_string().yellow());
                            println!("  Status: {}", "SENT_NO_RESPONSE".bright_yellow().bold());
                        }
                        Err(_) => {
                            println!("  No response received (this is normal for UDP)");
                            println!("  Status: {}", "SENT_NO_RESPONSE".bright_yellow().bold());
                        }
                    }
                }
                Ok(Err(e)) => {
                    println!("{} UDP send failed: {}", "❌".red(), e.to_string().red());
                }
                Err(_) => {
                    println!("{} UDP send timed out", "⏰".yellow());
                }
            }
            
            println!();
            println!("{} Note: UDP is connectionless", "ℹ️".blue());
            println!("  • No response doesn't necessarily mean failure");
            println!("  • Many UDP services don't respond to arbitrary data");
            println!("  • Firewalls often block UDP traffic");
        }
        Err(e) => {
            println!("{} Failed to create UDP socket: {}", "❌".red(), e.to_string().red());
        }
    }
    
    Ok(())
}