#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/stat.h>

#include "elf_loader.h"

void usage(const char *app)
{
    fprintf(stderr, 
        "Usage: %s [--memfd|--mmap||--info] <elf_binary> [args...]\n"
        "\n"
        "\t--memfd \n"
        "\t--mmap \n"
        "\t--info \n"
        "\n"
        "Host architecture: %s\n",
        app, ELF_HOST_NAME
    );
}

void *read_elf(const char *elf_path, size_t *out_size)
{
    int fd = open(elf_path, O_RDONLY); //Open, read-only
    if (fd < 0)
    {
        //Open failed.
        perror("Open ELF failed.");
        return NULL;
    }

    //Query file status using fstate()
    struct stat st;
    fstat(fd, &st);
    *out_size = (size_t)st.st_size; //Memory size.

    void *buffer = malloc(*out_size); //Allocate memory.
    if (!buffer)
    {
        //Allocate failed.
        fprintf(stderr, "malloc(%zu) failed\n", *out_size);
        close(fd);

        return NULL;
    }

    size_t done = 0;
    while (done < *out_size)
    {
        ssize_t n = read(fd, (unsigned char *)buffer + done, *out_size - done);
        if (n <= 0)
        {
            //read() failed.
            perror("read");
            free(buffer);
            close(fd);

            return NULL;
        }

        done += (size_t)n; //Absolutely positive value.
    }

    close(fd);

    return buffer;
}

int main(int argc, char **argv, char **envp)
{
    if (argc < 2) 
    {
        usage(argv[0]);
        return 1;
    }

    int force_memfd = 0; //--memfd: always use memfd path
    int force_mmap = 0; //--mmap: always use manual mmap path
    int info_only = 0; //--info: inspect but don't execute
    int i = 1; //argv index

    for (; i < argc; i++)
    {
        if (strcmp(argv[i], "--memfd") == 0)
            force_memfd = 1;
        else if (strcmp(argv[i], "--mmap") == 0)
            force_mmap = 1;
        else if (strcmp(argv[i], "--info") == 0)
            info_only = 1;
        else
            break;
    }

    if (i >= argc)
    {
        //Too many arguments
        usage(argv[0]);
        return 1;
    }

    const char *elf_path = argv[i]; //file path of ELF, last argument

    //Read ELF file into memory.
    size_t elf_size = 0;
    void *elf_buffer = read_elf(elf_path, &elf_size);
    if (!elf_buffer)
    {
        fprintf(stderr, "Process terminated.");
        return 1;
    }

    //Probe the ELF (not mapping yet!).
    ElfLoader loader;
    if (elf_probe(elf_buffer, elf_size, &loader) != 0)
    {
        fprintf(stderr, "Invalid ELF file\n");
        free(elf_buffer);

        return 1;
    }

    //Print summary.
    fprintf(stdout, "arch=%-12s class=ELF%d\t%s\t%s\n", elf_arch_name(loader.arch), loader.is_64bit ? 64 : 32, loader.is_pie ? "PIE" : "ET_EXEC", loader.interp_offset ? "dynamic" : "static");
    
    //Functionalities

    //Information
    if (info_only)
    {
        if (elf_load(elf_buffer, elf_size, &loader) == 0)
            elf_info(&loader);
        
        free(elf_buffer);

        return 0; //Exit
    }

    //Execution
    /* Logic:
        --memfd flag explicitly set -> use memfd path
        binary has PT_INTERP (Program Header Type: Interpreter, needs ld-linux.so) -> must use memfd
    */
    int use_memfd = force_memfd || (!force_mmap && loader.interp_offset);
    if (use_memfd)
    {
        fprintf(stdout, "Executing via memfd_create + fexecve\n");
        elf_memfd_exec(elf_buffer, elf_size, argc - i, argv + i, envp);

        //If it reaches here, elf_memfd_exec() failed.
        free(elf_buffer);

        return 1;
    }

    fprintf(stdout, "Executing via manual mmap loader\n");

    if (!elf_is_native(loader.arch))
    {
        fprintf(stderr, "Binary is %s but host is %s.\n", elf_arch_name(loader.arch), ELF_HOST_NAME);
        free(elf_buffer);

        return 1;
    }

    if (elf_load(elf_buffer, elf_size, &loader) != 0)
    {
        fprintf(stderr, "elf_load() failed\n");
        free(elf_buffer);

        return 1;
    }

    free(elf_buffer);

    elf_execute(&loader, argc - i, argv + i, envp);

    //If it reaches here, elf_execute() failed.
    fprintf(stderr, "elf_execute() failed.");
    elf_unload(&loader);

    return 1;
}