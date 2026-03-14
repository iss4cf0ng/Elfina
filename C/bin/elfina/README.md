# Elfina
Elfina is an ELF file loader for multiple platforms, including x86 and x64 architecture.

<p align="center">
    <img src="https://iss4cf0ng.github.io/images/meme/himari_elf.png" width="800">
</p>

## Background
This project is allow

## Quick Start
### Requirement
```
sudo apt install gcc-multilib
```

### Usage
```
wget https://github.com/iss4cf0ng/Elfina/releases/latest/download/elfina-linux.tar.gz
tar -xzf elfina-linux.tar.gz
cd ./elfina

./elfina --coffee
./elfina --info <x64_elf_path>
./elfina --mmap <x64_elf_path> [arguments]
./elfina --memfd <x64_elf_path> [arguments]

./elfina32 --coffee
./elfina32 --info <x86_elf_path>
./elfina32 --mmap <x86_elf_path> [arguments]
./elfina32 --memfd <x86_elf_path> [arguments] 
```

## Demonstration
### elfina (64-bit)
<p align="center">
    <img src="https://iss4cf0ng.github.io/images/article/2026-3-12-Elfina/1.png" width=800>
</p>

<p align="center">
    <img src="https://iss4cf0ng.github.io/images/article/2026-3-12-Elfina/2.png" width=800>
</p>

### elfina32 (32-bit)
<p align="center">
    <img src="https://iss4cf0ng.github.io/images/article/2026-3-12-Elfina/1.png" width=800>
</p>

<p align="center">
    <img src="https://iss4cf0ng.github.io/images/article/2026-3-12-Elfina/1.png" width=800>
</p>

On Windows Subsystem for Linux (WSL2), Elfina cannot execute 32-bit ELF binary files. Probing (`--info`) and 64-bit execution work fine. For full 32-bit support, use a native Linux environment or a VM such as VirtualBox or VMWare.
<p align="center">
    <img src="https://iss4cf0ng.github.io/images/article/2026-3-12-Elfina/1.png" width=800>
</p>