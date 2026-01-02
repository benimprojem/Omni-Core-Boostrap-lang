/* 
   GCC (MinGW-w64) ile derleme komutu:
   gcc -c core.s -o core.obj
*/

.intel_syntax noprefix

.section .data
    stdout_handle:    .quad 0
    bytes_written:    .quad 0
    output_buffer:    .space 4096
    buffer_ptr:       .quad 0
    _fmt_float_str: .asciz "%f"
    _conv_buffer: .space 64
    ten_float:        .double 10.0

.section .text
.global _print
.global _strlen
.global _input
.global _sprint
.global _fmod

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
    # 4 push = 32 byte. 
    # Return adresi (8) + RBP (8) + 32 = 48 byte. 
    # 48, 16'nın katıdır. Hizalama şu an mükemmel.
    # Ekstra 'sub rsp' yapmadan devam edebiliriz ya da 
    # hizalamayı bozmamak için 16'nın katı (16, 32) eklemeliyiz.
    sub rsp, 32 

    # Argümanları shadow space'e yedekle (Bu alan çağıran tarafından RSP+40'ta hazırlandı)
    mov [rbp + 16], rcx     # Format
    mov [rbp + 24], rdx     # Arg 1 (Double bitleri burada)
    mov [rbp + 32], r8      # Arg 2
    mov [rbp + 40], r9      # Arg 3

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
    # rbx = [rbp + 24] (ilk argüman adresi)
    # Kritik: Float değerleri 64-bit double olarak okuyoruz
    movq xmm0, qword ptr [rbx] 
    add rbx, 8
    mov rdx, r12            # Hassasiyet
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

_fmod:
    # Giriş: XMM0 = a, XMM1 = b
    movups xmm2, xmm0      # xmm2 = a (yedekle)
    divsd xmm0, xmm1       # xmm0 = a / b
    cvttsd2si rax, xmm0    # rax = trunc(a/b) -> tam sayıya çevir (truncate)
    cvtsi2sd xmm0, rax     # xmm0 = rax (tekrar float yap)
    mulsd xmm0, xmm1       # xmm0 = xmm0 * b
    subsd xmm2, xmm0       # xmm2 = a - xmm0
    movups xmm0, xmm2      # Sonuç xmm0'a
    ret

_sprint:
    push rbp
    mov rbp, rsp
    sub rsp, 64             # Yerel değişkenler ve shadow space
    mov rsi, rdx            # RSI = Format String ("%f" veya "%d")
    mov rdi, rcx            # RDI = Hedef Buffer (_conv_buffer)

.Lloop:
    mov al, [rsi]
    test al, al
    jz .Ldone
    cmp al, '%'
    je .Lformat
    mov [rdi], al           # Normal karakteri kopyala
    inc rsi
    inc rdi
    jmp .Lloop

.Lformat:
    inc rsi
    mov al, [rsi]
    cmp al, 'f'
    je .Lhandle_float
    # Buraya gerekirse %d eklenebilir
    jmp .Lloop

.Lhandle_float:
    # XMM2'deki float değerini tam ve ondalık olarak ayırıp basitleştirilmiş çevrim
    cvttsd2si rax, xmm2      # RAX = Tam kısım
    # --- Tam Kısım Çevrimi (inline itoa) ---
    mov r8, rdi              # Başlangıcı sakla
    mov rbx, 10
.Litoa_int:
    xor rdx, rdx
    div rbx
    add dl, '0'
    mov [rdi], dl
    inc rdi
    test rax, rax
    jnz .Litoa_int
    # Ters çevir
    mov r9, rdi
    dec r9
.Lrev1:
    cmp r8, r9
    jae .Ldot
    mov al, [r8]
    mov bl, [r9]
    mov [r8], bl
    mov [r9], al
    inc r8
    dec r9
    jmp .Lrev1

.Ldot:
    mov byte ptr [rdi], '.'
    inc rdi
    
    # --- Ondalık Kısım (6 basamak) ---
    cvtsi2sd xmm0, rax       # Az önce çevrilen tam kısmı geri al (Hatalıysa cvttsd2si'den önceki yedeği kullan)
    movups xmm0, xmm2
    cvttsd2si rax, xmm0
    cvtsi2sd xmm1, rax
    subsd xmm0, xmm1         # XMM0 = 0.xxxx
    # Mutlak değer
    pcmpeqd xmm1, xmm1
    psrlq xmm1, 1
    andpd xmm0, xmm1
    # Çarp ve çevir
    mov rax, 1000000
    cvtsi2sd xmm1, rax
    mulsd xmm0, xmm1
    cvttsd2si rax, xmm0      # Ondalık kısım artık tam sayı
    
    mov r8, rdi
.Litoa_frac:
    xor rdx, rdx
    div rbx
    add dl, '0'
    mov [rdi], dl
    inc rdi
    test rax, rax
    jnz .Litoa_frac
    # Ters çevir
    mov r9, rdi
    dec r9
.Lrev2:
    cmp r8, r9
    jae .Lnext
    mov al, [r8]
    mov bl, [r9]
    mov [r8], bl
    mov [r9], al
    inc r8
    dec r9
    jmp .Lrev2

.Lnext:
    inc rsi
    jmp .Lloop

.Ldone:
    mov byte ptr [rdi], 0
    leave
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

_input:
    sub rsp, 56
    mov rbx, rcx                # Prompt adresini yedekle
    
    # 1. Eğer prompt varsa ekrana yazdır (Daha önce yazdığın _print'i çağır)
    test rcx, rcx
    jz .L_no_prompt
    call _print
    
.L_no_prompt:

    # 2. Stdin Handle'ını al
    mov rcx, -10                # STD_INPUT_HANDLE
    call GetStdHandle
    mov rsi, rax                # RSI = Stdin Handle

    # 3. Buffer ayır (Derleyici içi malloc/alloc çağrısı)
    mov rcx, 1024               # Sabit 1KB veya dinamik
    call _alloc_str             # RAX = Yeni String Buffer
    mov rdi, rax                # RDI = Buffer adresi

    # 4. Konsoldan oku (ReadConsoleA)
    mov rcx, rsi                # Handle
    mov rdx, rdi                # Buffer
    mov r8, 1024                # Max length
    lea r9, [rsp + 32]          # lpNumberOfCharsRead
    mov qword ptr [rsp + 40], 0 # lpReserved = NULL (Stack üzerinden 5. parametre)
    call ReadConsoleA

    # 5. CRLF (\r\n) temizleme
    mov r8, [rsp + 32]          # Okunan karakter sayısı
    sub r8, 2                   # \r\n karakterlerini atla
    mov byte ptr [rdi + r8], 0  # Null terminator ekle
    
    mov rax, rdi                # Dönüş değeri: String adresi
    add rsp, 56
    ret
    
# --- _alloc_str(size: RCX) -> RAX (Pointer) ---
_alloc_str:
    sub rsp, 40
    mov r8, rcx             # R8 = İstenen boyut (bytes)
    
    # 1. İşlemin heap handle'ını al
    call GetProcessHeap     # RAX = Heap Handle
    
    # 2. HeapAlloc(hHeap: RCX, dwFlags: RDX, dwBytes: R8)
    mov rcx, rax            # RCX = Heap Handle
    mov rdx, 8              # RDX = HEAP_ZERO_MEMORY (0x00000008)
    # R8 zaten boyut bilgisini tutuyor
    call HeapAlloc          # RAX = Ayrılan bellek adresi
    
    add rsp, 40
    ret
