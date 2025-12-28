//use crate::token::TokenType;
//use std::collections::HashMap;

// YENİ: Derleme hedefini belirten enum.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TargetPlatform {
    Windows,
    Linux,
    Macos,
    Unknown, // Varsayılan veya belirtilmemiş
}

// Tipi temsil eden enum
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    // 1. Sayısal Tipler
    // İşaretli Tam Sayılar
    I8, I16, I32, I64, I128,
    // İşaretsiz Tam Sayılar
    U8, U16, U32, U64, U128,
    // Kayan Noktalı Sayılar
    F32, F64, F80, F128,
    // Decimal Tipler ve Hex
    D32, D64, D128,
    Hex,

    // 2. İlkel Tipler
    Bool,
    Char,
    Bit,
    Byte,
    Null,
    Void, // Fonksiyon dönüş tipi

    // 3. Birleşik ve Gelişmiş Tipler
    
    // Tuple Tipleri: (i32, str) gibi birden çok tipin birleşimi
    // YENİ: Never tipi, program akışını sonlandıran fonksiyonlar için (exit, panic)
    Never,
    Tuple(Vec<Type>),
    
    // Dizi Tipleri: (İç Tip, Opsiyonel Sabit Boyut)
    // T[] (Dinamik) veya T[N] (Sabit boyutlu dizi) için kullanılır.
    Array(Box<Type>, Option<usize>), 

    // YENİ: 'arr' anahtar kelimesini temsil eden genel dizi tipi.
    Arr,

    // YENİ: Tip kontrolü sırasında bir dizi literalinin tipini temsil eder.
    ArrayLiteral(Vec<Type>),
    
    // Referans ve İşaretçi Tipleri
    Ptr(Box<Type>), // İşaretçi (*)
    Ref(Box<Type>), // Referans (&)

    // String: (Opsiyonel uzunluk kısıtlaması için Some(u64), None ise varsayılan/dinamik)
    Str(Option<usize>), 

    // Özel Tipler
    Any, // Değişken/Karma tip
    Custom(String), // Kullanıcı tanımlı tipler (Struct, Enum vb.)
    // YENİ: Enum tipini ve temel aldığı tamsayı tipini saklar.
    // Örn: Enum("HttpStatus", Box(I32))
    Enum(String, Box<Type>),
	// YENİ:
    Fn(Vec<Type>, Box<Type>),
    Future(Box<Type>),
    Channel(Box<Type>),
    Result(Box<Type>, Box<Type>), // Result<T, E>
    Unknown, // Tip çıkarılamadığında
}

impl Type {
    // YENİ: is_array() yardımcı fonksiyonu
    pub fn is_array(&self) -> bool {
        matches!(self, Type::Array(_, _))
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Type::F32 | Type::F64 | Type::F80 | Type::F128)
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::I128 |
                       Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::U128)
    }

    pub fn is_unsigned_integer(&self) -> bool {
        matches!(self, Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::U128)
    }

    pub fn can_be_assigned_from(&self, other: &Type) -> bool {
        if self == other {
            return true;
        }

        match (self, other) {
            // Güvenli float upcasting: Küçük tipten büyük tipe atama
            (Type::F64, Type::F32) |
            (Type::F80, Type::F32) | (Type::F80, Type::F64) |
            (Type::F128, Type::F32) | (Type::F128, Type::F64) | (Type::F128, Type::F80) => true,

            // Güvenli integer upcasting
            (Type::I16, Type::I8) | (Type::I32, Type::I8) | (Type::I32, Type::I16) |
            (Type::I64, Type::I8) | (Type::I64, Type::I16) | (Type::I64, Type::I32) |
            (Type::I128, Type::I8) | (Type::I128, Type::I16) | (Type::I128, Type::I32) | (Type::I128, Type::I64) => true,

            // Güvenli unsigned integer upcasting
            (Type::U16, Type::U8) | (Type::U32, Type::U8) | (Type::U32, Type::U16) |
            (Type::U64, Type::U8) | (Type::U64, Type::U16) | (Type::U64, Type::U32) |
            (Type::U128, Type::U8) | (Type::U128, Type::U16) | (Type::U128, Type::U32) | (Type::U128, Type::U64) => true,

            // Tamsayıdan float'a atama (genellikle güvenli)
            (t, o) if t.is_float() && o.is_integer() => true,

            // 'any' tipine her şey atanabilir
            (Type::Any, _) => true,
            _ => false,
        }
    }
}

// YENİ: Desteklenen mimariler için CPU register'larını temsil eder.
#[allow(dead_code)] // Codegen aşamasında kullanılacağı için şimdilik uyarıyı bastır.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Register {
    // x86_64 Genel Amaçlı Register'lar
    RAX, RBX, RCX, RDX,
    RSI, RDI, RBP, RSP,
    R8, R9, R10, R11,
    R12, R13, R14, R15,

    // Diğer mimariler için register'lar buraya eklenebilir.
    // ARM
    // R0, R1, ...
}

