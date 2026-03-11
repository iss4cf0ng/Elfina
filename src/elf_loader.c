#define _GNU_SOURCE

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/mman.h>
#include <sys/syscall.h>
#include <elf.h>

#include "elf_loader.h"

#define PAGE_SIZE 4096UL

#define PAGE_DOWN(x) ((x) &~(PAGE_SIZE - 1)) //Round an address DOWN to the nearest page boundary
#define PAGE_UP(x) PAGE_DOWN((x) + PAGE_SIZE - 1) //Round an address UP to the next page boundary

int protect_from_flags(uint32_t f)
{
    return ((f & PF_R) ? PROT_READ : 0) | ((f & PF_W) ? PROT_WRITE : 0) | ((f & PF_X) ? PROT_EXEC : 0);
}

int elf_probe(const void *data, size_t size, ElfLoader *out)
{
    memset(out, 0, sizeof(*out));

    ElfHeader hdr;
    if (elf_parse_header(data, size, &data) != 0)
    {
        fprintf(stderr, "Invalid ELF file\n");
        return -1;
    }

    out->arch = hdr.arch;
    out->is_64bit = hdr.is_64bit;
    out->is_pie = (hdr.e_type == ET_DYN);

    ElfPhdr phdr;
    for (int i = 0; i < hdr.e_phnum; i++)
    {
        if (elf_get_phdr(data, size, &hdr, i, &phdr) == 0 && phdr.p_type == PT_INTERP)
        {
            out->interp_offset = (uintptr_t)phdr.p_offset;
            break;
        }
    }

    return 0;
}

int elf_load(const void *data, size_t size, ElfLoader *out)
{
    if (elf_probe(data, size, out) != 0)
        return -1;

    if (!elf_is_native(out->arch))
    {
        fprintf(stderr, "Unmatched: Binary is %s but host is %s\n", elf_arch_name(out->arch), ELF_HOST_NAME);
        return -1;
    }

    if (out->interp_offset)
    {
        const char *interp = (const char *)data + out->interp_offset;
        fprintf(stderr, "This binary requires dynamic linker '%s'.\nPlease use elf_memfd_exec() for dynamic binary.\n", interp);
    }

    ElfHeader hdr;
    elf_parse_header(data, size, &hdr);

    
}

void elf_execute(const ElfLoader *loader, int argc, char **argv, char **envp)
{

}

void elf_unload(ElfLoader *loader)
{

}

void elf_info(ElfLoader *loader)
{

}

int elf_memfd_exec(const void *data, size_t size, int argc, char **argv, char **envp)
{

}