#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    // --- Single Character Tokens ---
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
	Modulo,     // %
    Assign,     // =
    Semi,       // ;
    LBrace,     // {
    RBrace,     // }
    LParen,     // (
    RParen,     // )
    LBracket,   // [
    RBracket,   // ]
    Comma,      // ,
    Colon,      // :
    Dot,        // .
    Ampersand,  // &
    Pipe,       // |
    Caret,      // ^
    Tilde,      // ~
    Exclamation,// !
    Question,   // ?

    // --- Multi Character Tokens ---
    Eq,         // ==
    Identical,  // ===
    Ne,         // !=
    NotIdentical,// !==
    LessGreater, // <>
    Lt,         // <
    Gt,         // >
    Le,         // <=
    Ge,         // >=
    Inc,        // ++
    Dec,        // --
    
    // Logical
    LogAnd,     // && ve 'and' keyword
    LogOr,      // || ve 'or' keyword
    LogXor,     // 'xor' keyword
    // Assignment Ops
    PlusEq,     // +=
    MinusEq,    // -=
    StarEq,     // *=
    SlashEq,    // /=
    PercentEq,  // %=
    AndEq,      // &=
    OrEq,       // |=
    XorEq,      // ^=
    LShiftEq,   // <<=
    RShiftEq,   // >>=
    
    // Bitwise Shift
    LShift,     // <<
    RShift,     // >>
    
    // Special NIMBLE Tokens
    Arrow,      // ->
	FatArrow,	// =>
    Range,      // ..
    Ellipsis,   // ...
    RollingTag, // rolling:TAG yapısı için 'rolling' keywordü
    // YENİ: Kanal operatörleri
    //Send,       // <- (atama pozisyonunda)
    Recv,       // <- (ifade pozisyonunda)
    
    // --- Literals ---
    IntLit(i64),     
    FloatLit(f64),
    HexLit(i64),
    StrLit(String),  // "Normal String"
    InterpolatedStr(String), // "Value: {val}" gibi
    CharLit(char),
    Ident(String),  

    // --- Keywords (NIMBLE Specific Included) ---
    // Basic Types
    TypeI8, TypeI16, TypeI32, TypeI64, TypeI128,
    TypeU8, TypeU16, TypeU32, TypeU64, TypeU128,
    TypeF32, TypeF64, TypeF80, TypeF128,
    TypeD32, TypeD64, TypeD128,
    TypeBool, TypeChar, TypeVoid, TypeAny,
    TypeStr, TypeArr, TypePtr, TypeRef, TypeBit, TypeByte, TypeHex, TypeDec,

    // Control Flow
    If, Else, ElseIf, While, For, In, Loop, Return, Break, Continue, Match, Def,
    
    // Declarations & Modifiers
    Fn, Var, Const, Let, Struct, Enum, Group, Typedef,
    Pub, Export, Use, Extern, Inline, As,
    Self_, Super, // 'self' ve 'super' anahtar kelimeleri
    Echo, Print, Input, Strlen, Arrlen, Panic, Exit,
    Mut, Null, True, False,
    
    // Advanced Features
    Async, Await, Unsafe, Asm, FastExec, Routine, Style,
    
    // Memory
    Sizeof,

    // --- Preprocessor ---
    Preprocessor(String), // #ifdef, #define vb.

    // --- End of File ---
    Eof,
    Illegal(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenType,
    pub line: usize,
}

impl Token {
    pub fn new(kind: TokenType, line: usize) -> Self {
        Self { kind, line }
    }
}