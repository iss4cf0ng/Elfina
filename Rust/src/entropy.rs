use std::fmt::Write as FmtWrite;
use std::fs;
use std::io::{self, Write};


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

pub fn entropy_report(data: &[u8], out_path: Option<&str>) -> Result<(), String> {
    

    return Ok(());
}