# NIM (NIMBLE) Derleyici Projesi - Görev Listesi (Güncel)

## ✅ Tamamlananlar (Temel Altyapı)
- [x] **Lexer & Parser:** Temel sözdizimi, bloklar, yorumlar.
- [x] **Type Checker:** Statik tip sistemi, typedef, struct/group tanımları.
- [x] **Codegen (Temel):** GCC/GAS uyumlu ilk çıktılar (Intel Syntax).
- [x] **Build Pipeline:** NASM çıkarılıp doğrudan GCC (GAS) entegrasyonu.

## 🚧 Yakın Plan (Aşama 3.5: Eksik Kontrol Yapıları & Sözdizimi Esnekliği)
- [x] **Sözdizimi Esnekliği:**
    - [x] `var x = 5` (Tip çıkarımı/Inference)
    - [x] `for i in 1..5` (Opsiyonel parantez)
- [x] **Gelişmiş Döngüler:**
    - [x] C-Tarzı `for (init, cond, inc)`
    - [x] Range `for i in 0..10`
    - [x] Foreach `for x in list`
        - [x] Type Checker: `ArrayLiteral` desteği
        - [x] Codegen: `Expr::ArrayLiteral` implementasyonu
        - [x] Codegen: `VarDecl` array stack allocation
        - [x] Codegen: `arrlen` built-in
        - [x] Type::Arr heterogeneous array desteği
