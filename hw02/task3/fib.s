.section .data
    format: .asciz "Fibonacci(%d) = %d\n"

.section .text
.globl main
main:
    pushq %rbp
    movq %rsp, %rbp

    movq $10, %rdi     # n = 10

    movq $0, %rbx      # F(0) = 0 в rbx храним окончательный результат
    cmpq $0, %rdi
    je output

    movq $0, %rax      # F(0) = 0
    movq $1, %rbx      # F(1) = 1
    movq $1, %rcx      # i = 1


loop:
    cmpq %rcx, %rdi
    je output

    movq %rbx, %rdx   # сохраняем F(i) для следующего шага
    addq %rax, %rbx   # F(i+1) = F(i) + F(i+1)
    movq %rdx, %rax   # восстанавливаем F(i) для следующего шага

    incq %rcx
    jmp loop

output:
    # Печатаем результат через libc на консоль
    movq %rdi, %rsi
    leaq format(%rip), %rdi
    movq %rbx, %rdx
    call printf

    # возвращаем результат, как exit code
    movq %rbx, %rax
    popq %rbp
    ret
