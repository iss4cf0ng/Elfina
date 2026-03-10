#pragma once
#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    void *base;
    size_t map_size;
    void *entry;
    void *stack;
    size_t stack_size;
    int is_pie;
    uintptr_t load_bias;
} ElfLoader;

int elf_load_from_memory(const void *data, size_t size, ElfLoader *out);

void elf_execute(ElfLoader *loaded, int argc, char **argv, char **envp);

void elf_unload(ElfLoader *loaded);

#ifdef __cplusplus
}
#endif
