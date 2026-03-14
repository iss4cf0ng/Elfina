use std::ptr;
use libc::{
    mmap, munmap, mprotect, fexecve, write, close,
};

use crate::elf_arch::{
    ElfArch, ElfHeader, ElfPhdr,

};

const PAGE_SIZE: usize = 4096;