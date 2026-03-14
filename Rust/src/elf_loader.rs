use std::{arch::x86_64::_mm_broadcastss_ps, env, fmt::format, ops::BitOrAssign, ptr};
use capstone::arch::{self, sparc::SparcReg::SPARC_REG_ENDING};
use libc::{
    MAP_ANON, MAP_ANONYMOUS, MAP_FAILED, MAP_FIXED, MAP_GROWSDOWN, MAP_PRIVATE, PROT_NONE, PROT_READ, PROT_WRITE, close, fexecve, mmap, mprotect, munmap, write
};

use crate::elf_arch::{
    ElfArch, ElfHeader, ElfPhdr,
    elf_get_phdr, elf_parse_header, prot_from_flags, host_arch_name,
    ET_DYN, PT_LOAD, PT_INTERP, ELF_DEFAULT_STACK_SIZE,
};

const PAGE_SIZE: usize = 4096;

#[inline]
fn page_down(x: usize) -> usize {
    return x & !(PAGE_SIZE);
}

#[inline]
fn page_up(x: usize) -> usize {
    return page_down(x + PAGE_SIZE - 1);
}

#[derive(Debug)]
pub struct ElfLoader {
    pub arch         : ElfArch,   // target architecture
    pub is_64bit     : bool,      // true = ELF64, false = ELF32
    pub is_pie       : bool,      // true = ET_DYN (PIE), false = ET_EXEC
    pub interp_offset: usize,     // file offset of PT_INTERP string, or 0

    pub base     : *mut u8,   // base of the mmap reservation
    pub map_size : usize,     // total size of the reservation
    pub load_bias: usize,     // ASLR slide: real_addr = elf_vaddr + load_bias

    pub entry     : *mut u8,
    pub stack_base: *mut u8,
    pub stack_size: usize,
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
        }
    }
}

pub fn elf_probe(data: &[u8]) -> Result<ElfLoader, String> {
    let hdr = elf_parse_header(data).ok_or_else(|| "Invalid ELF file".to_string())?;
    let mut loader = ElfLoader::default();
    loader.arch = hdr.arch;
    loader.is_64bit = hdr.is_64bit;
    loader.is_pie = hdr.e_type == ET_DYN;

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

fn interp_path(data: &[u8], offset: usize) -> String {
    if offset >= data.len() {
        return "Invalid".to_string();
    }

    let slice = &data[offset..];
    let p = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());

    return String::from_utf8_lossy(&slice[..p]).to_string();
}

fn map_segment(data: &[u8], ph: &ElfPhdr, bias: usize) -> Result<(), String> {
    let seg_start = page_down(ph.p_vaddr as usize + bias); // segment start
    let seg_end = page_up(ph.p_vaddr as usize + bias + ph.p_memsize as usize); // segment end
    let seg_len = seg_end - seg_start;

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

    let dest = (ph.p_vaddr as usize + bias) as *mut u8;
    let src_offset = ph.p_offset as usize;
    let copy_bytes = (ph.p_filesize as usize).min(data.len() - src_offset);

    unsafe {
        ptr::copy_nonoverlapping(data.as_ptr().add(src_offset), dest, copy_bytes);
    }

    // Zero to BSS tail (p_memsize > p_filesize region)
    if ph.p_memsize > ph.p_filesize {
        let bss_start = (ph.p_vaddr as usize + bias + ph.p_filesize as usize) as *mut u8;
        let bss_len = (ph.p_memsize - ph.p_filesize) as usize;
        
        unsafe  {
            ptr::write_bytes(bss_start, 0, bss_len);
        }
    }

    // Apply correct R/W/X permissions via mprotect
    let prot = prot_from_flags(ph.p_flags);
    if unsafe {
        mprotect(seg_start as *mut libc::c_void, seg_len, prot)
    } != 0 {
        return Err(format!("mprotect failed: {}", std::io::Error::last_os_error()));
    }

    return Ok(());
}

