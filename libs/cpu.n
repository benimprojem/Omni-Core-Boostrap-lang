// libs/cpu.nim
//
// Bu modül, kod üretimi (codegen) aşamasında doğrudan assembly komutlarına
// çevrilecek olan yerleşik (intrinsic) fonksiyonları ve sabitleri tanımlar.
// Bu fonksiyonların NIM dilinde bir gövdesi yoktur, sadece derleyiciye
// imzaları bildirilir.

export group cpu {
    // --- x86-64 Register Sabitleri ---
    // Bu sabitlerin tipleri derleyici tarafından özel olarak "Register"
    // olarak algılanır. Değerleri önemli değildir.
    pub const RAX: i64 = 0;
    pub const RBX: i64 = 1;
    pub const RCX: i64 = 2;
    pub const RDX: i64 = 3;
    pub const RSI: i64 = 4;
    pub const RDI: i64 = 5;
    pub const RBP: i64 = 6;
    pub const RSP: i64 = 7;
    pub const R8:  i64 = 8;
    pub const R9:  i64 = 9;
    pub const R10: i64 = 10;
    pub const R11: i64 = 11;
    pub const R12: i64 = 12;
    pub const R13: i64 = 13;
    pub const R14: i64 = 14;
    pub const R15: i64 = 15;

    // --- Yerleşik (Intrinsic) Fonksiyonlar ---
    // Bu fonksiyonlar `extern` olarak bildirilse de, codegen tarafından
    // özel olarak işlenip assembly komutlarına dönüştürülecektir.
    pub extern fn mov(dest: any, src: any): void;
    pub extern fn syscall(): void;
}