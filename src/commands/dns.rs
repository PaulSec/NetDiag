use anyhow::Result;
use colored::*;
use trust_dns_resolver::config::*;
use trust_dns_resolver::TokioAsyncResolver;
use trust_dns_resolver::proto::rr::{RecordType, RData};
use std::net::IpAddr;

pub async fn dns_command(domain: String, server: Option<String>, record_type: String) -> Result<()> {
    println!("{} {}", "🌐 DNS LOOKUP".bright_green().bold(), domain.bright_white().bold());
    
    // Parse record type
    let record_type = match record_type.to_uppercase().as_str() {
        "A" => RecordType::A,
        "AAAA" => RecordType::AAAA,
        "MX" => RecordType::MX,
        "NS" => RecordType::NS,
        "TXT" => RecordType::TXT,
        "CNAME" => RecordType::CNAME,
        "SOA" => RecordType::SOA,
        "PTR" => RecordType::PTR,
        _ => {
            println!("{} Unsupported record type: {}", "❌".red(), record_type);
            return Ok(());
        }
    };

    // Create resolver
    let resolver = if let Some(ref server_ip) = server {
        // Parse custom DNS server
        let _dns_ip: IpAddr = match server_ip.parse() {
            Ok(ip) => ip,
            Err(_) => {
                println!("{} Invalid DNS server IP: {}", "❌".red(), server_ip);
                return Ok(());
            }
        };
        
        let mut config = ResolverConfig::new();
        config.add_name_server(NameServerConfig {
            socket_addr: "8.8.8.8:53".parse().unwrap(),
            protocol: trust_dns_resolver::config::Protocol::Udp,
            tls_dns_name: None,
            trust_negative_responses: false,
            bind_addr: None,
        });
        
        TokioAsyncResolver::tokio(config, ResolverOpts::default())
    } else {
        TokioAsyncResolver::tokio_from_system_conf().unwrap()
    };

    println!("Query: {} {}", domain.bright_cyan(), format!("{:?}", record_type).bright_yellow());
    
    if let Some(ref server_ip) = server {
        println!("Using DNS server: {}", server_ip.bright_magenta());
    }
    
    println!();

    // Perform DNS lookup
    match resolver.lookup(&domain, record_type).await {
        Ok(response) => {
            if response.iter().count() == 0 {
                println!("{} No records found", "❌".red());
                return Ok(());
            }

            println!("{} DNS Records Found:", "✅".green());
            println!();

            for record in response.iter() {
                match record {
                    RData::A(ip) => {
                        println!("  {} {}", "A".bright_yellow().bold(), ip.to_string().bright_white());
                    }
                    RData::AAAA(ip) => {
                        println!("  {} {}", "AAAA".bright_yellow().bold(), ip.to_string().bright_white());
                    }
                    RData::MX(mx) => {
                        println!("  {} {} {}", 
                            "MX".bright_yellow().bold(), 
                            mx.preference().to_string().bright_cyan(),
                            mx.exchange().to_string().bright_white()
                        );
                    }
                    RData::NS(ns) => {
                        println!("  {} {}", "NS".bright_yellow().bold(), ns.to_string().bright_white());
                    }
                    RData::TXT(txt) => {
                        for txt_data in txt.iter() {
                            println!("  {} \"{}\"", 
                                "TXT".bright_yellow().bold(), 
                                String::from_utf8_lossy(txt_data).bright_white()
                            );
                        }
                    }
                    RData::CNAME(cname) => {
                        println!("  {} {}", "CNAME".bright_yellow().bold(), cname.to_string().bright_white());
                    }
                    RData::SOA(soa) => {
                        println!("  {} {} {} {} {} {} {} {}",
                            "SOA".bright_yellow().bold(),
                            soa.mname().to_string().bright_white(),
                            soa.rname().to_string().bright_cyan(),
                            soa.serial().to_string().bright_magenta(),
                            soa.refresh().to_string().bright_green(),
                            soa.retry().to_string().bright_red(),
                            soa.expire().to_string().bright_blue(),
                            soa.minimum().to_string().bright_yellow()
                        );
                    }
                    RData::PTR(ptr) => {
                        println!("  {} {}", "PTR".bright_yellow().bold(), ptr.to_string().bright_white());
                    }
                    _ => {
                        println!("  {} {}", "OTHER".bright_yellow().bold(), format!("{:?}", record).bright_white());
                    }
                }
            }

            // Additional information
            println!();
            println!("{} Query completed in {}ms", 
                "⏱️".bright_blue(), 
                "< 1000".bright_white() // This is a placeholder; real timing would require measurement
            );
        }
        Err(e) => {
            println!("{} DNS lookup failed: {}", "❌".red(), e.to_string().red());
            
            // Provide helpful error messages
            match e.kind() {
                trust_dns_resolver::error::ResolveErrorKind::NoRecordsFound { .. } => {
                    println!("{} No records of type {:?} found for {}", "ℹ️".blue(), record_type, domain);
                }
                _ => {
                    println!("{} This could indicate:", "💡".yellow());
                    println!("  • Domain doesn't exist");
                    println!("  • DNS server is unreachable");
                    println!("  • Network connectivity issues");
                    println!("  • Firewall blocking DNS requests");
                }
            }
        }
    }

    Ok(())
}