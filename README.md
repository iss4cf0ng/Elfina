# Elfina: A Multi-Architecture ELF Loader

![License](https://img.shields.io/github/license/iss4cf0ng/Elfina)
![Stars](https://img.shields.io/github/stars/iss4cf0ng/Elfina)
![Issues](https://img.shields.io/github/issues/iss4cf0ng/Elfina)

Elfina is a multi-architecture ELF loader supporting x86 and x86-64 binaries.

## Background
Recently, I have been studying reverse engineering on Windows and the PE file format.  
After that, I started exploring ELF binaries to learn more about Linux reverse engineering and rootkit development.

To better understand how ELF executables are loaded and executed, I developed **Elfina** as a learning project focused on the Linux kernel and the ELF file format.

<p align="center">
    <img src="https://iss4cf0ng.github.io/images/meme/himari_elf.png" width="700">
</p>

<p align="center">
    If you find this project useful or informative, a ⭐ would be appreciated!
</p>

## Disclaimer

This project is intended for **educational and research purposes only**.

It is designed to help understand:

- ELF file format
- Reverse engineering concepts

<p align="center">
    <img src="https://iss4cf0ng.github.io/images/meme/mika_rollcake_hit.png" width=300>
</p>

## Features
- Multi-architecture ELF loader
- Support for x86 and x86-64 ELF binaries
- Multiple execution methods
    - `--mmap` loading
    - `--memfd` execution
- ELF probing
  - `--info` displays ELF metadata and structure

## Supported ELF Architectures

| Architecture | Bits | Common Devices |
| --- | -- | --- |
| x86 (i386) | 32-bit | Old PCs, 32-bit Linux |
| x86-64 | 64-bit | Modern PCs, servers |
| ARM32 | 32-bit | Raspberry Pi 2, older Android |
| AArch64 (ARM64) | 64-bit | Raspberry Pi 3/4/5, modern Android |
| RISC-V 64 | 64-bit | SiFive boards, VisionFive, emerging Linux devices |

## Quick Start
### Requirements
```bash
sudo apt install gcc-multilib
```

Download and extract the release package:
```bash
wget https://github.com/iss4cf0ng/Elfina/releases/latest/download/elfina-linux.tar.gz
tar -xzf elfina-linux.tar.gz
cd elfina
chmod +x ./elfina
chmod +x ./elfina32
```

The layout is shown as follows:
```
elfina/
 ├ elfina
 └ elfina32
```

### Usage
```bash
// ---------- elfina (x64) ----------

./elfina --coffee
./elfina --info <x64_elf_path>
./elfina --mmap <x64_elf_path> [arguments]
./elfina --memfd <x64_elf_path> [arguments]

./elfina --hexdump <x64_elf_path>
./elfina --hexdump-out <x64_elf_path>

./elfina --entropy <x64_elf_path>
./elfina --entropy-out <output_file> <x64_elf_path>

./elfina --disasm <x64_elf_path>
./elfina --disasm-out <output_file> <x64_elf_path>

// ---------- elfina32 (x86) ----------

./elfina32 --coffee
./elfina32 --info <x86_elf_path>
./elfina32 --mmap <x86_elf_path> [arguments]
./elfina32 --memfd <x86_elf_path> [arguments]

./elfina32 --hexdump <x86_elf_path>
./elfina32 --hexdump-out <x86_elf_path>

./elfina32 --entropy <x86_elf_path>
./elfina32 --entropy-out <output_file> <x86_elf_path>

./elfina32 --disasm <x86_elf_path>
./elfina32 --disasm-out <output_file> <x86_elf_path>
```

## Build from Source
### C-lang
Clone the repository and compile the project:
```bash
git clone https://github.com/iss4cf0ng/Elfina
cd Elfina/C
make
```
or
```bash
chmod +x build.sh
./build.sh
```

### Rust
```bash
git clone https://github.com/iss4cf0ng/Elfina
cd Elfina/Rust
chmod +x build.sh
./build.sh
```

## Demonstration
### elfina (x86-64)
<p align="center">
    <img src="https://iss4cf0ng.github.io/images/article/2026-3-12-Elfina/1.png" width=700>
</p>

<p align="center">
    <img src="https://iss4cf0ng.github.io/images/article/2026-3-12-Elfina/2.png" width=700>
</p>

---

### elfina32 (32-bit)
<p align="center">
    <img src="https://iss4cf0ng.github.io/images/article/2026-3-12-Elfina/3.png" width=700>
</p>

On Windows Subsystem for Linux (WSL2), Elfina cannot execute 32-bit ELF binary files. Probing (`--info`) and 64-bit execution work fine. For full 32-bit support, use a native Linux environment or a VM such as VirtualBox or VMWare.
<p align="center">
    <img src="https://iss4cf0ng.github.io/images/article/2026-3-12-Elfina/4.png" width=700>
</p>

### `--hexdump`
<p align="center">
    <img src="https://iss4cf0ng.github.io/images/article/2026-3-14-Elfina2.0.0/2.png" width=700>
</p>

### `--entropy`
<p align="center">
    <img src="https://iss4cf0ng.github.io/images/article/2026-3-14-Elfina2.0.0/1.png" width=700>
</p>

### `--disasm`
<p align="center">
    <img src="https://iss4cf0ng.github.io/images/article/2026-3-14-Elfina2.0.0/3.png" width=700>
</p>
