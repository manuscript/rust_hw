# clang -no-pie sum.s -o sum

.section .data
    format: .asciz "Result: %d\n"

.section .text
.globl main
main:
    # Сохраняем %rbp — это выравнивает стек по 16 байтам
    pushq %rbp
    movq %rsp, %rbp      # Устанавливаем новый базовый указатель

    # Регистры и их назначение:
    # %rbx — первое слагаемое (число 25)
    # %rcx — второе слагаемое (число 27)
    # %eax — код возврата из main (результат сложения, 32 бита)

    movq $25, %rbx          # Первое число: 25
    movq $27, %rcx          # Второе число: 27
    addq %rcx, %rbx         # %rbx = %rbx + %rcx = 25 + 27 = 52

    # Подготавливаем аргументы для printf
    leaq format(%rip), %rdi  # %rdi = указатель на строку формата
    movl %ebx, %esi         # %esi = результат сложения (52), 32-битная версия %rbx
    call printf              # Вызываем printf для вывода результата

    # Копируем результат в %eax (код завершения, 32 бита)
    movl %ebx, %eax         # %eax = 52 (код завершения)

    # Восстанавливаем стек и базовый указатель
    popq %rbp
    ret
