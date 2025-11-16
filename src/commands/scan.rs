use anyhow::Result;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::time;

use crate::network::resolver::resolve_hostname;
use crate::utils::ports::{get_common_ports, parse_port_range};

pub async fn scan_command(
    host: String,
    ports: String,
    timeout: Duration,
    concurrency: usize,
) -> Result<()> {
    println!("{} {}", "🔍 PORT SCAN".bright_green().bold(), host.bright_white().bold());

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

    // Determine ports to scan
    let ports_trimmed = ports.trim();
    let port_list = if ports_trimmed.eq_ignore_ascii_case("common") {
        println!(
            "{} Using preset of common service ports",
            "[i]".bright_blue()
        );
        let mut presets = get_common_ports();
        presets.sort_unstable();
        presets
    } else {
        match parse_port_range(ports_trimmed) {
            Ok(ports) => ports,
            Err(e) => {
                println!("{} Invalid port range: {}", "❌".red(), e);
                return Ok(());
            }
        }
    };

    println!("Scanning {} ports on {}", port_list.len(), ip.to_string().bright_yellow());
    println!();

    // Create progress bar
    let pb = ProgressBar::new(port_list.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({per_sec}) {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut handles = Vec::new();
    let open_ports = Arc::new(tokio::sync::Mutex::new(HashSet::new()));
    let total_ports = port_list.len();

    for port in &port_list {
        let permit = semaphore.clone().acquire_owned().await?;
        let pb = pb.clone();
        let open_ports = open_ports.clone();
        let ip = ip;
        let port = *port; // Clone the port value to avoid lifetime issues

        let handle = tokio::spawn(async move {
            let _permit = permit;
            let addr = SocketAddr::new(ip, port);
            
            let is_open = match time::timeout(timeout, TcpStream::connect(addr)).await {
                Ok(Ok(_)) => true,
                Ok(Err(_)) | Err(_) => false,
            };

            if is_open {
                open_ports.lock().await.insert(port);
            }

            pb.inc(1);
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        let _ = handle.await;
    }

    pb.finish_with_message("Scan complete");
    println!();

    // Display results
    let open_ports = open_ports.lock().await;
    
    if open_ports.is_empty() {
        println!("{} No open ports found", "❌".red());
    } else {
        println!("{} {} open ports found:", "✅".green(), open_ports.len());
        println!();
        
        let mut sorted_ports: Vec<_> = open_ports.iter().collect();
        sorted_ports.sort();
        
        for port in sorted_ports {
            let service = get_service_name(*port);
            let service_name = 
                if !service.is_empty() {
                    format!("({})", service).bright_white()
                } else {
                    "".bright_white()
                };
            
            println!(
                "  {} {} {}",
                format!("{}/tcp", port).bright_cyan().bold(),
                "OPEN".bright_green().bold(),
                service_name
            );
        }
    }

    println!();
    println!(
        "{} Scanned {} ports in total",
        "📊".bright_blue(),
        total_ports.to_string().bright_white()
    );

    Ok(())
}

fn get_service_name(port: u16) -> &'static str {
    match port {
        21 => "FTP",
        22 => "SSH",
        23 => "Telnet",
        25 => "SMTP",
        53 => "DNS",
        80 => "HTTP",
        110 => "POP3",
        143 => "IMAP",
        443 => "HTTPS",
        993 => "IMAPS",
        995 => "POP3S",
        3306 => "MySQL",
        5432 => "PostgreSQL",
        6379 => "Redis",
        27017 => "MongoDB",
        _ => "",
    }
}