use anyhow::Result;
use colored::*;
use std::collections::HashMap;
use std::time::Duration;

use crate::utils::format::format_bytes;

pub async fn http_command(
    url: String,
    timeout: Duration,
    follow_redirects: bool,
    show_headers: bool,
) -> Result<()> {
    println!("{} {}", "🌐 HTTP TEST".bright_green().bold(), url.bright_white().bold());

    // Simple HTTP client implementation using tokio
    let client = create_http_client(timeout, follow_redirects)?;
    
    println!("Testing HTTP connectivity...");
    println!();

    match perform_http_request(&client, &url, show_headers).await {
        Ok((status_code, headers, response_time)) => {
            // Status code with color
            let status_color = match status_code {
                200..=299 => status_code.to_string().bright_green(),
                300..=399 => status_code.to_string().bright_yellow(),
                400..=499 => status_code.to_string().bright_red(),
                500..=599 => status_code.to_string().bright_magenta(),
                _ => status_code.to_string().bright_white(),
            };

            println!("{} HTTP Response", "✅".green());
            println!("  Status Code: {}", status_color.bold());
            println!("  Response Time: {}ms", response_time.to_string().bright_cyan());
            if let Some(content_length_str) = headers.get("Content-Length") {
                if let Ok(content_length) = content_length_str.parse::<u64>() {
                    println!(
                        "  Content Length: {}",
                        format_bytes(content_length).bright_magenta()
                    );
                }
            }
            
            if show_headers && !headers.is_empty() {
                println!();
                println!("{} Response Headers:", "📋".bright_blue());
                for (name, value) in &headers {
                    println!("  {}: {}", name.bright_yellow(), value.bright_white());
                }
            }

            // Provide status code interpretation
            println!();
            match status_code {
                200..=299 => println!("{} Connection successful!", "🎉".green()),
                300..=399 => println!("{} Redirection response", "🔄".yellow()),
                400..=499 => println!("{} Client error", "❌".red()),
                500..=599 => println!("{} Server error", "💥".red()),
                _ => println!("{} Unexpected response code", "❓".yellow()),
            }
            if (300..=399).contains(&status_code) && !follow_redirects {
                println!(
                    "{} Use --follow-redirects to automatically follow Location headers",
                    "[hint]".bright_blue()
                );
            }
        }
        Err(e) => {
            println!("{} HTTP request failed: {}", "❌".red(), e.to_string().red());
            
            println!();
            println!("{} This could indicate:", "💡".yellow());
            println!("  • Host is unreachable");
            println!("  • Port is closed or filtered");
            println!("  • SSL/TLS certificate issues");
            println!("  • Firewall blocking the connection");
            println!("  • DNS resolution problems");
        }
    }

    Ok(())
}

struct SimpleHttpClient {
    timeout: Duration,
    follow_redirects: bool,
}

fn create_http_client(timeout: Duration, follow_redirects: bool) -> Result<SimpleHttpClient> {
    Ok(SimpleHttpClient {
        timeout,
        follow_redirects,
    })
}

async fn perform_http_request(
    client: &SimpleHttpClient,
    url: &str,
    _show_headers: bool,
) -> Result<(u16, HashMap<String, String>, u64)> {
    const MAX_REDIRECTS: usize = 5;

    let mut current_url = url.to_string();
    let mut redirects_followed = 0usize;

    loop {
        let (status_code, headers, response_time, redirect_target) =
            send_http_request_once(client, &current_url).await?;

        if (300..=399).contains(&status_code)
            && client.follow_redirects
            && redirects_followed < MAX_REDIRECTS
        {
            if let Some(location) = redirect_target {
                let next_url = resolve_redirect(&current_url, &location)?;
                println!(
                    "{} Redirecting to {}",
                    "[->]".bright_yellow(),
                    next_url.bright_white()
                );
                current_url = next_url;
                redirects_followed += 1;
                continue;
            }
        }

        if redirects_followed == MAX_REDIRECTS && client.follow_redirects {
            println!(
                "{} Maximum redirect depth ({}) reached",
                "[warn]".bright_yellow(),
                MAX_REDIRECTS
            );
        }

        return Ok((status_code, headers, response_time));
    }
}

