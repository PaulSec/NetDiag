use std::time::Duration;

pub fn format_duration(duration: Duration) -> String {
    let total_micros = duration.as_micros();
    
    if total_micros < 1000 {
        format!("{}μs", total_micros)
    } else if total_micros < 1_000_000 {
        format!("{:.2}ms", duration.as_secs_f64() * 1000.0)
    } else if total_micros < 60_000_000 {
        format!("{:.2}s", duration.as_secs_f64())
    } else {
        let minutes = duration.as_secs() / 60;
        let seconds = duration.as_secs() % 60;
        format!("{}m{}s", minutes, seconds)
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}