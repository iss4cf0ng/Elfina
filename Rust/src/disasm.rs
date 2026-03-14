use std::fs;
use std::io::{self, Write};
use std::fmt::Write as FmtWrite;

use capstone::prelude::*;
use capstone::arch::{
    x86::ArchMode as X86Mode,
    arm::ArchMode as ArmMode,
    arm64::ArchMode as Arm64Mode,
    riscv::ArchMode as RiscVMode,
};

use crate::elf_arch::{
    ElfArch,
    elf_parse_header, elf_get_phdr,
    PT_LOAD,
};

struct ExecSegment {
    vaddr: u64,
    data: Vec<u8>,
}

const PF_X: u32 = 0x1;

pub fn disassemble(data: &[u8], arch: ElfArch, out_path: Option<&str>) -> Result<(), String> {
    let output = print_disasm(data, arch)
        .map_err(|e| format!("Disassembly failed: {}", e))?;
 
    match out_path {
        Some(path) => {
            fs::write(path, &output).map_err(|e| format!("Failed to write disasm to '{}': {}", path, e))?;
            println!("Disassembly written to '{}' ({} bytes)", path, output.len());
        }
        None => {
            io::stdout().write_all(output.as_bytes()).map_err(|e| format!("stdout write failed: {}", e))?;
        }
    }
 
    Ok(())
}

fn make_capstone(arch: ElfArch) -> Result<Capstone, String> {
    match arch {
        ElfArch::X86_64 => {
            Capstone::new()
                .x86()
                .mode(X86Mode::Mode64)
                .detail(true)
                .build()
                .map_err(|e| format!("capstone x86-64 init: {}", e))
        }
        ElfArch::X86 => {
            Capstone::new()
                .x86()
                .mode(X86Mode::Mode32)
                .detail(true)
                .build()
                .map_err(|e| format!("capstone x86 init: {}", e))
        }
        ElfArch::Arm64 => {
            Capstone::new()
                .arm64()
                .mode(Arm64Mode::Arm)
                .detail(true)
                .build()
                .map_err(|e| format!("capstone arm64 init: {}", e))
        }
        ElfArch::Arm32 => {
            // ARM32 with Thumb-2 interworking support
            Capstone::new()
                .arm()
                .mode(ArmMode::Arm)
                .detail(true)
                .build()
                .map_err(|e| format!("capstone arm32 init: {}", e))
        }
        ElfArch::RiscV64 => {
            Capstone::new()
                .riscv()
                .mode(RiscVMode::RiscV64)
                .detail(true)
                .build()
                .map_err(|e| format!("capstone riscv64 init: {}", e))
        }
        ElfArch::Unknown => {
            Err("Cannot disassemble: unknown architecture".to_string())
        }
    }
}

fn collect_exec_segments(data: &[u8]) -> Vec<ExecSegment> {
    let hdr = match elf_parse_header(data) {
        Some(h) => h,
        None    => return Vec::new(),
    };
 
    let mut segments = Vec::new();
 
    for i in 0..hdr.e_phnum {
        let ph = match elf_get_phdr(data, &hdr, i) {
            Some(p) if p.p_type == PT_LOAD => p,
            _                              => continue,
        };
 
        // Skip non-executable segments (data, BSS, read-only)
        if ph.p_flags & PF_X == 0 {
            continue;
        }
 
        let offset     = ph.p_offset as usize;
        let file_size  = ph.p_filesize as usize;
 
        if offset + file_size > data.len() {
            continue;
        }
 
        segments.push(ExecSegment {
            vaddr: ph.p_vaddr,
            data : data[offset .. offset + file_size].to_vec(),
        });
    }
 
    return segments;
}

fn print_disasm(data: &[u8], arch: ElfArch) -> Result<String, String> {
    let cs       = make_capstone(arch)?;
    let segments = collect_exec_segments(data);
 
    if segments.is_empty() {
        return Err("No executable PT_LOAD segments found".to_string());
    }
 
    let mut out         = String::with_capacity(segments.len() * 4096);
    let mut total_insns = 0usize;
 
    writeln!(out, "Disassembly ({}, {} executable segment(s))",
        arch.to_str(),
        segments.len()
    ).unwrap();
    writeln!(out, "{}", "=".repeat(60)).unwrap();
 
    for seg in &segments {
        writeln!(out, "\n; segment @ 0x{:x}  ({} bytes)",
            seg.vaddr,
            seg.data.len()
        ).unwrap();
        writeln!(out, "{}", "-".repeat(60)).unwrap();
 
        let insns = cs.disasm_all(&seg.data, seg.vaddr)
            .map_err(|e| format!("capstone disasm_all: {}", e))?;
 
        for insn in insns.iter() {
            let addr  = insn.address();
            let bytes = insn.bytes();
            let mnem  = insn.mnemonic().unwrap_or("???");
            let ops   = insn.op_str().unwrap_or("");
 
            // hex bytes column — pad to 15 chars so columns align
            let hex: String = bytes.iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join(" ");
 
            writeln!(out, "  0x{:016x}:  {:<15}  {} {}",
                addr, hex, mnem, ops
            ).unwrap();
        }
 
        total_insns += insns.len();
    }
 
    writeln!(out, "\n{}", "=".repeat(60)).unwrap();
    writeln!(out, "  {} instructions disassembled across {} segment(s)",
        total_insns,
        segments.len()
    ).unwrap();
 
    return Ok(out);
}