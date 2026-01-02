# io.s - Full Windows x64 I/O Library (Register Annotated)
.intel_syntax noprefix
.text

# Kernel32 API Externals
.extern CreateFileA
.extern CloseHandle
.extern ReadFile
.extern WriteFile
.extern SetFilePointerEx
.extern GetFileSizeEx
.extern DeleteFileA
.extern CopyFileA
.extern GetFileAttributesA
.extern FlushFileBuffers

# yeni externals
.extern GetStdHandle
.extern ReadConsoleA
.extern SetConsoleMode
.extern GetConsoleMode

# Exported Symbols
.global _io_open
.global _io_close
.global _io_read
.global _io_write
.global _io_seek
.global _io_size
.global _io_exists
.global _io_remove
.global _io_copy
.global _io_flush

# --- _io_open(path: RCX, access: RDX, share: R8, create: R9) ---
_io_open:
    sub rsp, 56
    mov qword ptr [rsp + 48], 0     # hTemplateFile (Stack)
    mov qword ptr [rsp + 40], 128   # FILE_ATTRIBUTE_NORMAL (Stack)
    mov [rsp + 32], r9              # dwCreationDisposition (Stack)
    # RCX, RDX, R8 zaten register'larda hazır
    call CreateFileA
    add rsp, 56
    ret

# --- _io_close(handle: RCX) ---
_io_close:
    sub rsp, 40
    call CloseHandle
    add rsp, 40
    ret

# --- _io_read(handle: RCX, buf: RDX, len: R8, bytesReadPtr: R9) ---
_io_read:
    sub rsp, 40
    mov qword ptr [rsp + 32], 0     # lpOverlapped (Stack)
    call ReadFile
    add rsp, 40
    ret

# --- _io_write(handle: RCX, buf: RDX, len: R8, bytesWrittenPtr: R9) ---
_io_write:
    sub rsp, 40
    mov qword ptr [rsp + 32], 0     # lpOverlapped (Stack)
    call WriteFile
    add rsp, 40
    ret

# --- _io_seek(handle: RCX, offset: RDX, origin: R8) ---
_io_seek:
    sub rsp, 40
    xor r9, r9                      # lpNewFilePointer = NULL (RCX, RDX, R8 hazır)
    call SetFilePointerEx
    add rsp, 40
    ret

# --- _io_size(handle: RCX, sizePtr: RDX) ---
_io_size:
    sub rsp, 40
    call GetFileSizeEx
    add rsp, 40
    ret

# --- _io_exists(path: RCX) ---
_io_exists:
    sub rsp, 40
    call GetFileAttributesA
    cmp eax, -1                     # INVALID_FILE_ATTRIBUTES kontrolü
    setne al                        # Eşit değilse 1 (true)
    movzx rax, al
    add rsp, 40
    ret

# --- _io_remove(path: RCX) ---
_io_remove:
    sub rsp, 40
    call DeleteFileA
    add rsp, 40
    ret

# --- _io_copy(src: RCX, dest: RDX) ---
_io_copy:
    sub rsp, 40
    mov r8, 0                       # bFailIfExists = FALSE
    call CopyFileA
    add rsp, 40
    ret

# --- _io_flush(handle: RCX) ---
_io_flush:
    sub rsp, 40
    call FlushFileBuffers
    add rsp, 40
    ret
	
# --- _io_get_std(type: RCX) -> RAX (Handle) ---
# type: -10 (stdin), -11 (stdout), -12 (stderr)
_io_get_std:
    sub rsp, 40
    call GetStdHandle
    add rsp, 40
    ret

# --- _io_read_console(handle: RCX, buf: RDX, len: R8, readPtr: R9) ---
_io_read_console:
    sub rsp, 40
    mov qword ptr [rsp + 32], 0     # pInputControl = NULL
    call ReadConsoleA
    add rsp, 40
    ret