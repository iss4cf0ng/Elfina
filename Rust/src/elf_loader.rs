//elf_loader.rs

use std::ptr;
use libc::{
    close, fexecve, mmap, mprotect, munmap, write,
    MAP_ANON, MAP_ANONYMOUS, MAP_FAILED, MAP_FIXED, MAP_GROWSDOWN, MAP_PRIVATE, PROT_NONE, PROT_READ, PROT_WRITE,
};

use crate::elf_arch::{
    ElfArch, ElfHeader, ElfPhdr,
    elf_get_phdr, elf_parse_header, prot_from_flags, host_arch_name,
    ET_DYN, PT_LOAD, PT_INTERP, ELF_DEFAULT_STACK_SIZE,
};

const PAGE_SIZE: usize = 4096;

#[inline]
fn page_down(x: usize) -> usize {
    return x & !(PAGE_SIZE - 1);
}

#[inline]
fn page_up(x: usize) -> usize {
    return page_down(x + PAGE_SIZE - 1);
}

#[derive(Debug)]
pub struct ElfLoader {
    pub arch         : ElfArch,   // architecture
    pub is_64bit     : bool,      // is 64-bit
    pub is_pie       : bool,      // is Position-Independent Executable (PIE, for ASLR)
    pub interp_offset: usize,     // interpreter offset
    pub base         : *mut u8,   // base address
    pub map_size     : usize,     // map size
    pub load_bias    : usize,     // bias
    pub entry        : *mut u8,   // entry point
    pub stack_base   : *mut u8,   // base address of stack
    pub stack_size   : usize,     // stack size

    // stored for auxv AT_PHDR/AT_PHENT/AT_PHNUM
    // libc _start (musl, glibc) reads these to find the program headers
    pub phdr_offset  : usize,     // e_phoff: file offset of program header table
    pub phdr_entsize : usize,     // e_phentsize: size of one program header entry
    pub phdr_num     : usize,     // e_phnum: number of program header entries
}

impl Default for ElfLoader {
    fn default() -> Self {
        Self {
            arch         : ElfArch::Unknown,
            is_64bit     : false,
            is_pie       : false,
            interp_offset: 0,
            base         : ptr::null_mut(),
            map_size     : 0,
            load_bias    : 0,
            entry        : ptr::null_mut(),
            stack_base   : ptr::null_mut(),
            stack_size   : 0,
            phdr_offset  : 0,
            phdr_entsize : 0,
            phdr_num     : 0,
        }
    }
}

// probe ELF
pub fn elf_probe(data: &[u8]) -> Result<ElfLoader, String> {
    let hdr = elf_parse_header(data).ok_or_else(|| "Invalid ELF file".to_string())?;
    let mut loader = ElfLoader::default();

    loader.arch     = hdr.arch;
    loader.is_64bit = hdr.is_64bit;
    loader.is_pie   = hdr.e_type == ET_DYN;

    for i in 0..hdr.e_phnum {
        if let Some(ph) = elf_get_phdr(data, &hdr, i) {
            if ph.p_type == PT_INTERP {
                loader.interp_offset = ph.p_offset as usize;
                break;
            }
        }
    }

    return Ok(loader);
}

// read interpreter path
fn interp_path(data: &[u8], offset: usize) -> String {
    if offset >= data.len() {
        return "Invalid".to_string();
    }

    let slice = &data[offset..];
    let p = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());

    return String::from_utf8_lossy(&slice[..p]).to_string();
}

fn map_segment(data: &[u8], ph: &ElfPhdr, bias: usize) -> Result<(), String> {
    let seg_start = page_down(ph.p_vaddr as usize + bias);
    let seg_end   = page_up(ph.p_vaddr as usize + bias + ph.p_memsize as usize);
    let seg_len   = seg_end - seg_start;

    let mem = unsafe {
        mmap(
            seg_start as *mut libc::c_void,
            seg_len,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANON | MAP_FIXED,
            -1,
            0,
        )
    };

    if MAP_FAILED == mem {
        return Err(format!("mmap segment failed: {}", std::io::Error::last_os_error()));
    }

    let dest       = (ph.p_vaddr as usize + bias) as *mut u8;
    let src_offset = ph.p_offset as usize;
    let copy_bytes = (ph.p_filesize as usize).min(data.len() - src_offset);

    unsafe {
        ptr::copy_nonoverlapping(data.as_ptr().add(src_offset), dest, copy_bytes);
    }

    // Zero the BSS tail (p_memsize > p_filesize region)
    if ph.p_memsize > ph.p_filesize {
        let bss_start = (ph.p_vaddr as usize + bias + ph.p_filesize as usize) as *mut u8;
        let bss_len   = (ph.p_memsize - ph.p_filesize) as usize;
        unsafe { ptr::write_bytes(bss_start, 0, bss_len); }
    }

    let prot = prot_from_flags(ph.p_flags);
    if unsafe { mprotect(seg_start as *mut libc::c_void, seg_len, prot) } != 0 {
        return Err(format!("mprotect failed: {}", std::io::Error::last_os_error()));
    }

    return Ok(());
}

