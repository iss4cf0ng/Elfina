//elf_arch.rs

use libc::{EI_CLASS, EI_NIDENT, ELFCLASS32, ELFCLASS64, };

pub const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];
pub const ELF_CLASS32: u8   = 1;
pub const ELF_CLASS64: u8   = 2;
pub const ELF_CLASS: usize  = 4;
pub const ELF_NIDENT: usize = 16;

pub const ET_EXEC: u16 = 2; // fixed-address executable
pub const ET_DYN: u16  = 3; // PIE or shared object

pub const PT_LOAD: u32   = 1; // loadable segment
pub const PT_INTERP: u32 = 3; // path to dynamic linker

// Permission
pub const PF_X: u32 = 0x1; // Execute permission
pub const PF_W: u32 = 0x2; // Write permission
pub const PF_R: u32 = 0x4; // Read permission

pub const EM_386: u16     = 3;
pub const EM_ARM: u16     = 40;
pub const EM_X86_64: u16  = 62;
pub const EM_AARCH64: u16 = 183;
pub const EM_RISCV: u16   = 243;

pub const ELF_DEFAULT_STACK_SIZE: usize = 8 * 1024 * 1024; //8 MB

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfArch {
    Unknown,
    X86    ,  // i386 / IA-32
    X86_64 ,  // AMD64
    Arm32  ,  // ARMv7 / Thumb-2
    Arm64  ,  // AArch64
    RiscV64,  // RISC-V 64-bit
}

impl ElfArch {
    pub fn to_str(self) -> &'static str {
        match self {
            ElfArch::Unknown => "Unknown",
            ElfArch::X86     => "x86 (i386)",
            ElfArch::X86_64  => "x86-64",
            ElfArch::Arm32   => "ARM32 (ARMv7)",
            ElfArch::Arm64   => "AArch64 (ARM64)",
            ElfArch::RiscV64 => "RISC-V 64",
        }
    }

    pub fn machine_cast(e_machine: u16) -> Self {
        match e_machine {
            EM_386     => ElfArch::X86,
            EM_X86_64  => ElfArch::X86_64,
            EM_ARM     => ElfArch::Arm32,
            EM_AARCH64 => ElfArch::Arm64,
            EM_RISCV   => ElfArch::RiscV64,
            _          => ElfArch::Unknown,
        }
    }

    pub fn is_native(self) -> bool {
        let host = host_arch();

        #[cfg(target_arch = "x86_64")]
        if self == ElfArch::X86 {
            return true;
        }

        return self == host;
    }
}

impl Default for ElfArch {
    fn default() -> Self {
        return ElfArch::Unknown;
    }
}

pub fn host_arch() -> ElfArch {
    #[cfg(target_arch = "x86_64")] { return ElfArch::X86_64; }
    #[cfg(target_arch = "x86")] { return ElfArch::X86; }
    #[cfg(target_arch = "aarch64")] { return ElfArch::Arm64; }
    #[cfg(target_arch = "arm")] { return ElfArch::Arm32; }
    #[cfg(target_arch = "riscv64")] { return ElfArch::RiscV64; }
    #[allow(unreachable_code)]

    return ElfArch::Unknown;
}

pub fn host_arch_name() -> &'static str {
    return host_arch().to_str();
}

#[derive(Debug, Default)]
pub struct ElfHeader {
    pub arch       : ElfArch,
    pub is_64bit   : bool,
    pub e_type     : u16,
    pub e_entry    : u64,       // virtual address of entry point
    pub e_phoffset : u64,       // file offset to program header table
    pub e_phentsize: u16,       // size of one program header entry
    pub e_phnum    : u16,       // number of program header entries
}

#[derive(Debug, Default)]
pub struct ElfPhdr {
    pub p_type    : u32,
    pub p_flags   : u32,
    pub p_offset  : u64,
    pub p_vaddr   : u64,
    pub p_filesize: u64,
    pub p_memsize : u64,
    pub p_align   : u64,
}

#[repr(C, packed)]
pub struct Elf64Ehdr {
    e_ident    : [u8; 16],
    e_type     : u16,
    e_machine  : u16,
    e_version  : u32,
    e_entry    : u64,
    e_phoffset : u64,
    e_shoffset : u64,
    e_flags    : u32,
    e_ehsize   : u16,
    e_phentsize: u16,
    e_phnum    : u16,
    e_shentsize: u16,
    e_shnum    : u16,
    e_shstrndx : u16,
}