- [x] **ULTRA: Stil Sistemi & Formatlama:**
    - [x] Float Fix (0.000000)
    - [x] [style](file:///c:/Users/Asus/Desktop/Nimble/src/parser.rs#182-210) anahtar kelimesi ve parser desteği.
    - [x] Stil tablosu (Registry) ve Tip Kontrolü.
    - [x] Codegen lookup ve dinamik ANSI desteği.

## 🔴 KRİTİK ÖNCELİK (Faz 1-2: Temel Eksiklikler)
- [ ] **Bitwise Operatörler (Codegen):**
    - [ ] `&` (BitwiseAnd) - `and rax, rbx`
    - [ ] `|` (BitwiseOr) - `or rax, rbx`
    - [ ] `^` (BitwiseXor) - `xor rax, rbx`
    - [ ] `<<` (LShift) - TypeChecker + Codegen
    - [ ] `>>` (RShift) - TypeChecker + Codegen
    - [ ] `~` (BitwiseNot) - Parser + TypeChecker + Codegen
- [ ] **Unary Operatörler (Codegen):**
    - [ ] `++x` (PreInc) - Codegen
    - [ ] `--x` (PreDec) - Codegen
- [ ] **Never Tipi:**
    - [ ] TypeChecker: `panic`, [exit](file:///c:/Users/Asus/Desktop/Nimble/src/codegen.rs#226-241) için dönüş tipi
    - [ ] Codegen: Unreachable kod işaretleme
- [ ] **StructLiteral İfadesi:**
    - [ ] Codegen: `Point { x: 10, y: 20 }` syntax desteği
    - [ ] Stack allocation ve field initialization
- [ ] **Pointer Semantiği:**
    - [ ] `Ptr<T>` tipi - TypeChecker + Codegen
    - [ ] `&` (AddressOf) operatörü - TypeChecker + Codegen
    - [ ] `*` (Deref) operatörü - TypeChecker + Codegen
    - [ ] Pointer aritmetiği

## 🟡 ÖNEMLİ ÖNCELİK (Faz 3-4: Gelişmiş Özellikler)
- [ ] **Desen Eşleştirme:**
    - [ ] [match](file:///c:/Users/Asus/Desktop/Nimble/src/parser.rs#856-891) ifadesinin kod üretimi (Codegen)
    - [ ] Pattern matching: literal, variable, wildcard
    - [ ] Exhaustiveness checking
- [ ] **Enum Codegen:**
    - [ ] Enum variant değerleri
    - [ ] `EnumAccess` ifadesi codegen
    - [ ] Tag-based representation
- [ ] **Tuple Desteği:**
    - [ ] `Tuple(Vec<Type>)` tipi - TypeChecker
    - [ ] Tuple literal ve destructuring - Codegen
    - [ ] Çoklu değer dönüşü
- [ ] **Ternary Operator:**
    - [ ] `Conditional { cond, then, else }` - Codegen
    - [ ] `cond ? then_val : else_val` syntax
- [ ] **Struct Tamamlama:**
    - [ ] `MemberAccess` codegen tamamlama
    - [ ] Nested struct desteği
- [ ] **Ref<T> Tipi:**
    - [ ] Referans semantiği - TypeChecker + Codegen
    - [ ] Borrow checking (basit)

## 📅 Gelecek Planı (Faz 5-7: İleri Özellikler)
- [ ] **Lambda ve First-Class Fonksiyonlar:**
    - [ ] `Lambda { params, return_type, body }` - TypeChecker + Codegen
    - [ ] `Fn(Vec<Type>, Box<Type>)` tipi
    - [ ] Closure desteği
- [ ] **Hata Yönetimi:**
    - [ ] `Result<T, E>` tipi - Parser + TypeChecker + Codegen
    - [ ] `Try (expr?)` ifadesi - Parser + TypeChecker + Codegen
    - [ ] `Option<T>` tipi
    - [ ] [match](file:///c:/Users/Asus/Desktop/Nimble/src/parser.rs#856-891) ile zorunlu kontrol
- [ ] **Inline Assembly:**
    - [ ] `Asm { tag, body }` deyimi - Codegen
    - [ ] GAS syntax desteği
    - [ ] Register allocation
- [ ] **Async/Await:**
    - [ ] `Future<T>` tipi
    - [ ] `Await` ifadesi
    - [ ] Runtime integration
- [ ] **Concurrency:**
    - [ ] `Channel<T>` tipi
    - [ ] `Send`/`Recv` ifadeleri
    - [ ] Thread primitives
- [ ] **Unsafe & FastExec:**
    - [ ] `Unsafe` bloğu
    - [ ] `FastExec` bloğu
    - [ ] Raw pointer operations
- [ ] **Bellek Yönetimi (İleri):**
    - [ ] Dinamik diziler için `heap` yönetimi ([push](file:///c:/Users/Asus/Desktop/Nimble/src/type_checker.rs#105-109), [pop](file:///c:/Users/Asus/Desktop/Nimble/src/type_checker.rs#110-118), `count`)
    - [ ] `memory` modülü: `alloc`, `free`, `read<T>`, `write<T>` rutinleri
- [ ] **Donanım Erişimi:**
    - [ ] `cpu` modülü: `rdtsc`, `pause`, `core_count`
- [ ] **SIMD & Matematik:**
    - [ ] `Vec2/3/4` veri tipleri ve temel operatör aşırı yüklemesi
- [ ] **Blok Yönetimi:**
    - [ ] `rolling` bloğu ve `$rolling` değişkeni mekanizması
    - [ ] [Group](file:///c:/Users/Asus/Desktop/Nimble/src/type_checker.rs#16-22) tam desteği

## 🟢 DÜŞÜK ÖNCELİK (Nice-to-Have)
- [ ] **Genişletilmiş Tipler:**
    - [ ] `F80`, `F128` Codegen
    - [ ] `D32`, `D64`, `D128` Decimal tipler
    - [ ] `Bit`, `Byte` tip kontrolü
- [ ] **Utility Operatörler:**
    - [ ] `SizeOf(Type)` operatörü
- [ ] **NIMBLE Özel:**
    - [ ] `Routine` semantiği (spec gerekli)
    - [ ] `Tag` ve `LabeledStmt` tam desteği

## 🚀 Test ve Stabilizasyon (Aşama 5)
- [x] **Kapsamlı Test Suite:** 20 test dosyası oluşturuldu (test1-test20)
    - [x] Temel literaller ve değişkenler (test1-test2)
    - [x] Operatörler: Aritmetik, karşılaştırma, mantıksal, unary (test3-test4, test16-test17)
    - [x] Kontrol yapıları: if-else, while, for-range, for-in (test5-test7, test11-test12)
    - [x] Fonksiyonlar ve recursion (test8)
    - [x] String işlemleri ve interpolation (test9)
    - [x] Array: tanımlama, erişim, for-in, algoritmalar (test10-test11, test19)
    - [x] Struct tanımlama ve üye erişimi (test13)
    - [x] Tip dönüşümleri (test14)
    - [x] İç içe yapılar: Fibonacci, asal sayı (test15)
    - [x] ANSI stil sistemi (test18)
    - [x] Kapsamlı entegrasyon testi (test20)
- [ ] **Standard Lib:** `io.n`, `math.n`, `string.n` fiziksel dosya entegrasyonu

---

**Sonraki Adım:** Kritik öncelik listesinden başlayarak implementasyona devam et.
