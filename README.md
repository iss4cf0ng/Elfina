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
make

./elfina --info <x64_elf_path>
./elfina --mmap <x64_elf_path> [arguments]
./elfina --memfd <x64_elf_path> [arguments]

./elfina32 --info <x86_elf_path>
./elfina32 --mmap <x86_elf_path> [arguments]
./elfina32 --memfd <x86_elf_path> [arguments] 
```

