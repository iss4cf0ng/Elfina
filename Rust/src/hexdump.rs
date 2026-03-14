//hexdump.rs

use std::fmt::Write as FmtWrite;
use std::{fs, path};
use std::io::{self, Write};

const COLS: usize = 16;
const GROUP: usize = 8;

pub fn hexdump(data: &[u8], base_offset: u64, out_path: Option<&str>) -> Result<(), String> {
    let output = render_hexdump(data, base_offset);
    match  out_path {
        Some(path) => {
            fs::write(path, &output).map_err(|e| format!("Failed to write hexdump to '{}': {}", path, e))?;
            println!("Hexdump written to '{}' ({} bytes)", path, output.len());
        }

        None => {
            io::stdout().write_all(output.as_bytes()).map_err(|e| format!("stdout write failed: {}", e))?;
        }
    }

    Ok(())
}

fn render_hexdump(data: &[u8], base_offset: u64) -> String {
    // Pre-allocate: each line is ~78 chars
    let mut out = String::with_capacity((data.len() / COLS + 1) * 80);
 
    for (line_idx, chunk) in data.chunks(COLS).enumerate() {
        let offset = base_offset + (line_idx * COLS) as u64;
 
        write!(out, "{:08x}  ", offset).unwrap();
 
        for (i, byte) in chunk.iter().enumerate() {
            if i == GROUP { out.push(' '); }  // mid-line gap
            write!(out, "{:02x} ", byte).unwrap();
        }
 
        let missing = COLS - chunk.len();
        if missing > 0 {
            let pad_chars = missing * 3 + if chunk.len() <= GROUP { 1 } else { 0 };
            for _ in 0..pad_chars { out.push(' '); }
        }
 
        out.push(' ');
        out.push('|');
        for &byte in chunk {
            out.push(if (0x20..=0x7e).contains(&byte) { byte as char } else { '.' });
        }
        out.push('|');
        out.push('\n');
    }
 
    out
}