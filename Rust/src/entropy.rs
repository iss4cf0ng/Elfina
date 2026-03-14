use std::fmt::Write as FmtWrite;
use std::fs;
use std::io::{self, Write};


const BLOCK_SIZE: usize = 256;
const BAR_WIDTH: usize = 32;
const TOP_BYTES: usize = 16;

pub fn shannon_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let mut freq = [0u64; 256];
    for &b in data {
        freq[b as usize] += 1;
    }

    let len = data.len() as f64;
    let mut entropy = 0.0f64;

    for &count in &freq {
        if count == 0 {
            continue;
        }

        let p = count as f64 / len;
        entropy -= p * p.log2();
    }

    return entropy;
}

fn assess(e: f64) -> &'static str {
    match e as u32 {
        0 => "All identical bytes (empty / zero-filled section)",
        1 => "Highly repetitive data",
        2 | 3 => "Low entropy — structured data or sparse code",
        4 | 5 => "Normal code or data (compressible)",
        6 => "Compressed data or dense code",
        7  => "High entropy — likely compressed or encrypted",
        _ => "Very high entropy — packed, encrypted, or random",
    }
}

pub fn entropy_output(data: &[u8], out_path: Option<&str>) -> Result<(), String> {
    let report = print_entropy(data);
    match out_path {
        Some(path) => {
            fs::write(path, &report)
                .map_err(|e| format!("Failed to write entropy report to '{}': {}", path, e))?;
            println!("Entropy report written to '{}' ({} bytes)", path, report.len());
        }
        None => {
            io::stdout()
                .write_all(report.as_bytes())
                .map_err(|e| format!("stdout write failed: {}", e))?;
        }
    }
 
    Ok(())
}

pub fn print_entropy(data: &[u8]) -> String {
    let mut out = String::with_capacity(4096);
    let overall = shannon_entropy(data);

    out.push_str("Entropy Analysis\n");
    out.push_str("================\n");
    writeln!(out, "Total bytes : {}", data.len()).unwrap();
    writeln!(out, "Entropy     : {:.4} / 8.0000", overall).unwrap();
    writeln!(out, "Assessment  : {}", assess(overall)).unwrap();

    out.push_str("\nByte frequency histogram (top 16 most common):\n");
 
    let mut freq = [0u64; 256];
    for &b in data { freq[b as usize] += 1; }
 
    let mut indexed: Vec<(usize, u64)> = freq.iter().copied().enumerate().collect();
    indexed.sort_by(|a, b| b.1.cmp(&a.1));
 
    let max_count = indexed[0].1.max(1) as f64;
    let total     = data.len() as f64;
 
    for &(byte_val, count) in indexed.iter().take(TOP_BYTES) {
        if count == 0 { break; }
        let pct      = count as f64 / total * 100.0;
        let filled   = ((count as f64 / max_count) * BAR_WIDTH as f64) as usize;
        let empty    = BAR_WIDTH - filled;
        let bar: String = "█".repeat(filled) + &"░".repeat(empty);
        writeln!(out, "  0x{:02x} [{}] {:5.1}%", byte_val, bar, pct).unwrap();
    }
 
    out.push_str("\nBlock entropy (256-byte blocks):\n");
    out.push_str("  offset     entropy  visual\n");
 
    for (block_idx, chunk) in data.chunks(BLOCK_SIZE).enumerate() {
        let offset  = block_idx * BLOCK_SIZE;
        let e       = shannon_entropy(chunk);
        // Scale bar: 8.0 = full bar
        let filled  = ((e / 8.0) * BAR_WIDTH as f64) as usize;
        let empty   = BAR_WIDTH - filled;
        let bar: String = "█".repeat(filled) + &"░".repeat(empty);
 
        let flag = if e > 7.0 { " !" } else { "  " };
        writeln!(out, "  0x{:06x}   {:.4}   [{}]{}", offset, e, bar, flag).unwrap();
    }
 
    out.push_str("\n  ! = high entropy block (>7.0), possibly encrypted or packed\n");
 
    out
}