pub fn elf_load(data: &[u8]) -> Result<ElfLoader, String> {
    let mut loader = elf_probe(data)?;

    if !loader.arch.is_native() {
        return Err(format!("Cross-arch: binary is {} host is {}", loader.arch.to_str(), host_arch_name()));
    }

    if loader.interp_offset != 0 {
        let interp = interp_path(data, loader.interp_offset);
        eprintln!("Warning: Binary requires dynamic linker '{}'.\nUse elf_memfd_exec() for dynamic binary.", interp);
    }

    let hdr = elf_parse_header(data).unwrap();

    // Store program header metadata for auxv AT_PHDR/AT_PHENT/AT_PHNUM.
    // libc's _start reads these to locate the program header table.
    loader.phdr_offset  = hdr.e_phoffset  as usize;
    loader.phdr_entsize = hdr.e_phentsize as usize;
    loader.phdr_num     = hdr.e_phnum     as usize;

    let mut vmin = usize::MAX;
    let mut vmax = 0usize;

    for i in 0..hdr.e_phnum {
        let ph = match elf_get_phdr(data, &hdr, i) {
            Some(p) if p.p_type == PT_LOAD => p,
            _ => continue,
        };

        let start = page_down(ph.p_vaddr as usize);
        let end   = page_up(ph.p_vaddr as usize + ph.p_memsize as usize);

        if start < vmin { vmin = start; }
        if end   > vmax { vmax = end;   }
    }

    if usize::MAX == vmin {
        return Err("No PT_LOAD segments found".to_string());
    }

    let map_size = vmax - vmin;
    let is_pie   = loader.is_pie;

    // ---- STEP 1: allocate stack FIRST ----
    // Stack must be allocated before reserving the ELF virtual range.
    // If we reserve ELF range first, the kernel may place the stack inside
    // that reservation (especially for PIE where the kernel picks the base freely).
    // Stack overlapping with code/data = corrupted sp = segfault.
    let stack = unsafe {
        mmap(
            ptr::null_mut(),
            ELF_DEFAULT_STACK_SIZE,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANONYMOUS | MAP_GROWSDOWN,
            -1,
            0,
        )
    };

    if MAP_FAILED == stack {
        return Err(format!("mmap stack failed: {}", std::io::Error::last_os_error()));
    }

    loader.stack_base = stack as *mut u8;
    loader.stack_size = ELF_DEFAULT_STACK_SIZE;

    // ---- STEP 2: reserve the full ELF virtual range with PROT_NONE ----
    // Stack is already claimed so the kernel won't overlap it with ELF base.
    let hint  = if is_pie { ptr::null_mut() } else { vmin as *mut libc::c_void };
    let flags = MAP_PRIVATE | MAP_ANONYMOUS | if is_pie { 0 } else { MAP_FIXED };

    let base = unsafe {
        mmap(hint, map_size, PROT_NONE, flags, -1, 0)
    };

    if MAP_FAILED == base {
        unsafe { munmap(stack, ELF_DEFAULT_STACK_SIZE); }
        return Err(format!("mmap reservation failed: {}", std::io::Error::last_os_error()));
    }

    let base      = base as *mut u8;
    let load_bias = if is_pie { base as usize - vmin } else { 0 };

    loader.base      = base;
    loader.map_size  = map_size;
    loader.load_bias = load_bias;

    // ---- STEP 3: map each PT_LOAD segment over the reservation ----
    for i in 0..hdr.e_phnum {
        let ph = match elf_get_phdr(data, &hdr, i) {
            Some(p) if PT_LOAD == p.p_type => p,
            _ => continue,
        };

        if let Err(e) = map_segment(data, &ph, load_bias) {
            // Clean up both reservations on failure
            unsafe {
                munmap(base as *mut libc::c_void, map_size);
                munmap(stack, ELF_DEFAULT_STACK_SIZE);
            }
            return Err(e);
        }
    }

    loader.entry = (hdr.e_entry as usize + load_bias) as *mut u8;

    eprintln!("Loaded {} ELF{} at {:p}, entry={:p}, bias=0x{:x}",
        loader.arch.to_str(),
        if loader.is_64bit { 64 } else { 32 },
        loader.base,
        loader.entry,
        loader.load_bias,
    );

    return Ok(loader);
}

