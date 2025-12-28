/* 
   GCC (MinGW-w64) ile derleme komutu:
   gcc -c _print.s -o _print.obj
*/

.intel_syntax noprefix

.section .data
    stdout_handle:    .quad 0
    bytes_written:    .quad 0
    output_buffer:    .space 4096
    buffer_ptr:       .quad 0
    
    ten_float:        .double 10.0

.section .text
.global _print
.global _strlen
/* -------------------------------------------------------------------------- */
/* _print(format, ...)                                                        */
/* -------------------------------------------------------------------------- */
_print:
    push rbp
    mov rbp, rsp
    push rbx
    push rsi
    push rdi
    push r12
    sub rsp, 32

    /* Argümanları yığına (shadow space) yedekle */
    mov [rbp + 16], rcx     /* Format string*/
    mov [rbp + 24], rdx     /* arg1 */
    mov [rbp + 32], r8      /* arg2 */
    mov [rbp + 40], r9      /* arg3 */

    cmp qword ptr [stdout_handle], 0
    jne .skip_init
    
    mov rcx, -11            /* STD_OUTPUT_HANDLE */
    call GetStdHandle
    mov [stdout_handle], rax
    lea rax, [output_buffer]
    mov [buffer_ptr], rax

.skip_init:
    mov rsi, [rbp + 16]     
    lea rbx, [rbp + 24]     

.loop:
    movzx rax, byte ptr [rsi]
    test al, al
    jz .done
    
    cmp al, '%'
    jne .print_char
    
    inc rsi
    movzx rax, byte ptr [rsi]
    
    mov r12, 6              /* Precision */
    cmp al, '.'
    jne .no_dot
    inc rsi
    movzx rax, byte ptr [rsi]
    sub rax, '0'
    mov r12, rax
    inc rsi
    movzx rax, byte ptr [rsi]
    
.no_dot:
    cmp al, 's'
    je .handle_string
    cmp al, 'd'
    je .handle_int
    cmp al, 'f'
    je .handle_float
    cmp al, 'c'
    je .handle_char
    
    jmp .print_char

.handle_string:
    mov rdi, [rbx]
    add rbx, 8
    call _putstr_buf
    inc rsi
    jmp .loop

.handle_int:
    mov rax, [rbx]
    add rbx, 8
    call _putint_buf
    inc rsi
    jmp .loop

.handle_char:
    mov rax, [rbx]
    add rbx, 8
    call _putchar_buf
    inc rsi
    jmp .loop

.handle_float:
    movq xmm0, qword ptr [rbx]
    add rbx, 8
    mov rdx, r12
    call _putfloat_buf
    inc rsi
    jmp .loop

.print_char:
    mov al, byte ptr [rsi]
    call _putchar_buf
    inc rsi
    jmp .loop

.done:
    call _flush_buffer
    add rsp, 32
    pop r12
    pop rdi
    pop rsi
    pop rbx
    pop rbp
    ret

/* --- Yardımcı Fonksiyonlar --- */

_putchar_buf:
    /* entry: ...8 */
    push rdi        /* ...0 */
    push r11        /* ...8 */
    sub rsp, 8      /* ...0 (Aligned) */
    
    mov rdi, [buffer_ptr]
    mov [rdi], al
    inc rdi
    mov [buffer_ptr], rdi
    lea r11, [output_buffer + 4096]
    cmp rdi, r11
    jl .no_flush
    call _flush_buffer
.no_flush:
    add rsp, 8
    pop r11
    pop rdi
    ret

_putstr_buf:
    push rsi
    mov rsi, rdi
.ps_loop:
    movzx rax, byte ptr [rsi]
    test al, al
    jz .ps_done
    call _putchar_buf
    inc rsi
    jmp .ps_loop
.ps_done:
    pop rsi
    ret

_putint_buf:
    /* entry: ...8 */
    push rdi        /* ...0 */
    push rsi        /* ...8 */
    push rbx        /* ...0 */
    push rdx        /* ...8 */
    push rcx        /* ...0 */
    /* 5 pushes = 40 bytes. ...0 - 40 = ...8. */
    /* To get ...0, we need 8 + 16k. Let's use 8 + 32 = 40. Or 8 + 80 = 88? */
    /* Wait: ...0 - 40 is 8-byte aligned only. 40 = 2*16+8. */
    /* So we need sub rsp, 24 (or 40, 56, etc.) to make it 16-byte aligned. */
    /* Let's use sub rsp, 40. Total space = 40 + 40 = 80. */
    sub rsp, 40 
    
    test rax, rax
    jns .pi_pos
    push rax
    mov al, '-'
    call _putchar_buf
    pop rax
    neg rax
.pi_pos:
    lea rdi, [rsp + 32]     /* Local buffer area */
    mov byte ptr [rdi], 0
    mov rcx, 10
.pi_loop:
    xor rdx, rdx
    div rcx
    add dl, '0'
    dec rdi
    mov [rdi], dl
    test rax, rax
    jnz .pi_loop
.pi_print:
    movzx rax, byte ptr [rdi]
    test al, al
    jz .pi_done
    call _putchar_buf
    inc rdi
    jmp .pi_print
.pi_done:
    add rsp, 40
    pop rcx
    pop rdx
    pop rbx
    pop rsi
    pop rdi
    ret

_putfloat_buf:
    push r12
    push rax
    sub rsp, 40
    mov r12, rdx
    xorpd xmm1, xmm1
    ucomisd xmm0, xmm1
    jae .pf_pos
    mov al, '-'
    call _putchar_buf
    mov rax, 0x7FFFFFFFFFFFFFFF
    movq xmm1, rax
    andpd xmm0, xmm1
.pf_pos:
    cvttsd2si rax, xmm0
    movsd [rsp + 32], xmm0 
    call _putint_buf
    movsd xmm0, [rsp + 32]
    test r12, r12
    jz .pf_done
    mov al, '.'
    call _putchar_buf
    cvttsd2si rax, xmm0
    cvtsi2sd xmm1, rax
    subsd xmm0, xmm1
.pf_loop:
    test r12, r12
    jz .pf_done
    mulsd xmm0, [ten_float]
    cvttsd2si rax, xmm0
    add al, '0'
    call _putchar_buf
    cvttsd2si rax, xmm0
    cvtsi2sd xmm1, rax
    subsd xmm0, xmm1
    dec r12
    jmp .pf_loop
.pf_done:
    add rsp, 40
    pop rax
    pop r12
    ret

_flush_buffer:
    push rbp
    mov rbp, rsp
    push rdi
    push rsi
    sub rsp, 32
    mov rdi, [buffer_ptr]
    lea rsi, [output_buffer]
    sub rdi, rsi
    jz .f_ret
    mov rcx, [stdout_handle]
    lea rdx, [output_buffer]
    mov r8, rdi
    lea r9, [bytes_written]
    mov qword ptr [rsp + 32], 0
    call WriteFile
    lea rax, [output_buffer]
    mov [buffer_ptr], rax
.f_ret:
    add rsp, 32
    pop rsi
    pop rdi
    pop rbp
    ret

_strlen:
    xor rax, rax
.sl_loop:
    cmp byte ptr [rcx + rax], 0
    je .sl_done
    inc rax
    jmp .sl_loop
.sl_done:
    ret
