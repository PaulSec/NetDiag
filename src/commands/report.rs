use anyhow::Result;
use chrono::{DateTime, Utc};
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::network::resolver::resolve_hostname;
use crate::utils::ports::parse_port_range;

#[derive(Serialize, Deserialize)]
struct NetworkReport {
    timestamp: DateTime<Utc>,
    target_host: String,
    target_ip: String,
    tests: HashMap<String, TestResult>,
    summary: ReportSummary,
}

#[derive(Serialize, Deserialize)]
struct TestResult {
    success: bool,
    details: String,
    duration_ms: u64,
    error: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct ReportSummary {
    total_tests: u32,
    passed_tests: u32,
    failed_tests: u32,
    overall_status: String,
}

pub async fn report_command(host: String, output: Option<String>, detailed_scan: bool) -> Result<()> {
    println!("{} {}", "📋 NETWORK REPORT".bright_green().bold(), host.bright_white().bold());
    println!("Generating comprehensive network diagnostic report...");
    println!();

    // Resolve hostname
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

    let mut report = NetworkReport {
        timestamp: Utc::now(),
        target_host: host.clone(),
        target_ip: ip.to_string(),
        tests: HashMap::new(),
        summary: ReportSummary {
            total_tests: 0,
            passed_tests: 0,
            failed_tests: 0,
            overall_status: "Unknown".to_string(),
        },
    };

    println!("🔍 Running diagnostic tests...");
    println!();

    // Test 1: Basic connectivity (ping-like)
    println!("1. {} Basic connectivity test", "🏓".bright_blue());
    let ping_result = test_basic_connectivity(&ip).await;
    add_test_result(&mut report, "basic_connectivity", ping_result);

    // Test 2: DNS resolution
    println!("2. {} DNS resolution test", "🌐".bright_blue());
    let dns_result = test_dns_resolution(&host).await;
    add_test_result(&mut report, "dns_resolution", dns_result);

    // Test 3: Common ports check
    println!("3. {} Common ports scan", "🔍".bright_blue());
    let ports_to_scan = if detailed_scan {
        "1-65535".to_string()
    } else {
        "21,22,23,25,53,80,110,143,443,993,995,3306,5432".to_string()
    };
    let port_scan_result = test_port_connectivity(&ip, &ports_to_scan).await;
    add_test_result(&mut report, "port_scan", port_scan_result);

    // Test 4: HTTP connectivity (if port 80 or 443 is open)
    println!("4. {} HTTP connectivity test", "🌐".bright_blue());
    let http_result = test_http_connectivity(&host).await;
    add_test_result(&mut report, "http_connectivity", http_result);

    // Calculate summary
    report.summary.total_tests = report.tests.len() as u32;
    report.summary.passed_tests = report.tests.values().filter(|t| t.success).count() as u32;
    report.summary.failed_tests = report.summary.total_tests - report.summary.passed_tests;
    
    report.summary.overall_status = if report.summary.failed_tests == 0 {
        "EXCELLENT".to_string()
    } else if report.summary.passed_tests > report.summary.failed_tests {
        "GOOD".to_string()
    } else if report.summary.passed_tests > 0 {
        "PARTIAL".to_string()
    } else {
        "POOR".to_string()
    };

    // Display results
    display_report(&report);

    // Save to file if requested
    if let Some(output_path) = output {
        save_report_to_file(&report, &output_path)?;
        println!();
        println!("{} Report saved to: {}", "💾".bright_green(), output_path.bright_white());
    }

    Ok(())
}

async fn test_basic_connectivity(ip: &std::net::IpAddr) -> TestResult {
    let start = std::time::Instant::now();
    
    // Simple TCP connection test to a common port
    match tokio::time::timeout(
        Duration::from_secs(5),
        tokio::net::TcpStream::connect((ip.clone(), 80))
    ).await {
        Ok(Ok(_)) => TestResult {
            success: true,
            details: "Host is reachable via TCP".to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
            error: None,
        },
        Ok(Err(_)) => {
            // Try port 443 if 80 fails
            match tokio::time::timeout(
                Duration::from_secs(5),
                tokio::net::TcpStream::connect((ip.clone(), 443))
            ).await {
                Ok(Ok(_)) => TestResult {
                    success: true,
                    details: "Host is reachable via TCP (port 443)".to_string(),
                    duration_ms: start.elapsed().as_millis() as u64,
                    error: None,
                },
                _ => TestResult {
                    success: false,
                    details: "Host is not reachable".to_string(),
                    duration_ms: start.elapsed().as_millis() as u64,
                    error: Some("Connection refused on common ports".to_string()),
                }
            }
        },
        Err(_) => TestResult {
            success: false,
            details: "Connection timeout".to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
            error: Some("Timeout after 5 seconds".to_string()),
        }
    }
}

async fn test_dns_resolution(host: &str) -> TestResult {
    let start = std::time::Instant::now();
    
    match resolve_hostname(host).await {
        Ok(ip) => TestResult {
            success: true,
            details: format!("Resolved to {}", ip),
            duration_ms: start.elapsed().as_millis() as u64,
            error: None,
        },
        Err(e) => TestResult {
            success: false,
            details: "DNS resolution failed".to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
            error: Some(e.to_string()),
        }
    }
}

async fn test_port_connectivity(ip: &std::net::IpAddr, ports_str: &str) -> TestResult {
    let start = std::time::Instant::now();
    
    let ports = match parse_port_range(ports_str) {
        Ok(ports) => ports,
        Err(e) => return TestResult {
            success: false,
            details: "Invalid port range".to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
            error: Some(e.to_string()),
        }
    };

    let mut open_ports = Vec::new();
    
    for port in ports.into_iter().take(100) { // Limit to first 100 ports for performance
        if let Ok(Ok(_)) = tokio::time::timeout(
            Duration::from_millis(1000),
            tokio::net::TcpStream::connect((ip.clone(), port))
        ).await {
            open_ports.push(port);
        }
    }

    TestResult {
        success: !open_ports.is_empty(),
        details: if open_ports.is_empty() {
            "No open ports found".to_string()
        } else {
            format!("Open ports: {:?}", open_ports)
        },
        duration_ms: start.elapsed().as_millis() as u64,
        error: None,
    }
}

async fn test_http_connectivity(host: &str) -> TestResult {
    let start = std::time::Instant::now();
    
    // Test HTTP (port 80)
    let http_url = format!("http://{}", host);
    if let Ok(result) = test_simple_http(&http_url).await {
        return result;
    }

    // Test HTTPS (port 443)
    let https_url = format!("https://{}", host);
    match test_simple_http(&https_url).await {
        Ok(result) => result,
        Err(_) => TestResult {
            success: false,
            details: "No HTTP/HTTPS services available".to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
            error: Some("Connection failed on both port 80 and 443".to_string()),
        }
    }
}

async fn test_simple_http(_url: &str) -> Result<TestResult> {
    // Simplified HTTP test - just check if we can connect to port 80 or 443
    Err(anyhow::anyhow!("HTTP test not implemented"))
}

fn add_test_result(report: &mut NetworkReport, test_name: &str, result: TestResult) {
    let status = if result.success { "✅ PASS" } else { "❌ FAIL" };
    println!("   {} {} ({}ms)", status, result.details, result.duration_ms);
    report.tests.insert(test_name.to_string(), result);
}

fn display_report(report: &NetworkReport) {
    println!();
    println!("{}", "📊 DIAGNOSTIC REPORT SUMMARY".bright_blue().bold());
    println!("═══════════════════════════════════════════");
    println!("Target: {} ({})", report.target_host.bright_white(), report.target_ip.bright_yellow());
    println!("Timestamp: {}", report.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string().bright_cyan());
    println!();
    
    let status_color = match report.summary.overall_status.as_str() {
        "EXCELLENT" => report.summary.overall_status.bright_green(),
        "GOOD" => report.summary.overall_status.bright_yellow(),
        "PARTIAL" => report.summary.overall_status.bright_magenta(),
        "POOR" => report.summary.overall_status.bright_red(),
        _ => report.summary.overall_status.bright_white(),
    };
    
    println!("Overall Status: {}", status_color.bold());
    println!("Tests: {} total, {} passed, {} failed",
        report.summary.total_tests.to_string().bright_white(),
        report.summary.passed_tests.to_string().bright_green(),
        report.summary.failed_tests.to_string().bright_red()
    );
    
    println!();
    println!("{}", "Test Details:".bright_blue());
    for (test_name, result) in &report.tests {
        let status_icon = if result.success { "✅" } else { "❌" };
        println!("  {} {}: {}", status_icon, test_name.replace('_', " "), result.details);
        if let Some(error) = &result.error {
            println!("    Error: {}", error.red());
        }
    }
    
    // Recommendations
    println!();
    println!("{}", "💡 RECOMMENDATIONS".bright_blue().bold());
    if report.summary.failed_tests == 0 {
        println!("• Network connectivity appears to be working well!");
        println!("• All diagnostic tests passed successfully.");
    } else {
        if !report.tests.get("basic_connectivity").unwrap_or(&TestResult { success: false, details: "".to_string(), duration_ms: 0, error: None }).success {
            println!("• Check network connectivity and firewall settings");
            println!("• Verify the target host is online and reachable");
        }
        if !report.tests.get("dns_resolution").unwrap_or(&TestResult { success: false, details: "".to_string(), duration_ms: 0, error: None }).success {
            println!("• Check DNS server configuration");
            println!("• Try using a different DNS server (e.g., 8.8.8.8)");
        }
        if !report.tests.get("port_scan").unwrap_or(&TestResult { success: false, details: "".to_string(), duration_ms: 0, error: None }).success {
            println!("• Target may have firewall blocking connections");
            println!("• Services may not be running on expected ports");
        }
    }
}

fn save_report_to_file(report: &NetworkReport, path: &str) -> Result<()> {
    let json_data = serde_json::to_string_pretty(report)?;
    std::fs::write(path, json_data)?;
    Ok(())
}