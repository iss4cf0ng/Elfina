//main.rs

mod entropy;
mod hexdump;
mod disasm;
mod elf_arch;
mod elf_loader;

use std::fs;
use std::process;

use crate::disasm::disassemble;
use crate::elf_arch::host_arch_name;
use crate::elf_loader::{
    elf_probe,
    elf_load,
    elf_execute,
    elf_unload,
    elf_print_info,
    elf_memfd_exec,
};
use hexdump::hexdump as do_hexdump;
use entropy::entropy_output;

const BANNER: &str = r#"
 (`-')  _                    _     <-. (`-')_ (`-')  _  
 ( OO).-/  <-.      <-.     (_)       \( OO) )(OO ).-/  
(,------.,--. )  (`-')-----.,-(`-'),--./ ,--/ / ,---.   
 |  .---'|  (`-')(OO|(_\---'| ( OO)|   \ |  | | \ /`.\  
(|  '--. |  |OO ) / |  '--. |  |  )|  . '|  |)'-'|_.' | 
 |  .--'(|  '__ | \_)  .--'(|  |_/ |  |\    |(|  .-.  | 
 |  `---.|     |'  `|  |_)  |  |'->|  | \   | |  | |  | 
 `------'`-----'    `--'    `--'   `--'  `--' `--' `--' 
"#;

const DESCRIPTION: &str = "Author: iss4cf0ng/ISSAC\nGitHub: https://github.com/iss4cf0ng/Elfina";

