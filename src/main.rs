use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use std::time::Duration;

mod commands;
mod network;
mod utils;

use commands::*;

#[derive(Parser)]
#[command(name = "netdiag")]
#[command(about = "A comprehensive network diagnostic tool")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Test basic connectivity to a host
    Ping {
        /// Target host or IP address
        host: String,
        /// Number of packets to send
        #[arg(short = 'c', long, default_value = "4")]
        count: u32,
        /// Timeout in seconds
        #[arg(short = 't', long, default_value = "5")]
        timeout: u64,
        /// Packet size in bytes
        #[arg(short = 's', long, default_value = "64")]
        size: usize,
    },
    /// Scan ports on a target host
    Scan {
        /// Target host or IP address
        host: String,
        /// Port range (e.g., 80, 80-443, 22,80,443)
        #[arg(short = 'p', long, default_value = "1-1000")]
        ports: String,
        /// Timeout in milliseconds
        #[arg(short = 't', long, default_value = "3000")]
        timeout: u64,
        /// Number of concurrent connections
        #[arg(short = 'c', long, default_value = "100")]
        concurrency: usize,
    },
    /// Perform DNS resolution
    Dns {
        /// Domain name to resolve
        domain: String,
        /// DNS server to use (optional)
        #[arg(short = 's', long)]
        server: Option<String>,
        /// Record type (A, AAAA, MX, NS, TXT, etc.)
        #[arg(short = 't', long, default_value = "A")]
        record_type: String,
    },
    /// Test HTTP/HTTPS connectivity
    Http {
        /// URL to test
        url: String,
        /// Request timeout in seconds
        #[arg(short = 't', long, default_value = "10")]
        timeout: u64,
        /// Follow redirects
        #[arg(short = 'f', long)]
        follow_redirects: bool,
        /// Show response headers
        #[arg(short = 'H', long)]
        show_headers: bool,
    },
    /// Trace network path to destination
    Trace {
        /// Target host or IP address
        host: String,
        /// Maximum number of hops
        #[arg(short = 'm', long, default_value = "30")]
        max_hops: u32,
        /// Timeout per hop in seconds
        #[arg(short = 't', long, default_value = "5")]
        timeout: u64,
    },
    /// Test connection to specific port
    Connect {
        /// Target host or IP address
        host: String,
        /// Port number
        port: u16,
        /// Connection timeout in seconds
        #[arg(short = 't', long, default_value = "5")]
        timeout: u64,
        /// Test UDP instead of TCP
        #[arg(short = 'u', long)]
        udp: bool,
    },
    /// Generate network test report
    Report {
        /// Target host or IP address
        host: String,
        /// Output file path (optional, defaults to stdout)
        #[arg(short = 'o', long)]
        output: Option<String>,
        /// Include detailed port scan
        #[arg(long)]
        detailed_scan: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    println!("{}", format!("🔍 NetDiag - Network Diagnostic Tool v{}", env!("CARGO_PKG_VERSION")).bright_cyan().bold());
    println!();

    match cli.command {
        Commands::Ping { host, count, timeout, size } => {
            ping_command(host, count, Duration::from_secs(timeout), size).await
        }
        Commands::Scan { host, ports, timeout, concurrency } => {
            scan_command(host, ports, Duration::from_millis(timeout), concurrency).await
        }
        Commands::Dns { domain, server, record_type } => {
            dns_command(domain, server, record_type).await
        }
        Commands::Http { url, timeout, follow_redirects, show_headers } => {
            http_command(url, Duration::from_secs(timeout), follow_redirects, show_headers).await
        }
        Commands::Trace { host, max_hops, timeout } => {
            trace_command(host, max_hops, Duration::from_secs(timeout)).await
        }
        Commands::Connect { host, port, timeout, udp } => {
            connect_command(host, port, Duration::from_secs(timeout), udp).await
        }
        Commands::Report { host, output, detailed_scan } => {
            report_command(host, output, detailed_scan).await
        }
    }
}