// ------------------------------------------------------------------ //
// Execution                                                          //
// ------------------------------------------------------------------ //

#[cfg(target_arch = "x86_64")]
unsafe fn jump_to_entry(sp: *mut u8, entry: *mut u8) -> ! {
    std::arch::asm!(
        "mov rsp, {sp}",
        "xor rbp, rbp",
        "xor rdx, rdx",
        "jmp {entry}",
        sp    = in(reg) sp,
        entry = in(reg) entry,
        options(nostack, noreturn)
    );
}

#[cfg(target_arch = "x86")]
unsafe fn jump_to_entry(sp: *mut u8, entry: *mut u8) -> ! {
    std::arch::asm!(
        "mov esp, {sp}",
        "xor ebp, ebp",
        "xor edx, edx",
        "jmp {entry}",
        sp    = in(reg) sp,
        entry = in(reg) entry,
        options(nostack, noreturn)
    );
}

#[cfg(target_arch = "aarch64")]
unsafe fn jump_to_entry(sp: *mut u8, entry: *mut u8) -> ! {
    std::arch::asm!(
        "mov sp,  {sp}",
        "mov x29, #0",
        "mov x30, #0",
        "br  {entry}",
        sp    = in(reg) sp,
        entry = in(reg) entry,
        options(nostack, noreturn)
    );
}

#[cfg(target_arch = "arm")]
unsafe fn jump_to_entry(sp: *mut u8, entry: *mut u8) -> ! {
    std::arch::asm!(
        "mov sp, {sp}",
        "mov fp, #0",
        "mov lr, #0",
        "bx  {entry}",
        sp    = in(reg) sp,
        entry = in(reg) entry,
        options(nostack, noreturn)
    );
}

#[cfg(target_arch = "riscv64")]
unsafe fn jump_to_entry(sp: *mut u8, entry: *mut u8) -> ! {
    std::arch::asm!(
        "mv  sp, {sp}",
        "li  fp, 0",
        "li  ra, 0",
        "jr  {entry}",
        sp    = in(reg) sp,
        entry = in(reg) entry,
        options(nostack, noreturn)
    );
}

#[cfg(not(any(
    target_arch = "x86_64",
    target_arch = "x86",
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "riscv64",
)))]
unsafe fn jump_to_entry(_sp: *mut u8, _entry: *mut u8) -> ! {
    panic!("jump_to_entry: unsupported architecture");
}

pub unsafe fn elf_execute(loader: &ElfLoader, args: &[String], env: &[String]) -> ! {
    let mut stk = (loader.stack_base as usize + loader.stack_size) & !15usize;

    // ---- push env strings onto the stack (high to low) ----
    let mut env_ptrs: Vec<*const u8> = Vec::with_capacity(env.len() + 1);
    for s in env.iter().rev() {
        let bytes = s.as_bytes();
        stk -= bytes.len() + 1;
        let dst = stk as *mut u8;
        ptr::copy_nonoverlapping(bytes.as_ptr(), dst, bytes.len());
        *dst.add(bytes.len()) = 0;
        env_ptrs.push(dst);
    }
    env_ptrs.reverse();
    env_ptrs.push(ptr::null());

    // ---- push arg strings onto the stack (high to low) ----
    let mut argu_ptrs: Vec<*const u8> = Vec::with_capacity(args.len() + 1);
    for s in args.iter().rev() {
        let bytes = s.as_bytes();
        stk -= bytes.len() + 1;
        let dst = stk as *mut u8;
        ptr::copy_nonoverlapping(bytes.as_ptr(), dst, bytes.len());
        *dst.add(bytes.len()) = 0;
        argu_ptrs.push(dst);
    }
    argu_ptrs.reverse();
    argu_ptrs.push(ptr::null());

    stk &= !15usize;

    // AT_PHDR = base + phdr_offset: the in-memory address of the program
    // header table. libc's _start reads this to find TLS segments, vDSO, etc.
    let at_phdr = loader.base as usize + loader.phdr_offset;

    let auxv: &[(usize, usize)] = &[
        (libc::AT_PAGESZ as usize, PAGE_SIZE),
        (libc::AT_BASE   as usize, loader.base  as usize),
        (libc::AT_FLAGS  as usize, 0),
        (libc::AT_ENTRY  as usize, loader.entry as usize),
        // program header info — required by musl/glibc _start
        (libc::AT_PHDR   as usize, at_phdr),
        (libc::AT_PHENT  as usize, loader.phdr_entsize),
        (libc::AT_PHNUM  as usize, loader.phdr_num),
        (libc::AT_UID    as usize, libc::getuid()  as usize),
        (libc::AT_EUID   as usize, libc::geteuid() as usize),
        (libc::AT_GID    as usize, libc::getgid()  as usize),
        (libc::AT_EGID   as usize, libc::getegid() as usize),
        (libc::AT_SECURE as usize, 0),
        (0, 0), // AT_NULL terminator
    ];

    let pw = if loader.is_64bit { 8usize } else { 4usize };

    let n_slots = 1
        + args.len() + 1
        + env.len()  + 1
        + auxv.len() * 2;

    stk -= n_slots * pw;
    stk &= !15usize;

    let mut p = stk;

    macro_rules! push_val {
        ($v:expr) => {
            if loader.is_64bit {
                *(p as *mut u64) = $v as u64;
            } else {
                *(p as *mut u32) = $v as u32;
            }
            p += pw;
        };
    }

    push_val!(args.len());
    for ptr in &argu_ptrs { push_val!(*ptr as usize); }
    for ptr in &env_ptrs  { push_val!(*ptr as usize); }
    for (t, v) in auxv {
        push_val!(*t);
        push_val!(*v);
    }

    eprintln!("{}: entry={:p} sp=0x{:x}", loader.arch.to_str(), loader.entry, stk);

    jump_to_entry(stk as *mut u8, loader.entry);
}