pub fn elf_load(data: &[u8]) -> Result<ElfLoader, String> {
    let mut loader = elf_probe(data)?;
    if !loader.arch.is_native() {
        return Err(format!("Cross-arch: binary os {} host is {}", loader.arch.to_str(), host_arch_name()));
    }

    if loader.interp_offset != 0 {
        let interp = interp_path(data, loader.interp_offset);
        eprintln!("Warning: Binary requires dynamic linker '{}'.\nUse elf_memfd_exec() for dynamic binary.", interp);
    }

    let hdr = elf_parse_header(data).unwrap();

    let mut vmin = usize::MAX;
    let mut vmax = 0usize;

    for i in 0..hdr.e_phnum {
        let ph = match elf_get_phdr(data, &hdr, i) {
            Some(p) if p.p_type == PT_LOAD => p,
            _ => continue,
        };

        let start = page_down(ph.p_vaddr as usize);
        let end = page_up(ph.p_vaddr as usize + ph.p_memsize as usize);

        if start < vmin {
            vmin = start;
        }
        if end > vmax {
            vmax = end;
        }
    }

    if usize::MAX == vmin {
        return Err("No PT_LOAD segments found".to_string());
    }

    let map_size = vmax - vmin;
    let is_pie = loader.is_pie;
    let hint = if is_pie { ptr::null_mut() } else { vmin as *mut libc::c_void };
    let flags = MAP_PRIVATE | MAP_ANONYMOUS | if is_pie { 0 } else { MAP_FIXED };
    let base = unsafe {
        mmap(hint, map_size, PROT_NONE, flags, -1, 0)
    };

    if MAP_FAILED == base {
        return Err(format!("mmap reservation failed: {}", std::io::Error::last_os_error()));
    }

    let base = base as *mut u8;
    let load_bias = if is_pie { base as usize - vmin } else { 0 };

    loader.base = base;
    loader.map_size = map_size;
    loader.load_bias = load_bias;

    for i in 0..hdr.e_phnum {
        let ph = match elf_get_phdr(data, &hdr, i) {
            Some(p) if PT_LOAD == p.p_type => p,
            _ => continue,
        };

        if let Err(e) = map_segment(data, &ph, load_bias) {
            unsafe {
                munmap(base as *mut libc::c_void, map_size);
            }

            return Err(e);
        }
    }

    loader.entry = (hdr.e_entry as usize + load_bias) as *mut u8;

    let stack = unsafe {
        mmap(ptr::null_mut(), ELF_DEFAULT_STACK_SIZE, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS | MAP_GROWSDOWN, -1, 0)
    };

    if MAP_FAILED == stack {
        unsafe {
            munmap(base as *mut libc::c_void, map_size);
        }
    }

    loader.stack_base = stack as *mut u8;
    loader.stack_size = ELF_DEFAULT_STACK_SIZE;

    eprintln!("Loaded {} ELF{} at {:p}, entry={:p}, bias=0x{:x}", 
        loader.arch.to_str(), 
        if loader.is_64bit { 64 } else { 32 }, 
        loader.base, 
        loader.entry, 
        loader.load_bias, 
    );

    return Ok(loader);
}

// Execution

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
 
// Fallback for unsupported architectures: panics at runtime
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

unsafe fn elf_execute(loader: &ElfLoader, args: &[String], env: &[String]) -> !{
    let mut stk = (loader.stack_base as usize + loader.stack_size) & !15usize;

    let mut env_ptrs: Vec<*const u8> = Vec::with_capacity(env.len() + 1);
    

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

}

pub fn elf_memfd_exec(data: &[u8], args: &[String], env: &[String]) -> Result<(), String> {

    unsafe {
        let mfd = libc::syscall(libc::SYS_memfd_create, b"elf_mem\0".as_ptr(), 0u32) as i32;


        let err = std::io::Error::last_os_error();
        close(mfd);

        return Err(format!("fexecve failed: {}", err));
    }
}