.section .rodata
msg:
    .asciz "Hello, world"

.section .text
.globl main
main:
    # Передаём адрес строки в регистр RDI (первый аргумент функции по ABI)
    leaq msg(%rip), %rdi
    # Вызываем функцию puts из libc
    call puts
    # Возвращаем 0 из main (код успешного завершения)
    #movl $0, %eax
    #xorq %rax, %rax  # RCX = 0 (64 бита)
    xorl %eax, %eax  # EAX = 0 (32 бита)
    ret

