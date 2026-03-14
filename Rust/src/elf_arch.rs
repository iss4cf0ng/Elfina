//elf_arch.rs

pub const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];
pub const ELF_CLASS32: u8   = 1;
pub const ELF_CLASS64: u8   = 2;
pub const ELF_CLASS: usize  = 4;
pub const ELF_NIDENT: usize = 16;

pub const ET_EXEC: u16 = 2;
pub const ET_DYN: u16  = 3;

pub const PT_LOAD: u32   = 1;
pub const PT_INTERP: u32 = 3;

// Permission
pub const PF_X: u32 = 0x1; // Execute
pub const PF_W: u32 = 0x2; // Write
pub const PF_R: u32 = 0x4; // Read

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

}


#[derive(Debug, Default)]
pub struct ElfPhdr {

}

#[repr(C, packed)]
pub struct Elf64Ehdr {
    
}