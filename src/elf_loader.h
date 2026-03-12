//elf_loader.h

#pragma once

#include <stdint.h>
#include <stddef.h>
#include "elf_arch.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct
{
    //Target binary properties
    ElfArch arch;
    int is_64bit;
    int is_pie;
    uintptr_t interp_offset;

    //Mapping details
    void *base;
    size_t map_size;
    uintptr_t bias;

    void *entry;
    void *stack_base;
    size_t stack_size;
} ElfLoader;

typedef struct 
{
    uint64_t type;
    uint64_t val;
} Aux64;

int elf_probe(const void *data, size_t size, ElfLoader *out);

int elf_load(const void *data, size_t size, ElfLoader *out);

void elf_execute(const ElfLoader *loader, int argc, char **argv, char **envp);

void elf_unload(ElfLoader *loader);

void elf_info(const ElfLoader *loader);

int elf_memfd_exec(const void *data, size_t size, int argc, char **argv, char **envp);

#ifdef __cplusplus
}
#endif