const COFFEE: &str = r#"
    (  )   (   )  )
     ) (   )  (  (
     ( )  (    ) )
     _____________
    <_____________> ___
    |             |/ _ \
    |               | | |
    |               |_| |
 ___|             |\___/
/    \___________/    \
\_____________________/
"#;

#[derive(Default)]
struct Opts {
    // funny
    coffee: bool,

    // execution
    force_memfd: bool,
    force_mmap: bool,
    info_only: bool,

    // analysis
    hexdump: bool,
    hexdump_out: Option<String>,
    disasm: bool,
    disasm_out: Option<String>,
    entropy: bool,
    entropy_out: Option<String>,
}

impl Opts {
    fn any_analysis(&self) -> bool {
        self.hexdump || self.hexdump_out.is_some() || self.disasm || self.disasm_out.is_some() || self.entropy || self.entropy_out.is_some()
    }
}

fn print_usage(app: &str) {
    eprintln!(
        "Usage: {app} [options] <elf_binary> [args...]\n\
         \n\
         Execution:\n\
         \t--memfd             execute via memfd_create + fexecve\n\
         \t--mmap              execute via manual mmap loader\n\
         \t--info              print ELF info, do not execute\n\
         \n\
         Analysis (can be combined):\n\
         \t--hexdump           hex dump entire file to stdout\n\
         \t--hexdump-out <f>   hex dump to file\n\
         \t--disasm            disassemble entire file to stdout\n\
         \t--disasm-out <f>    disassemble to file\n\
         \t--entropy           Shannon entropy report to stdout\n\
         \t--entropy-out <f>   entropy report to file\n\
         \n\
         Funny:\n\
         \t--coffee             Print a cup of coffee\n\
         \n\
         Host architecture: {host}",
        app  = app,
        host = host_arch_name(),
    );
}

fn parse_args(raw: &[String]) -> Result<(Opts, usize), String> {
    let mut opts = Opts::default();
    let mut i = 1usize;

    while i < raw.len() {
        match raw[i].as_str() {
            "--coffee" => {
                opts.coffee = true;
            }

            "--memfd" => {
                opts.force_memfd = true;
            }
            "--mmap" => {
                opts.force_mmap = true;
            }
            "--info" => {
                opts.info_only = true;
            }

            "--hexdump" => { 
                opts.hexdump = true; 
            }
            "--hexdump-out" => {
                i += 1;
                let path = raw.get(i).ok_or("--hexdump-out requires a filename argument")?;
                opts.hexdump_out = Some(path.clone());
            }

            "--disasm" => {
                opts.disasm = true;
            }
            "--disasm-out" => {
                i += 1;
                let path = raw.get(i).ok_or("--disasm-out requires a filename argument")?;
                opts.disasm_out = Some(path.clone());
            }

            "--entropy" => {
                opts.entropy = true;
            }
            "--entropy-out" => {
                i += 1;
                let path = raw.get(i).ok_or("--entropy-ut requires a filename argument")?;
                opts.entropy_out = Some(path.clone());
            }

            arg if arg.starts_with("--") => {
                return Err(format!("Unknown flag: {}", arg));
            }

            _ => break,
        }

        i += 1;
    }

    Ok((opts, i))
}

fn main() {
    println!("{}", BANNER);       // print banner
    println!("{}", DESCRIPTION);  // print discription
    println!("");                 // separation line

    let args: Vec<String> = std::env::args().collect();
    let env_vars: Vec<String> = std::env::vars().map(|(k, v)| format!("{}={}", k, v)).collect();

    // validate argument
    if args.len() < 2 {
        print_usage(&args[0]);
        process::exit(1);
    }

    let (opts, path_idx) = match parse_args(&args) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}", e);
            print_usage(&args[0]);
            process::exit(1);
        }
    };

    // the only exception: print a cupt of coffee!
    if opts.coffee {
        println!("{}", COFFEE);
        process::exit(1);
    }

    if path_idx >= args.len() {
        print_usage(&args[0]);
        process::exit(1);
    }

    let elf_path = &args[path_idx]; // file path of the ELF file
    let target_args: Vec<String> = args[path_idx..].to_vec(); // arguments for in-memory execution

    // read ELF file bytes
    let elf_data = match  fs::read(elf_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to read '{}': {}", elf_path, e);
            process::exit(1);
        }
    };

    // loader ELF into memory
    let mut loader = match elf_probe(&elf_data) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("elf_probe failed: {}", e);
            process::exit(1);
        }
    };

    // print summary
    println!(
        "arch={:<12} class=ELF{}\t{}\t{}",
        loader.arch.to_str(),
        if loader.is_64bit { 64 } else { 32 },
        if loader.is_pie { "PIE" } else { "ET_EXEC" },
        if loader.interp_offset != 0 { "dynamic" } else { "static" },
    );

    // hexdump
    if opts.hexdump || opts.hexdump_out.is_some() {
        println!();
        if let Err(e) = do_hexdump(&elf_data, 0, opts.hexdump_out.as_deref()) {
            eprintln!("hexdump error: {}", e);
        }
    }

    
    if opts.entropy || opts.entropy_out.is_some() {
        println!();
        if let Err(e) = entropy_output(&elf_data, opts.entropy_out.as_deref()) {
            eprintln!("print_entropy error: {}", e);
        }
    }

    if opts.disasm || opts.disasm_out.is_some() {
        println!();
        if let Err(e) = disassemble(&elf_data, loader.arch, opts.disasm_out.as_deref()) {
            eprintln!("disasm error: {}", e);
        }
    }

    if opts.any_analysis() && !opts.info_only && !opts.force_memfd && !opts.force_mmap {
        return;
    }

    if opts.info_only {
        match elf_load(&elf_data) {
            Ok(loader) => {
                elf_print_info(&loader);
                let mut loader = loader;
                elf_unload(&mut loader);
            }

            Err(e) => eprintln!("elf_load failed: {}", e),
        }
        
        return;
    }

    let use_memfd = opts.force_memfd || (!opts.force_mmap && loader.interp_offset != 0);
    if use_memfd {
        println!("Executing via memfd_create + fexecve");
        if let Err(e) = elf_memfd_exec(&elf_data, &target_args, &env_vars) {
            eprintln!("elf_memfd_exec failed: {}", e);
        }

        process::exit(1);
    }

    if !loader.arch.is_native() {
        eprintln!("Binary is {} build host is {}. Use QEMU for cross-arch execution.", loader.arch.to_str(), host_arch_name());
        process::exit(1);
    }

    loader = match  elf_load(&elf_data) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("elf_load failed: {}", e);
            process::exit(1);
        }
    };

    drop(elf_data);

    unsafe {
        elf_execute(&loader, &target_args, &env_vars)
    };
    
}
