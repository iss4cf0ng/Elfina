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

use crate::elf_arch::ElfArch;

pub fn disassemble(data: &[u8], arch: ElfArch, base_addr: u64, out_path: Option<&str>) -> Result<(), String> {
    let output = print_disasm(data, arch, base_addr)
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

fn print_disasm(data: &[u8], arch: ElfArch, base_addr: u64) -> Result<String, String> {
    let cs = make_capstone(arch)?;
    let insns = cs.disasm_all(data, base_addr).map_err(|e| format!("capstone disasm_all: {}", e))?;
 
    let mut out = String::with_capacity(insns.len() * 50);
 
    writeln!(out, "Disassembly ({}, {} bytes)", arch.to_str(), data.len()).unwrap();
    writeln!(out, "{}", "=".repeat(60)).unwrap();
 
    for insn in insns.iter() {
        let addr  = insn.address();
        let bytes = insn.bytes();
        let mnem  = insn.mnemonic().unwrap_or("???");
        let ops   = insn.op_str().unwrap_or("");
 
        let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
 
        writeln!(out, "  0x{:016x}:  {:<14}  {} {}", addr, hex, mnem, ops).unwrap();
    }
 
    writeln!(out, "\n  {} instructions disassembled", insns.len()).unwrap();
 
    Ok(out)
}