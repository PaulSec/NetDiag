use anyhow::Result;
use colored::*;
use std::time::{Duration, Instant};
use tokio::time;

use crate::network::icmp::IcmpPinger;
use crate::network::resolver::resolve_hostname;
use crate::utils::format::format_duration;

pub async fn ping_command(host: String, count: u32, timeout: Duration, _size: usize) -> Result<()> {
    println!("{} {}", "🏓 PING".bright_green().bold(), host.bright_white().bold());
    
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

    let pinger = IcmpPinger::new(ip)?;
    let mut successful_pings = 0;
    let mut total_time = Duration::ZERO;
    let mut min_time = Duration::MAX;
    let mut max_time = Duration::ZERO;

    println!();
    
    for seq in 0..count {
        let _start = Instant::now();
        
        match time::timeout(timeout, pinger.ping(seq as u16)).await {
            Ok(Ok(duration)) => {
                successful_pings += 1;
                total_time += duration;
                min_time = min_time.min(duration);
                max_time = max_time.max(duration);
                
                println!(
                    "{} Reply from {}: seq={} time={}",
                    "✓".bright_green(),
                    ip.to_string().bright_yellow(),
                    seq.to_string().bright_cyan(),
                    format_duration(duration).bright_white()
                );
            }
            Ok(Err(e)) => {
                println!(
                    "{} Request timeout for seq={}: {}",
                    "✗".bright_red(),
                    seq.to_string().bright_cyan(),
                    e.to_string().red()
                );
            }
            Err(_) => {
                println!(
                    "{} Request timeout for seq={} ({})",
                    "⏰".yellow(),
                    seq.to_string().bright_cyan(),
                    format_duration(timeout).yellow()
                );
            }
        }
        
        if seq < count - 1 {
            time::sleep(Duration::from_secs(1)).await;
        }
    }

    // Print statistics
    println!();
    println!("{}", "📊 PING STATISTICS".bright_blue().bold());
    println!("Packets: Sent = {}, Received = {}, Lost = {} ({:.1}%)",
        count.to_string().bright_white(),
        successful_pings.to_string().bright_green(),
        (count - successful_pings).to_string().bright_red(),
        ((count - successful_pings) as f64 / count as f64 * 100.0)
    );

    if successful_pings > 0 {
        let avg_time = total_time / successful_pings;
        println!("Round-trip times: min = {}, max = {}, avg = {}",
            format_duration(min_time).bright_green(),
            format_duration(max_time).bright_red(),
            format_duration(avg_time).bright_yellow()
        );
    }

    Ok(())
}