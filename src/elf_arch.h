//elf_arch.h

#pragma once

#include <elf.h>
#include <stdint.h>

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

static inline ElfArch elf_arch_from_machine(uint16_t machine)
{

}