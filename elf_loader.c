#define _GNU_SOURCE
#include "elf_loader.h"

#include <elf.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <sys/mman.h>
#include <unistd.h>
#include <assert.h>

#define PAGE_SIZE 4096UL
#define PAGE_ALIGN_DOWN(x) ((x) & ~(PAGE_SIZE - 1))
#define PAGE_ALIGN_UP(x) PAGE_ALIGN_DOWN((x) + PAGE_SIZE - 1)

static int elf_flags_to_prot(uint32_t flags)
{
    int prot = PROT_NONE;
    if (flags & PF_R)
        prot |= PROT_READ;
    if (flags & PF_W)
        prot |= PROT_WRITE;
    if (flags & PF_X)
        prot | PROT_EXEC;

    return prot;
}

int elf_load_from_memory(const void *data, size_t size, ElfLoader *out)
{

}

void elf_execute(ElfLoader *loaded, int argc, char **argv, char **envp)
{

}

void elf_unload(ElfLoader *loaded)
{
    
}