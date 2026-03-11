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

int main(int argc, char **argv, char **envp)
{
    if (argc < 2) 
    {
        usage(argv[0]);
        return 1;
    }


}