impl Type {
    // ... (mevcut kod)
}


// Matematiksel ve Mantıksal Operatörler
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum BinOp {
    Add, Sub, Mul, Div, 
	Mod,// <-- Modulo (%) için eklendi
    
    // Eşitlik Operatörleri
    Equal,      // <-- Eq (==) için eklendi
    NotEqual,   // <-- Ne (!=) için eklendi
    
    // Karşılaştırma Operatörleri
    Greater,    // <-- Gt (>) için eklendi
    Less,       // <-- Lt (<) için eklendi
    GreaterEqual, // <-- Ge (>=) için eklendi,
    LessEqual,  // <-- Le (<=) için eklendi
    Eq, Ne, Lt, Gt, Le, Ge, // Eski, kaldırılabilir
    Identical, // ===
    NotIdentical, // !==
    And, Or,
    BitwiseAnd, BitwiseOr, BitwiseXor,
    LShift, RShift,
}

// YENİ: UnOp enum'una derive özellikleri ekleniyor
#[derive(Debug, Clone)] 
pub enum UnOp {
    Neg, // Tekli eksi (örn: -a)
    Not, // Mantıksal DEĞİL (örn: !b)
    
    // YENİ: Artırma/Azaltma Operatörleri
    PreInc, // ++x
    PreDec, // --x
    PostInc, // x++
    PostDec, // x--
    BitwiseNot, // ~
    AddressOf,  // &
    Deref,      // *
}

// İfadeler (Değer dönen yapılar)
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Expr {
    Literal(LiteralValue),
    Variable(String),
    
    // YENİ: Tuple değer ifadesi (örn: (100, 250))
    Tuple(Vec<Expr>), 
    // YENİ: Dizi literali ifadesi (örn: [10, 20, 30])
    ArrayLiteral(Vec<Expr>),

	// YENİ: Match İfadesi
    Match {
        discriminant: Box<Expr>, // Kontrol edilen ifade (örn: match x { ... })
        cases: Vec<(Expr, Box<Expr>)>, // Durumlar: (Pattern, Sonuç İfadesi)
    },
	
	Block {
        statements: Vec<Stmt>, // Blok içindeki deyimler
    },
	DefaultCase,
    // YENİ: Dizi/Array erişimi (örn: arr[index])
    ArrayAccess { name: String, index: Box<Expr> }, // Burayı da ileride düzeltmeli: 'name' yerine 'object: Box<Expr>' olmalı

    // YENİ: Üye erişimi (örn: car1.owner.firstName)
    MemberAccess { object: Box<Expr>, member: String }, 

    // YENİ: Aralık ifadesi (örn: 0..10)
    Range { start: Box<Expr>, end: Box<Expr> }, 
    
    Binary { left: Box<Expr>, op: BinOp, right: Box<Expr> },
	Unary { op: UnOp, right: Box<Expr> },

    // Ternary conditional: cond ? then_expr : else_expr
    Conditional { cond: Box<Expr>, then_branch: Box<Expr>, else_branch: Box<Expr> },
    Await(Box<Expr>),
    // GÜNCELLEME: Atama ifadesinin sol tarafı artık herhangi bir ifade olabilir (örn: p.x)
    Assign { left: Box<Expr>, value: Box<Expr> },

    // ÇOK ÖNEMLİ GÜNCELLEME: 'callee' artık Box<Expr> alıyor (Zincirleme çağrılar için)
    Call { callee: Box<Expr>, args: Vec<(Option<String>, Expr)> },
    Lambda {
        params: Vec<(String, Type, Option<Expr>)>,
        return_type: Type,
        body: Box<Expr>,
    },
    InterpolatedString(Vec<Expr>), // Artık bir ifade listesi tutuyor: [Literal("..."), Variable("x"), Literal("...")]
    Try(Box<Expr>), // expr?
    // YENİ: Enum üyesine erişim (örn: Color::Red)
    EnumAccess {
        enum_name: String,
        variant_name: String,
    },
    // YENİ: Struct oluşturma ifadesi (örn: Point { x: 10, y: 20 })
    StructLiteral {
        name: String,
        fields: Vec<(String, Expr)>,
    },
    SizeOf(Type), // sizeof(type)
    // YENİ: Kanal işlemleri
    Send { channel: Box<Expr>, value: Box<Expr> }, // ch <- value
    Recv(Box<Expr>), // <-ch

}
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum LiteralValue {
    Int(i64),
    Float(f64),
    Hex(u64),
    Char(char),
    Str(String),
    Bool(bool),
    Null,
}

