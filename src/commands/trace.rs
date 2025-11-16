use anyhow::Result;
use colored::*;
use std::net::IpAddr;
use std::time::Duration;

use crate::network::resolver::resolve_hostname;
use crate::network::traceroute::Traceroute;

pub async fn trace_command(host: String, max_hops: u32, timeout: Duration) -> Result<()> {
    println!("{} {}", "🛣️ TRACEROUTE".bright_green().bold(), host.bright_white().bold());

    // Resolve hostname to IP
    let target_ip = match resolve_hostname(&host).await {
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

    println!("Tracing route to {} with maximum {} hops", 
        target_ip.to_string().bright_yellow(), 
        max_hops.to_string().bright_cyan()
    );
    println!();

    let mut traceroute = Traceroute::new(target_ip, max_hops, timeout)?;
    
    for hop in 1..=max_hops {
        print!("{:3} ", hop.to_string().bright_cyan());
        
        match traceroute.trace_hop(hop).await {
            Ok(Some((hop_ip, rtt))) => {
                // Try to resolve the IP to hostname
                let hostname = match resolve_ip_to_hostname(hop_ip).await {
                    Ok(name) if name != hop_ip.to_string() => {
                        format!("{} ({})", name, hop_ip)
                    }
                    _ => hop_ip.to_string()
                };
                
                println!("{} {}ms", 
                    hostname.bright_white(),
                    format!("{:.2}", rtt.as_secs_f64() * 1000.0).bright_green()
                );
                
                // Check if we've reached the destination
                if hop_ip == target_ip {
                    println!();
                    println!("{} Trace complete - reached destination!", "🎯".green());
                    break;
                }
            }
            Ok(None) => {
                println!("{}", "* * *    Request timed out".yellow());
            }
            Err(e) => {
                println!("{} Error: {}", "❌".red(), e.to_string().red());
            }
        }
        
        // Small delay between hops
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    println!();
    println!("{} Traceroute completed", "📊".bright_blue());

    Ok(())
}

async fn resolve_ip_to_hostname(ip: IpAddr) -> Result<String> {
    // Simple reverse DNS lookup
    // In a real implementation, you'd use a proper DNS resolver
    // For now, we'll just return the IP as string
    Ok(ip.to_string())
}