#[repr(C, packed)]
pub struct Elf32Ehdr {
    e_ident    : [u8; 16],
    e_type     : u16,
    e_machine  : u16,
    e_version  : u32,
    e_entry    : u32,
    e_phoffset : u32,
    e_shoffset : u32,
    e_flags    : u32,
    e_ehsize   : u16,
    e_phentsize: u16,
    e_phnum    : u16,
    e_shetsize : u16,
    e_shnum    : u16,
    e_shstrndx : u16,
}

#[repr(C, packed)]
pub struct Elf64Phdr {
    p_type    : u32,
    p_flags   : u32,
    p_offset  : u64,
    p_vaddr   : u64,
    p_filesize: u64,
    p_memsize : u64,
    p_align   : u64,
}

#[repr(C, packed)]
pub struct Elf32Phdr {
    p_type    : u32,
    p_offset  : u32,
    p_vaddr   : u32,
    p_paddr   : u32,
    p_filesize: u32,
    p_memsize : u32,
    p_flags   : u32,
    p_align   : u32,
}

pub fn elf_parse_header(data: &[u8]) -> Option<ElfHeader> {
    if data.len() < EI_NIDENT {
        return None;
    }

    if &data[0..4] != &ELF_MAGIC {
        return None;
    }

    let cls = data[EI_CLASS];
    let mut hdr = ElfHeader::default();

    if cls == ELFCLASS64 {
        if data.len() < std::mem::size_of::<Elf64Ehdr>() {
            return None;
        }

        let e = unsafe {
            &*(data.as_ptr() as *const Elf64Ehdr)
        };

        hdr.is_64bit    = true;
        hdr.arch        = ElfArch::machine_cast(e.e_machine);
        hdr.e_type      = e.e_type;
        hdr.e_entry     = e.e_entry;
        hdr.e_phoffset  = e.e_phoffset;
        hdr.e_phentsize = e.e_phentsize;
        hdr.e_phnum     = e.e_phnum;

    } else if cls == ELFCLASS32 {
        if data.len() < std::mem::size_of::<Elf32Ehdr>() {
            return None;
        }

        let e = unsafe {
            &*(data.as_ptr() as *const Elf32Ehdr)
        };

        hdr.is_64bit    = false;
        hdr.arch        = ElfArch::machine_cast(e.e_machine);
        hdr.e_type      = e.e_type;
        hdr.e_entry     = e.e_entry as u64;
        hdr.e_phoffset  = e.e_phoffset as u64;
        hdr.e_phentsize = e.e_phentsize;
        hdr.e_phnum     = e.e_phnum;

    } else {
        return None;
    }

    return Some(hdr);
}

pub fn elf_get_phdr(data: &[u8], hdr: &ElfHeader, idx: u16) -> Option<ElfPhdr> {
    if idx >= hdr.e_phnum {
        return None;
    }

    let offset = hdr.e_phoffset + (idx as u64) + (hdr.e_phentsize as u64);
    let end = offset + hdr.e_phentsize as u64;

    if end as usize > data.len() {
        return None;
    }

    let ptr = &data[offset as usize] as *const u8;
    let mut ph = ElfPhdr::default();

    if hdr.is_64bit {
        let p = unsafe {
            &*(ptr as *const Elf64Phdr)
        };

        ph.p_type     = p.p_type;
        ph.p_flags    = p.p_flags;
        ph.p_offset   = p.p_offset;
        ph.p_vaddr    = p.p_vaddr;
        ph.p_filesize = p.p_filesize;
        ph.p_memsize  = p.p_memsize;
        ph.p_align    = p.p_align;

    } else {
        let p = unsafe {
            &*(ptr as *const Elf32Phdr)
        };

        ph.p_type     = p.p_type;
        ph.p_flags    = p.p_flags;
        ph.p_offset   = p.p_offset as u64;
        ph.p_vaddr    = p.p_vaddr as u64;
        ph.p_filesize = p.p_filesize as u64;
        ph.p_memsize  = p.p_memsize as u64;
        ph.p_align    = p.p_align as u64;

    }

    return Some(ph);
}

pub fn prot_from_flags(flags: u32) -> i32 {
     let mut prot = 0i32;
    if flags & PF_R != 0 {
        prot |= libc::PROT_READ;  
    }

    if flags & PF_W != 0 {
        prot |= libc::PROT_WRITE;
    }

    if flags & PF_X != 0 {
        prot |= libc::PROT_EXEC;
    }

    return prot;
}