// İfadeler (İşlem yapan yapılar)
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Stmt {
    // GÜNCELLENDİ: 'is_const' ve 'is_mutable' alanları eklendi
    VarDecl { 
        name: String, 
        ty: Type, 
        init: Option<Expr>, 
        is_const: bool,      // const PI: f32 = ...
        is_let: bool,        // let x: i32 = ...
        is_mutable: bool,    // mut x: i32 = ...
        is_public: bool,     // pub const ...
    },
    // GÜNCELLEME: Atama ifadesinin sol tarafı artık herhangi bir ifade olabilir (örn: p.x)
    Assign { left: Expr, value: Expr },
    Block(Vec<Stmt>),
    If { cond: Expr, then_branch: Box<Stmt>, else_branch: Option<Box<Stmt>> },
    Return(Option<Expr>),
    Break,      // YENİ
    Continue,   // YENİ
    ExprStmt(Expr),
    // YENİ: While Döngüsü
    While { 
        condition: Expr, 
        body: Box<Stmt> 
    },
    // YENİ: Sonsuz Döngü (loop { ... })
    Loop {
        body: Box<Stmt>
    },
    
    // YENİ: For Döngüsü
    // Hem C-stili `for (init; cond; inc)` hem de `for (var in iter)` için kullanılır.
    For {
        // C-stili için: initializer, condition, increment
        initializer: Option<Box<Stmt>>, 
        condition: Option<Expr>,      
        increment: Option<Expr>,      

        // For-in için: variable, iterable
        variable: Option<String>,     // for (color in ...) -> "color"
        iterable: Option<Expr>,       // for (... in colors) -> Expr::Variable("colors")

        body: Box<Stmt>,
    },

    // YENİ: Echo (Çıktı Deyimi)
    Echo(Expr),
    // ... (Diğer Stmt varyantları aynı kalır)
	Empty,
    // NIMBLE'a özel: Group içindeki Tag (Etiket) yapıları
    Tag { name: String, body: Box<Stmt> },
    Rolling(String),
    // YENİ: Group içindeki etiketli ifade (label => expr)
    LabeledExpr { 
        label: String, 
        expr: Expr 
    },
    LabeledStmt {
        label: String,
        stmt: Box<Stmt>,
        is_public: bool, // Metotların pub olması için eklendi
    },
    Routine(Box<Expr>),
    Unsafe(Box<Stmt>),
    // YENİ: fastexec ve asm blokları
    FastExec(Box<Stmt>),
    Asm {
        tag: String,
        body: String,
    },
}
// Üst Düzey Tanımlamalar (Global scope)
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Decl {
    Module(String), // :network;
    Function {
        name: String,
        params: Vec<(String, Type, Option<Expr>)>,
        return_type: Type,
        body: Stmt, // Block
        is_inline: bool,
        is_async: bool,
        is_public: bool, // Fonksiyonların pub olması için eklendi
    },
    // YENİ: Dış (C) fonksiyon bildirimi
    ExternFn {
        name: String,
        params: Vec<(String, Type, Option<Expr>)>,
        return_type: Type,
        is_public: bool,
    },
    Group {
        name: String,
        is_export: bool,
        params: Vec<(String, Type, Option<Expr>)>, // group HTTP(param: type = default)
        return_type: Type,
        body: Vec<Decl>, // Group içindeki bildirimler (fonksiyon, const, vs.)
    },
	// Program dışındaki diğer Decl'leri kullanmıyorsanız silebilirsiniz
    Struct { 
        name: String, 
        fields: Vec<(String, Type)>,
        is_public: bool,
    },
    // YENİ: Enum Tanımı
    Enum {
        name: String,
        variants: Vec<(String, Option<Expr>)>, // Variant adı ve opsiyonel değeri
        is_public: bool,
    },
    // YENİ: Tip Takma Adı Tanımı (typedef)
    Typedef {
        name: String,
        target: Type,
        is_public: bool,
    },
    // YENİ: `use` bildiriminin neyi içeri aktardığını belirtir (Ayrı bir enum olarak).
    // Bu enum, Decl'in dışında tanımlanmalıdır.
    // UseSpec {
    //     All, // `use my_module;` (Tüm pub öğeleri)
    //     Specific(Vec<String>), // `use my_module::{item1, item2};`
    //     Wildcard, // `use my_module::*;`
    // },
    Use {
        path: Vec<String>, 
        spec: UseSpec, // Artık ayrı bir enum tipi
        is_export: bool, // `export use ...` için eklendi
    },
    // YENİ: Stil Tanımı (style Name = "ANSI_CODE")
    Style {
        name: String,
        code: String,
    },
	// YENİ: Programın tamamını temsil eden varyant
    Program(Vec<Decl>),
	StmtDecl(Box<Stmt>),
}

// UseSpec enum'unu Decl enum'unun dışına taşıyoruz
#[derive(Debug, Clone, PartialEq)]
pub enum UseSpecItem {
    Item(String), // item
    RenamedItem(String, String), // item as new_item
}

#[derive(Debug, Clone, PartialEq)]
pub enum UseSpec {
    All(Option<String>), // Sadece `use my_module as my;` için kullanılır.
    Specific(Vec<UseSpecItem>), // `use my_module::{item1, item2 as i2};`
    Wildcard, // `use my_module::*;`
}