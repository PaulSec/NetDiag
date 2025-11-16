# NetDiag - Network Diagnostic Tool

A comprehensive command-line network diagnostic tool built in Rust for troubleshooting network connectivity issues, firewall rules, and identifying network bottlenecks.

## Features

- **Ping Testing** - Test basic connectivity to hosts with customizable packet count and timeout
- **Port Scanning** - Scan single ports, port ranges, or common service ports
- **DNS Resolution** - Perform DNS lookups with custom servers and record types
- **HTTP Testing** - Test HTTP/HTTPS connectivity with header inspection
- **Network Tracing** - Trace network paths to destinations (traceroute-like functionality)  
- **Connection Testing** - Test specific TCP/UDP connections
- **Comprehensive Reports** - Generate detailed network diagnostic reports

## Installation

### From Source

```bash
git clone <repository-url>
cd rust_network_tool
cargo build --release
```

The binary will be available at `target/release/netdiag`

### Cross-Platform Support

This tool works on:
- ✅ Windows
- ✅ Linux  
- ✅ macOS

## Usage

### Basic Commands

```bash
# Test basic connectivity (ping-like)
netdiag ping google.com

# Scan common ports
netdiag scan google.com

# Scan specific port range
netdiag scan google.com -p 80-443

# Test specific port connection
netdiag connect google.com 80

# Perform DNS lookup
netdiag dns google.com

# Test HTTP connectivity
netdiag http http://google.com

# Trace route to destination
netdiag trace google.com

# Generate comprehensive report
netdiag report google.com
```

### Advanced Examples

```bash
# Ping with custom settings
netdiag ping google.com -c 10 -t 3 -s 128

# Scan with high concurrency
netdiag scan 192.168.1.1 -p 1-65535 -c 500 -t 1000

# DNS lookup with custom server
netdiag dns google.com -s 8.8.8.8 -t MX

# HTTP test with headers
netdiag http https://api.github.com -H -f

# UDP connection test
netdiag connect 8.8.8.8 53 -u

# Detailed network report with port scan
netdiag report google.com --detailed-scan -o report.json
```

## Command Reference

### `ping` - Basic Connectivity Test
```
netdiag ping <HOST> [OPTIONS]

OPTIONS:
    -c, --count <COUNT>        Number of packets to send [default: 4]
    -t, --timeout <TIMEOUT>    Timeout in seconds [default: 5]
    -s, --size <SIZE>          Packet size in bytes [default: 64]
```

### `scan` - Port Scanner
```
netdiag scan <HOST> [OPTIONS]

OPTIONS:
    -p, --ports <PORTS>        Port range (e.g., 80, 80-443, 22,80,443) [default: 1-1000]
    -t, --timeout <TIMEOUT>    Timeout in milliseconds [default: 3000]
    -c, --concurrency <CONC>   Number of concurrent connections [default: 100]
```

### `dns` - DNS Resolution
```
netdiag dns <DOMAIN> [OPTIONS]

OPTIONS:
    -s, --server <SERVER>      DNS server to use (optional)
    -t, --record-type <TYPE>   Record type (A, AAAA, MX, NS, TXT, etc.) [default: A]
```

### `http` - HTTP Connectivity Test
```
netdiag http <URL> [OPTIONS]

OPTIONS:
    -t, --timeout <TIMEOUT>    Request timeout in seconds [default: 10]
    -f, --follow-redirects     Follow redirects
    -H, --show-headers         Show response headers
```

### `trace` - Network Path Tracing
```
netdiag trace <HOST> [OPTIONS]

OPTIONS:
    -m, --max-hops <HOPS>      Maximum number of hops [default: 30]
    -t, --timeout <TIMEOUT>    Timeout per hop in seconds [default: 5]
```

### `connect` - Connection Test
```
netdiag connect <HOST> <PORT> [OPTIONS]

OPTIONS:
    -t, --timeout <TIMEOUT>    Connection timeout in seconds [default: 5]
    -u, --udp                  Test UDP instead of TCP
```

### `report` - Generate Diagnostic Report
```
netdiag report <HOST> [OPTIONS]

OPTIONS:
    -o, --output <FILE>        Output file path (optional, defaults to stdout)
        --detailed-scan        Include detailed port scan
```

## Use Cases

### Troubleshooting Network Issues
- Test if a host is reachable
- Identify which ports are accessible
- Check DNS resolution problems
- Verify HTTP/HTTPS services are working

### Security Assessment  
- Discover open ports on a target system
- Test firewall rules and configurations
- Verify service availability

### Network Performance Analysis
- Measure connection latency and response times
- Identify network bottlenecks
- Trace network paths and routing

### Development & DevOps
- Verify service deployments are accessible
- Test API endpoint connectivity
- Validate network configurations
- Automate network health checks

## Output Examples

### Ping Output
```
🏓 PING google.com
Resolved google.com to 142.251.46.110

✓ Reply from 142.251.46.110: seq=0 time=12.34ms
✓ Reply from 142.251.46.110: seq=1 time=11.56ms
✓ Reply from 142.251.46.110: seq=2 time=13.22ms
✓ Reply from 142.251.46.110: seq=3 time=12.89ms

📊 PING STATISTICS
Packets: Sent = 4, Received = 4, Lost = 0 (0.0%)
Round-trip times: min = 11.56ms, max = 13.22ms, avg = 12.50ms
```

### Port Scan Output
```
🔍 PORT SCAN google.com
Resolved google.com to 142.251.46.110
Scanning 1000 ports on 142.251.46.110

✅ 2 open ports found:

  80/tcp  OPEN (HTTP)
  443/tcp OPEN (HTTPS)

📊 Scanned 1000 ports in total
```

## Building from Source

### Prerequisites
- Rust 1.70 or later
- Cargo package manager

### Build Steps
```bash
# Clone the repository
git clone <repository-url>
cd rust_network_tool

# Build in release mode
cargo build --release

# Run tests
cargo test

# Install to system PATH (optional)
cargo install --path .
```

## Limitations

- **ICMP Ping**: Raw ICMP sockets require administrator/root privileges on most systems. The tool falls back to TCP connectivity testing when ICMP is not available.
- **Traceroute**: The current implementation is simplified. A full traceroute implementation would require raw socket capabilities.
- **HTTPS**: The simple HTTP client doesn't support TLS. Use dedicated tools for comprehensive HTTPS testing.

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [clap](https://github.com/clap-rs/clap) for CLI parsing
- Uses [tokio](https://github.com/tokio-rs/tokio) for async networking
- DNS resolution powered by [trust-dns](https://github.com/bluejekyll/trust-dns)