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

    uintptr_t vmin = UINTPTR_MAX;
    uintptr_t vmax = 0;
    ElfPhdr phdr;
    for (int i = 0; i < hdr.e_phnum; i++)
    {
        if (elf_get_phdr(data, size, &hdr, i, &phdr) != 0)
            continue;

        if (phdr.p_type != PT_LOAD)
            continue;

        uintptr_t s = PAGE_DOWN(phdr.p_vaddr); //segment start
        uintptr_t e = PAGE_UP(phdr.p_vaddr + phdr.p_memsize); //segment end

        vmin = s < vmin ? s : vmin;
        vmax = e > vmax ? e : vmax;
    }

    if (UINTPTR_MAX == vmin)
    {
        fprintf(stderr, "PT_LOAD segments not found\n");
        return -1;
    }

    size_t map_size = vmax - vmin;
    int is_pie = out->is_pie;

    void *base = mmap(
        is_pie ? NULL : (void *)vmin, 
        map_size, 
        PROT_NONE, 
        MAP_PRIVATE | MAP_ANONYMOUS | (is_pie ? 0 : MAP_FIXED), 
        -1, 
        0
    );
    if (MAP_FAILED == base)
    {
        perror("mmap: reserve virtual range");
        return -1;
    }

    uintptr_t bias = is_pie ? (uintptr_t)base - vmin : 0;
    out->base = base;
    out->map_size = map_size;
    out->bias = bias;

    for (int i = 0; i < hdr.e_phnum; i++)
    {
        if (elf_get_phdr(data, size, &hdr, i, &phdr) != 0)
            continue;

        if (PT_LOAD != phdr.p_type)
            continue;

        uintptr_t segment_start = PAGE_DOWN(phdr.p_vaddr + bias);
        uintptr_t segment_end = PAGE_UP(phdr.p_vaddr + bias + phdr.p_memsize);
        size_t segment_length = segment_end - segment_start;

        void *mem = mmap(
            (void *)segment_start, 
            segment_length, 
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANONYMOUS | MAP_FIXED,
            -1,
            0
        );

        if (MAP_FAILED == mem)
        {
            perror("mmap: mmap error");
            goto failure;
        }

        size_t copy_bytes = phdr.p_filesize;
        if (phdr.p_offset + copy_bytes > size)
            copy_bytes = size - phdr.p_offset;

        memcpy((char *)(phdr.p_vaddr + bias), (const char *)data + phdr.p_offset, copy_bytes);

        if (phdr.p_memsize > phdr.p_filesize)
            memset((char *)(phdr.p_vaddr + bias) + phdr.p_filesize, 0, phdr.p_memsize - phdr.p_filesize);

        if (mprotect((void *)segment_start, segment_length, protect_from_flags(phdr.p_flags)) != 0)
        {
            perror("mprotect: permissions error");
            goto failure;
        }
    }

    out->entry = (void *)(hdr.e_entry + bias);

    size_t stack_size = ELF_DEFAULT_STACK_SIZE;
    void *stack = mmap(NULL, stack_size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS | MAP_GROWSDOWN, -1, 0);
    if (MAP_FAILED == stack)
    {
        perror("mmap: stack error");
        goto failure;
    }

    out->stack_base = stack;
    out->stack_size = stack_size;

    fprintf(stdout, "Loaded %s ELF%d at %p, entry=%p, bias=0x%lx\n", 
        elf_arch_name(out->arch),
        out->is_64bit ? 64 : 32,
        base,
        out->entry,
        (unsigned long)bias
    );

    return 0;

failure:
    munmap(base, map_size);
    return -1;
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