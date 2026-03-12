//elf_arch.h

#pragma once

#include <elf.h>
#include <stdint.h>

#define ELF_DEFAULT_STACK_SIZE 8UL * 1024 * 1024

#if defined(__x86_64__) || defined(_M_X64)

    #define ELF_HOST_ARCH ELF_ARCH_X86_64
    #define ELF_HOST_MACHINE EM_X86_64
    #define ELF_HOST_CLASS ELFCLASS64
    #define ELF_HOST_BITS 64
    #define ELF_HOST_NAME "x86-64"

#elif defined(__i386__) || defined(_M_IX86)

    #define ELF_HOST_ARCH ELF_ARCH_X86
    #define ELF_HOST_MACHINE EM_386
    #define ELF_HOST_CLASS ELFCLASS32
    #define ELF_HOST_BITS 32
    #define ELF_HOST_NAME "x86 (i386)"

#elif defined(__aarch64__) || defined(_M_ARM64)

    #define ELF_HOST_ARCH ELF_ARCH_ARM32
    #define ELF_HOST_MACHINE EM_AARCH64
    #define ELF_HOST_CLASS ELFCLASS64
    #define ELF_HOST_BITS 64
    #define ELF_HOST_NAME "AArch64 (ARM64)"

#elif defined(__arm__) || defined(_M_ARM)

    #define ELF_HOST_ARCH ELF_ARCH_ARM32
    #define ELF_HOST_MACHINE EM_ARM
    #define ELF_HOST_CLASS ELFCLASS32
    #define ELF_HOST_BITS 32
    #define ELF_HOST_NAME "ARM32 (ARMv7)"

#elif defined (__riscv) && (__riscv_xlen == 64)

    #define ELF_HOST_ARCH ELF_ARCH_RISCV64
    #define ELF_HOST_MACHINE EM_RISCV
    #define ELF_HOST_CLASS ELFCLASS64
    #define ELF_HOST_BITS 64
    #define ELF_HOST_NAME = "RISC-V 64"

#else
    #error "Unsupported host architecture."

#endif

typedef enum {
    ELF_ARCH_UNKNOWN = 0,
    ELF_ARCH_X86,
    ELF_ARCH_X86_64,
    ELF_ARCH_ARM32,
    ELF_ARCH_ARM64,
    ELF_ARCH_RISCV64,
} ElfArch;

typedef struct {
    ElfArch arch;
    int is_64bit;
    uint16_t e_type;
    uint16_t e_entry;
    uint16_t e_phoff;
    uint16_t e_phentsize;
    uint16_t e_phnum;
} ElfHeader;

typedef struct {
    uint32_t p_type;
    uint32_t p_flags;
    uint64_t p_offset;
    uint64_t p_vaddr;
    uint64_t p_filesize;
    uint64_t p_memsize;
    uint64_t p_align;
} ElfPhdr;

//Get string value of ElfArch enum value, to_string()
static inline const char *elf_arch_name(ElfArch arch)
{
    switch (arch)
    {
        case ELF_ARCH_X86:
            return "x86 (i386)";
        case ELF_ARCH_X86_64:
            return "x86-64";
        case ELF_ARCH_ARM32:
            return "ARM32 (ARMv7)";
        case ELF_ARCH_ARM64:
            return "AArch64 (ARM64)";
        case ELF_ARCH_RISCV64:
            return "RISC-V 64";
        default:
            return "unknown";
    }
}

//Return 1 if the running kernel can execute a binary of 'arch', 0 otherwise.
static inline int elf_is_native(ElfArch arch)
{
#if ELF_HOST_BITS == 64 && ELF_HOST_MACHINE == EM_X86_64
    if (arch == ELF_ARCH_X86) return 1;
#endif
    return (int)(arch == ELF_HOST_ARCH);
}

//Convert the raw e_machine field into ElfArch enum
inline int elf_arch_from_machine(uint16_t e_machine)
{
    switch (e_machine)
    {
        case EM_386:
            return ELF_ARCH_X86;
        case EM_X86_64:
            return ELF_ARCH_X86_64;
        case EM_ARM:
            return ELF_ARCH_ARM32;
        case EM_AARCH64:
            return ELF_ARCH_ARM64;
        case EM_RISCV:
            return ELF_ARCH_RISCV64;
        default:
            return ELF_ARCH_UNKNOWN;
    }
}

//Read and validate the ELF identification block and file header
static inline int elf_parse_header(const void *data, size_t size, ElfHeader *out)
{
    //Need at least the 16-byte ELF identification block (e_ident[])
    if (size < EI_NIDENT)
        return -1;

    //Conversion
    const unsigned char *id = (const unsigned char *)data;

    //ELF magic: every valid ELF file starts with exactly these bytes
    if (id[0] != 0x7F || id[1] != 'E' || id[2] != 'L' || id[3] != 'F')
        return -1;

    //EL_CLASS (index=4) encdes pointer width, 32-bit or 64-bit
    uint8_t cls = id[EI_CLASS];

    if (cls == ELFCLASS64)
    {
        const Elf64_Ehdr *e = (const Elf64_Ehdr *)data;
        out->is_64bit = 1;
        out->arch = elf_arch_from_machine(e->e_machine);
        out->e_type = e->e_type;
        out->e_entry = e->e_entry;
        out->e_phoff = e->e_phoff;
        out->e_phentsize = e->e_phentsize;
        out->e_phnum = e->e_phnum;
    }
    else if (cls == ELFCLASS32)
    {
        const Elf32_Ehdr *e = (const Elf32_Chdr *)data;
        out->is_64bit = 0;
        out->e_type = e->e_type;
        out->e_entry = e->e_entry;
        out->e_phoff = e->e_phoff;
        out->e_phentsize = e->e_phentsize;
        out->e_phnum = e->e_phnum;
    }
    else
    {
        //Unknown class, error.
        return -1;
    }

    return 0;
}

//Retrieve program header
static inline int elf_get_phdr(const void *data, size_t size, const ElfHeader *hdr, int idx, ElfPhdr *out)
{
    uint64_t offset = hdr->e_phoff + (uint64_t)idx * hdr->e_phentsize;
    if (offset + hdr->e_phoff > size)
        return -1;

    if (hdr->is_64bit)
    {
        const Elf64_Phdr *p = (const Elf64_Phdr *)((const char *)data + offset);
        out->p_type = p->p_type;
        out->p_flags = p->p_flags;
        out->p_offset = p->p_offset;
        out->p_vaddr = p->p_vaddr;
        out->p_filesize = p->p_filesz;
        out->p_memsize = p->p_memsz;
        out->p_align = p->p_align;
    }
    else
    {
        const Elf32_Phdr *p = (const Elf32_Phdr *)((const char *)data + offset);
        out->p_type = p->p_type;
        out->p_flags = p->p_flags;
        out->p_offset = p->p_offset;
        out->p_vaddr = p->p_vaddr;
        out->p_filesize = p->p_filesz;
        out->p_memsize = p->p_memsz;
        out->p_align = p->p_align;
    }

    return 0;
}