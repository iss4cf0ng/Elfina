.section .text
.global _start
_start: 
    # write(1, msg, len)
    mov $1, %rax # syscall number: SYS_write
    mov $1, %rdi # fd : stdout
    lea msg(%rip), %rsi # pointer to the message string
    mov $len, %rdx # length in bytes
    syscall

    # exit(0)
    mov $60,  %rax # syscall number: SYS_exit
    xor %rdi, %rdi # exit code     : 0
    syscall

msg: .ascii "Hello from raw-ASM ELF (no libc, mmap loader)!\n"
len = .- msg