pub fn elf_unload(loader: &mut ElfLoader) {
    unsafe {
        if !loader.base.is_null() {
            munmap(loader.base as *mut libc::c_void, loader.map_size);
            loader.base = ptr::null_mut();
        }

        if !loader.stack_base.is_null() {
            munmap(loader.stack_base as *mut libc::c_void, loader.stack_size);
            loader.stack_base = ptr::null_mut();
        }

        *loader = ElfLoader::default();
    }
}

pub fn elf_print_info(loader: &ElfLoader) {
    println!(
        "ELF Info:\n\
        \tArchitecture : {} ({})\n\
        \tType         : {}\n\
        \tMapped base  : {:p}\n\
        \tLoad bias    : 0x{:x}\n\
        \tEntry point  : {:p}\n\
        \tInterp       : {}\n\
        \tNative exec  : {}",
        loader.arch.to_str(),
        if loader.is_64bit { "ELF64" } else { "ELF32" },
        if loader.is_pie { "PIE (ET_DYN)" } else { "static (ET_EXEC)" },
        loader.base,
        loader.load_bias,
        loader.entry,
        if loader.interp_offset != 0 { "YES (dynamic)" } else { "NO (static)" },
        if loader.arch.is_native()   { "YES" }           else { "NO (cross-arch)" },
    );
}

fn to_cstring_vec(strings: &[String]) -> Vec<std::ffi::CString> {
    return strings.iter()
        .map(|s| std::ffi::CString::new(s.as_str()).unwrap_or_default())
        .collect();
}

pub fn elf_memfd_exec(data: &[u8], args: &[String], env: &[String]) -> Result<(), String> {
    let c_args = to_cstring_vec(args);
    let c_env  = to_cstring_vec(env);
    let argv: Vec<*const libc::c_char> = c_args.iter()
        .map(|s| s.as_ptr()).chain(std::iter::once(ptr::null())).collect();
    let envp: Vec<*const libc::c_char> = c_env.iter()
        .map(|s| s.as_ptr()).chain(std::iter::once(ptr::null())).collect();

    unsafe {
        let mfd = libc::syscall(libc::SYS_memfd_create, b"elf_mem\0".as_ptr(), 0u32) as i32;
        if mfd < 0 {
            return Err(format!("memfd_create failed: {}", std::io::Error::last_os_error()));
        }

        let mut done = 0usize;
        while done < data.len() {
            let n = write(mfd, data.as_ptr().add(done) as *const libc::c_void, data.len() - done);
            if n <= 0 {
                close(mfd);
                return Err(format!("write to memfd failed: {}", std::io::Error::last_os_error()));
            }
            done += n as usize;
        }

        fexecve(mfd, argv.as_ptr(), envp.as_ptr());

        let err = std::io::Error::last_os_error();
        close(mfd);
        return Err(format!("fexecve failed: {}", err));
    }
}