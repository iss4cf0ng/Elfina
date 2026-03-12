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
#define PUSH_VAL(v, loader)  do { \
    if (loader->is_64bit) { \
        *(uint64_t *)p = (uint64_t)(uintptr_t)(v); \
    } else { \
        *(uint32_t *)p = (uint32_t)(uintptr_t)(v); \
    } \
    p += pw; \
} while(0)

#if defined(__x86_64__)

static void __attribute__((noreturn))
jump_to_entry(void *sp, void *entry)
{
    __asm__ volatile(
        "mov  %0, %%rsp\n\t"
        "xor  %%rbp, %%rbp\n\t"
        "xor  %%rdx, %%rdx\n\t"
        "jmp  *%1\n\t"
        : : "r"(sp), "r"(entry) : "memory"
    );

    __builtin_unreachable();
}

#elif defined(__i386__)

static void __attribute__((noreturn))
jump_to_entry(void *sp, void *entry)
{
    __asm__ volatile(
        "mov  %0, %%esp\n\t"
        "xor  %%ebp, %%ebp\n\t"
        "xor  %%edx, %%edx\n\t"
        "jmp  *%1\n\t"
        : : "r"(sp), "r"(entry) : "memory");
    __builtin_unreachable();
}

#elif defined(__aarch64__)

static void __attribute__((noreturn))
jump_to_entry(void *sp, void *entry)
{
    __asm__ volatile(
        "mov  sp,  %0\n\t"
        "mov  x29, #0\n\t"
        "mov  x30, #0\n\t"
        "br   %1\n\t"
        : : "r"(sp), "r"(entry) : "memory");
    __builtin_unreachable();
}

#elif defined(__arm__)

static void __attribute__((noreturn))
jump_to_entry(void *sp, void *entry)
{
    __asm__ volatile(
        "mov  sp,  %0\n\t"
        "mov  fp,  #0\n\t"
        "mov  lr,  #0\n\t"
        "bx   %1\n\t"
        : : "r"(sp), "r"(entry) : "memory");
    __builtin_unreachable();
}

#elif defined(__riscv)
static void __attribute__((noreturn))
jump_to_entry(void *sp, void *entry)
{
    __asm__ volatile(
        "mv   sp,  %0\n\t"
        "li   fp,  0\n\t"
        "li   ra,  0\n\t"
        "jr   %1\n\t"
        : : "r"(sp), "r"(entry) : "memory");
    __builtin_unreachable();
}

#else

#error "No jump_to_entry() trampoline for this architecture."

#endif

int protect_from_flags(uint32_t f)
{
    return ((f & PF_R) ? PROT_READ : 0) | ((f & PF_W) ? PROT_WRITE : 0) | ((f & PF_X) ? PROT_EXEC : 0);
}