async fn send_http_request_once(
    client: &SimpleHttpClient,
    url: &str,
) -> Result<(u16, HashMap<String, String>, u64, Option<String>)> {
    use std::time::Instant;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;
    use tokio::time;

    let start_time = Instant::now();
    
    // Parse URL
    let parsed_url = parse_url(url)?;
    let host = parsed_url.host;
    let port = parsed_url.port;
    let path = parsed_url.path;
    let is_https = parsed_url.is_https;

    // Connect to server
    let addr = format!("{}:{}", host, port);
    let mut stream = time::timeout(client.timeout, TcpStream::connect(&addr))
        .await
        .map_err(|_| anyhow::anyhow!("Connection timeout"))?
        .map_err(|e| anyhow::anyhow!("Connection failed: {}", e))?;

    // For HTTPS, we'd need to wrap with TLS, but for simplicity we'll just do HTTP
    if is_https {
        return Err(anyhow::anyhow!("HTTPS not implemented in this simple client. Use HTTP instead."));
    }

    // Send HTTP request
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: netdiag/0.1.0\r\nConnection: close\r\n\r\n",
        path, host
    );

    stream.write_all(request.as_bytes()).await?;

    // Read response
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).await?;
    
    let response = String::from_utf8_lossy(&buffer);
    let response_time = start_time.elapsed().as_millis() as u64;

    // Parse response
    let lines: Vec<&str> = response.lines().collect();
    if lines.is_empty() {
        return Err(anyhow::anyhow!("Empty response"));
    }

    // Parse status line
    let status_line = lines[0];
    let status_parts: Vec<&str> = status_line.split_whitespace().collect();
    if status_parts.len() < 2 {
        return Err(anyhow::anyhow!("Invalid status line"));
    }

    let status_code: u16 = status_parts[1].parse()
        .map_err(|_| anyhow::anyhow!("Invalid status code"))?;

    // Parse headers
    let mut headers = HashMap::new();
    let mut redirect_target = None;
    for line in lines.iter().skip(1) {
        if line.is_empty() {
            break;
        }
        if let Some(colon_pos) = line.find(':') {
            let name = line[..colon_pos].trim().to_string();
            let value = line[colon_pos + 1..].trim().to_string();
            if name.eq_ignore_ascii_case("location") {
                redirect_target = Some(value.clone());
            }
            headers.insert(name, value);
        }
    }

    Ok((status_code, headers, response_time, redirect_target))
}

fn resolve_redirect(current_url: &str, location: &str) -> Result<String> {
    if location.starts_with("http://") || location.starts_with("https://") {
        return Ok(location.to_string());
    }

    let base = parse_url(current_url)?;
    let scheme = if base.is_https { "https" } else { "http" };
    let authority = if (base.is_https && base.port == 443) || (!base.is_https && base.port == 80) {
        base.host.clone()
    } else {
        format!("{}:{}", base.host, base.port)
    };

    let new_path = if location.starts_with('/') {
        location.to_string()
    } else {
        let trimmed = base.path.trim_end_matches('/');
        if trimmed.is_empty() {
            format!("/{}", location)
        } else {
            format!("{}/{}", trimmed, location)
        }
    };

    Ok(format!("{}://{}{}", scheme, authority, new_path))
}

struct ParsedUrl {
    host: String,
    port: u16,
    path: String,
    is_https: bool,
}

fn parse_url(url: &str) -> Result<ParsedUrl> {
    let url = url.trim();
    
    let (is_https, url_without_scheme) = if url.starts_with("https://") {
        (true, &url[8..])
    } else if url.starts_with("http://") {
        (false, &url[7..])
    } else {
        (false, url) // Assume HTTP if no scheme
    };

    let default_port = if is_https { 443 } else { 80 };

    let (host_port, path) = if let Some(slash_pos) = url_without_scheme.find('/') {
        (&url_without_scheme[..slash_pos], &url_without_scheme[slash_pos..])
    } else {
        (url_without_scheme, "/")
    };

    let (host, port) = if let Some(colon_pos) = host_port.find(':') {
        let host = host_port[..colon_pos].to_string();
        let port: u16 = host_port[colon_pos + 1..].parse()
            .map_err(|_| anyhow::anyhow!("Invalid port number"))?;
        (host, port)
    } else {
        (host_port.to_string(), default_port)
    };

    Ok(ParsedUrl {
        host,
        port,
        path: path.to_string(),
        is_https,
    })
}