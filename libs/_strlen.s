/* 
   GCC (MinGW-w64) ile derleme komutu:
   gcc -c _strlen.s -o _strlen.obj
*/
.intel_syntax noprefix

.section .data

.section .text

.global _strlen

_strlen:
    xor rax, rax
.sl_loop:
    cmp byte ptr [rdi + rax], 0
    je .sl_done
    inc rax
    jmp .sl_loop
.sl_done:
    ret