int elf_probe(const void *data, size_t size, ElfLoader *out)
{
    memset(out, 0, sizeof(*out));

    ElfHeader hdr;
    if (elf_parse_header(data, size, &hdr) != 0)
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
    int envc = 0;
    while (envp && envp[envc])
        envc++;

    char *stk = (char *)loader->stack_base + loader->stack_size;
    stk = (char *)((uintptr_t)stk & ~15UL);

    char **new_envp = malloc((envc + 1) * sizeof(char *));
    char **new_argv = malloc((argc + 1) * sizeof(char *));

    for (int i = envc - 1; i >= 0; i--)
    {
        size_t len = strlen(envp[i]) + 1;
        stk -= len;
        memcpy(stk, envp[i], len);
        new_envp[i] = stk;
    }

    new_envp[envc] = NULL;

    for (int i = argc - 1; i >= 0; i--)
    {
        size_t len = strlen(argv[i]) + 1;
        stk -= len;
        memcpy(stk, argv[i], len);
        new_argv[i] = stk;
    }

    new_argv[argc] = NULL;

    stk = (char *)((uintptr_t)stk & ~15UL);

    Aux64 auxv[] =
    {
        { AT_PAGESZ, PAGE_SIZE },
        { AT_BASE, (uintptr_t)loader->base },
        { AT_FLAGS, 0 },
        { AT_ENTRY, (uintptr_t)loader->entry },
        { AT_UID, (uint64_t)getuid() },
        { AT_EUID, (uint64_t)geteuid() },
        { AT_GID, (uint64_t)getgid() },
        { AT_EGID, (uint64_t)getegid() },
        { AT_SECURE, 0 },
        { AT_NULL, 0 },
    };

    int n_aux = (int)(sizeof(auxv) / sizeof(auxv[0]));
    int pw = loader->is_64bit ? 8 : 4;
    int n_slots = 1 + argc + 1 + envc + 1 + n_aux * 2;
    stk -= (size_t)n_slots * (size_t)pw;
    stk = (char *)((uintptr_t)stk & ~15UL);

    char *p = stk;
    PUSH_VAL(argc, loader);
    for (int i = 0; i < argc; i++)
        PUSH_VAL(new_argv[i], loader);
    PUSH_VAL(0, loader);

    for (int i = 0; i < envc; i++)
        PUSH_VAL(new_envp[i], loader);
    PUSH_VAL(0, loader);

    for (int i = 0; i < n_aux; i++)
    {
        PUSH_VAL(auxv[i].type, loader);
        PUSH_VAL(auxv[i].val, loader);
    }

    free(new_argv);
    free(new_envp);

    fprintf(stdout, "%s: entry=%p sp=%p\n", elf_arch_name(loader->arch), loader->entry, stk);

    jump_to_entry(stk, loader->entry);
}

void elf_unload(ElfLoader *loader)
{
    if (loader->base)
        munmap(loader->base, loader->map_size);
    
    if (loader->stack_base)
        munmap(loader->stack_base, loader->stack_size);

    memset(loader, 0, sizeof(*loader));
}

void elf_info(const ElfLoader *loader)
{
    fprintf(stdout,
        "ELF Info:\n"
        "\tArchitecture: %s (%s)\n"
        "\tType: %s\n"
        "\tMapped base: %p\n"
        "\tLoad bias: 0x%lx\n"
        "\tEntry point: %p\n"
        "\tInterp: %s\n"
        "\tNative exec: %s\n",

        elf_arch_name(loader->arch),
        loader->is_64bit ? "ELF64" : "ELF32",
        loader->is_pie ? "PIE (ET_DYN)" : "static (ET_EXEC)",
        loader->base,
        (unsigned long)loader->bias,
        loader->entry,
        loader->interp_offset ? "YES (dynamic)" : "NO (static)",
        elf_is_native(loader->arch) ? "YES" : "NO (cross-arch)"
    );
}

//Execute ELF via memory file descriptor.
int elf_memfd_exec(const void *data, size_t size, int argc, char **argv, char **envp)
{
#ifdef __NR_memfd_create
    /*  Not all Linux systems have this syscall.
        Create an anonymous in-memory file.
    */
    int mfd = (int)syscall(__NR_memfd_create, "elf_mem", 0);
    if (mfd < 0)
    {
        //syscall failed.
        perror("memfd_create");
        return -1;
    }

    /*  Write the entire ELF image into the memfd.
        Using loop because write() may return fewer bytes than requested for Linux systems.
    */ 
    size_t done = 0;
    while (done < size)
    {
        ssize_t n = write(mfd, (unsigned char *)data + done, size - done);
        if (n <= 0)
        {
            //Write error.
            perror("Write into memfd failed.");
            close(mfd);

            return -1;
        }

        done += (size_t)n; //Absoultely positive.
    }

    /*  Directly execute the ELF image from the file descriptor.
        The kernel's ELF binfmt handler takes over from here.
    */
   fexecve(mfd, argv, envp);

   //If it reaches here, fexecve failed.
   perror("fexecve() failed.");
   close(mfd);

   return -1;

#else
    //Kernel doesn't have memfd_create (pre 3.17 or non-Linux)
    fprintf(stderr, "memfd_create (__NR_memfd_create) not available on this kernel.");
    
    //Avoiding warning, learn from: https://gcc.gnu.org/onlinedocs/gcc/Warning-Options.html
    (void)data; (void)size; (void)argc; (void)argv; (void)envp;
    
    return -1;

#endif
}