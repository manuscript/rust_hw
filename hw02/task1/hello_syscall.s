# clang -nostdlib -no-pie hello_syscall.s -o hello_syscall

.section .rodata
msg:
    .ascii "Hello, world!\n"
msglen = . - msg

.section .text
.globl _start
_start:
    # Системный вызов write(1, msg, msglen)
    movq $1, %rax      # номер системного вызова write (1)
    movq $1, %rdi     # файловый дескриптор stdout (1)
    movq $msg, %rsi   # указатель на строку
    movq $msglen, %rdx # длина строки
    syscall

    # Системный вызов exit(0)
    movq $60, %rax   # номер системного вызова exit (60)
    xorq %rdi, %rdi   # код возврата 0 (обнуляем RDI)
    syscall

