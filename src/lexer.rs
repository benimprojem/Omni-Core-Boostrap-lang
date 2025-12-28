use crate::token::{Token, TokenType};

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
        }
    }

    fn peek(&self) -> char {
        if self.pos >= self.input.len() { '\0' } else { self.input[self.pos] }
    }
    
    fn peek_next(&self) -> char {
        if self.pos + 1 >= self.input.len() { '\0' } else { self.input[self.pos + 1] }
    }

    fn advance(&mut self) -> char {
        let c = self.peek();
        if c != '\0' {
            self.pos += 1;
            if c == '\n' { self.line += 1; }
        }
        c
    }

    fn skip_whitespace(&mut self) {
        loop {
            let c = self.peek();
            match c {
                ' ' | '\t' | '\r' | '\n' => { self.advance(); }
                '/' => {
                    if self.peek_next() == '/' {
                        // Tek satırlık yorum //
                        while self.peek() != '\n' && self.peek() != '\0' { self.advance(); }
                    } else if self.peek_next() == '*' {
                        // Çok satırlık yorum /* ... */
                        self.advance(); self.advance();
                        loop {
                            if self.peek() == '\0' { break; }
                            if self.peek() == '*' && self.peek_next() == '/' {
                                self.advance(); self.advance();
                                break;
                            }
                            self.advance();
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    fn scan_identifier(&mut self) -> Token {
        let mut text = String::new();
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            text.push(self.advance());
        }

        let kind = match text.as_str() {
            // Types
            "i8" => TokenType::TypeI8, "i16" => TokenType::TypeI16, "i32" => TokenType::TypeI32, "i64" => TokenType::TypeI64, "i128" => TokenType::TypeI128,
            "u8" => TokenType::TypeU8, "u16" => TokenType::TypeU16, "u32" => TokenType::TypeU32, "u64" => TokenType::TypeU64, "u128" => TokenType::TypeU128,
            "f32" => TokenType::TypeF32, "f64" => TokenType::TypeF64, "f80" => TokenType::TypeF80, "f128" => TokenType::TypeF128,
            "d32" => TokenType::TypeD32, "d64" => TokenType::TypeD64, "d128" => TokenType::TypeD128,
            "bool" => TokenType::TypeBool, "char" => TokenType::TypeChar, "void" => TokenType::TypeVoid, "any" => TokenType::TypeAny,
            "str" => TokenType::TypeStr, "arr" => TokenType::TypeArr, "ptr" => TokenType::TypePtr, "ref" => TokenType::TypeRef,
            "bit" => TokenType::TypeBit, "byte" => TokenType::TypeByte, "hex" => TokenType::TypeHex, "dec" => TokenType::TypeDec,
            "let" => TokenType::Let,
            // Keywords
            "fn" => TokenType::Fn, "var" => TokenType::Var, "const" => TokenType::Const,
            "if" => TokenType::If, "else" => TokenType::Else, "elseif" => TokenType::ElseIf, "in" => TokenType::In,
            "while" => TokenType::While, "for" => TokenType::For, "loop" => TokenType::Loop, "return" => TokenType::Return,
            "break" => TokenType::Break, "continue" => TokenType::Continue,
            "self" => TokenType::Self_, "super" => TokenType::Super,
            "match" => TokenType::Match, "def" => TokenType::Def,
            "struct" => TokenType::Struct, "enum" => TokenType::Enum, "group" => TokenType::Group, "typedef" => TokenType::Typedef,
            "as" => TokenType::As,
            "pub" => TokenType::Pub, "export" => TokenType::Export, "use" => TokenType::Use, "extern" => TokenType::Extern, "inline" => TokenType::Inline,
            "mut" => TokenType::Mut, "null" => TokenType::Null, "true" => TokenType::True, "false" => TokenType::False,
            "async" => TokenType::Async, "await" => TokenType::Await, "unsafe" => TokenType::Unsafe, "asm" => TokenType::Asm, "fastexec" => TokenType::FastExec,
            "routine" => TokenType::Routine,
            "sizeof" => TokenType::Sizeof, "rolling" => TokenType::RollingTag,
            "style" => TokenType::Style,
            // IO / Utility
            "echo"   => TokenType::Echo,
            "print"  => TokenType::Print,
            "input"  => TokenType::Input,
            "strlen" => TokenType::Strlen,
            "arrlen" => TokenType::Arrlen,
            "panic"  => TokenType::Panic,
            "exit"   => TokenType::Exit,
            
            // Logical Words
            "and" => TokenType::LogAnd, "or" => TokenType::LogOr, "xor" => TokenType::LogXor,
            
            _ => TokenType::Ident(text),
        };
        Token::new(kind, self.line)
    }

    fn scan_number(&mut self) -> Token {
        let mut text = String::new();
        let mut is_float = false;
        
        // Hexadecimal kontrolü
        if self.peek() == '0' && (self.peek_next() == 'x' || self.peek_next() == 'X') {
            self.advance(); // '0' atla
            self.advance(); // 'x' veya 'X' atla
            while self.peek().is_ascii_hexdigit() { // Hex rakamlarını oku
                text.push(self.advance());
            }
            if !text.is_empty() {
                let val = i64::from_str_radix(&text, 16).unwrap_or(0);
                return Token::new(TokenType::HexLit(val), self.line);
            } else {
                // Sadece "0x" varsa veya geçersiz hex karakteri varsa
                panic!("Geçersiz hexadecimal sayı: 0x{}", text);
            }
        }

        while self.peek().is_ascii_digit() {
            text.push(self.advance());
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            is_float = true;
            text.push(self.advance()); // .
            while self.peek().is_ascii_digit() {
                text.push(self.advance());
            }
        }
        
        // Bilimsel gösterim (10e3)
        if self.peek() == 'e' || self.peek() == 'E' {
            is_float = true;
            text.push(self.advance());
            if self.peek() == '+' || self.peek() == '-' {
                text.push(self.advance());
            }
            while self.peek().is_ascii_digit() {
                text.push(self.advance());
            }
        }

        if is_float {
            let val = text.parse::<f64>().unwrap_or(0.0);
            Token::new(TokenType::FloatLit(val), self.line)
        } else {
            let val = text.parse::<i64>().unwrap_or(0);
            Token::new(TokenType::IntLit(val), self.line)
        }
    }

    fn scan_string(&mut self) -> Token {
        self.advance(); // " atla
        let mut text = String::new();
        while self.peek() != '"' && self.peek() != '\0' {
            if self.peek() == '\\' {
                self.advance(); // '\' karakterini atla
                // Sonraki karaktere göre doğru kaçış dizisini ekle
                match self.advance() {
                    'n' => text.push('\n'),
                    'r' => text.push('\r'),
                    't' => text.push('\t'),
                    '\\' => text.push('\\'),
                    '"' => text.push('"'),
                    '\'' => text.push('\''),
                    '0' => text.push('\0'),
                    'x' => {
                        // Hexadecimal escape sequence: \xHH
                        let h1 = self.advance();
                        let h2 = self.advance();
                        if let Some(d1) = h1.to_digit(16) {
                            if let Some(d2) = h2.to_digit(16) {
                                let val = (d1 * 16 + d2) as u8;
                                text.push(val as char);
                            } else {
                                // Hata durumu: xH? -> x, H, ? (Basitçe geri ekle veya hata ver, şimdilik esnek olalım)
                                text.push('x'); text.push(h1); text.push(h2);
                            }
                        } else {
                             text.push('x'); text.push(h1); text.push(h2);
                        }
                    },
                    c => text.push(c),
                }
            } else {
                text.push(self.advance());
            }
        }
        
        if self.peek() == '"' { self.advance(); }
        
        if text.contains('{') && text.contains('}') {
             Token::new(TokenType::InterpolatedStr(text), self.line)
        } else {
             Token::new(TokenType::StrLit(text), self.line)
        }
    }

    // Char ('a') okuma fonksiyonu
    fn scan_char(&mut self) -> Token {
        self.advance(); // ' atla
        let c = self.advance(); // Karakteri al
        if self.peek() == '\'' { self.advance(); } // Kapanış ' atla
        Token::new(TokenType::CharLit(c), self.line)
    }

    fn scan_preprocessor(&mut self) -> Token {
        self.advance(); // # atla
        let mut text = String::new();
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            text.push(self.advance());
        }
        // Eğer sadece # varsa ve metin yoksa Hash döndürebiliriz ama
        // NIMBLE'da # genelde direktif. Şimdilik Preprocessor döndürüyoruz.
        Token::new(TokenType::Preprocessor(text), self.line)
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        let c = self.peek();

        if c == '\0' { return Token::new(TokenType::Eof, self.line); }

        if c.is_alphabetic() || c == '_' { return self.scan_identifier(); }
        if c.is_ascii_digit() { return self.scan_number(); }
        if c == '"' { return self.scan_string(); }
        if c == '\'' { return self.scan_char(); } // 'CharLit' burada çağrılıyor
        if c == '#' { return self.scan_preprocessor(); }

        self.advance();

        let kind = match c {
            '+' => if self.peek() == '+' { self.advance(); TokenType::Inc }
                   else if self.peek() == '=' { self.advance(); TokenType::PlusEq }
                   else { TokenType::Plus },
            '-' => if self.peek() == '-' { self.advance(); TokenType::Dec }
                   else if self.peek() == '>' { self.advance(); TokenType::Arrow } // 'Arrow' burada eklendi
                   else if self.peek() == '=' { self.advance(); TokenType::MinusEq }
                   else { TokenType::Minus },
            '*' => if self.peek() == '=' { self.advance(); TokenType::StarEq } else { TokenType::Star },
            '/' => if self.peek() == '=' { self.advance(); TokenType::SlashEq } else { TokenType::Slash },
            '%' => if self.peek() == '=' { self.advance(); TokenType::PercentEq } else { TokenType::Modulo },
            
            '=' => {
				//
				let line_for_token = self.line; 
				
				// 1. '=>' (FatArrow) Kontrolü (Yüksek Öncelik)
				if self.peek() == '>' {
					self.advance(); // '>' tüketildi
					return Token::new(TokenType::FatArrow, line_for_token); 
				} 
				
				// 2. '==' veya '===' Kontrolü
				if self.peek() == '=' {
					self.advance(); // İkinci '=' tüketildi
					
					if self.peek() == '=' {
						self.advance(); // Üçüncü '=' tüketildi
						return Token::new(TokenType::Identical, line_for_token); // ===
					} else {
						return Token::new(TokenType::Eq, line_for_token); // ==
					}
				} 
				
				// 3. Hiçbir kombinasyon eşleşmezse, tek '=' (Assign) döndürülür
				return Token::new(TokenType::Assign, line_for_token); // =
			},

            '!' => if self.peek() == '=' {
                        self.advance();
                        if self.peek() == '=' { self.advance(); TokenType::NotIdentical } 
                        else { TokenType::Ne } 
                   } else { TokenType::Exclamation },

                        '<' => if self.peek() == '=' { self.advance(); TokenType::Le }
                               else if self.peek() == '<' {
                                   self.advance();
                                   if self.peek() == '=' { self.advance(); TokenType::LShiftEq } else { TokenType::LShift }
                               } else if self.peek() == '>' { self.advance(); TokenType::LessGreater 
                               } else if self.peek() == '-' { self.advance(); TokenType::Recv } // YENİ: <- (receive)
                               else { TokenType::Lt },

            '>' => if self.peek() == '=' { self.advance(); TokenType::Ge }
                   else if self.peek() == '>' {
                       self.advance();
                       if self.peek() == '=' { self.advance(); TokenType::RShiftEq } else { TokenType::RShift }
                   } else { TokenType::Gt },
            
            '&' => if self.peek() == '&' { self.advance(); TokenType::LogAnd }
                   else if self.peek() == '=' { self.advance(); TokenType::AndEq }
                   else { TokenType::Ampersand },
            
            '|' => if self.peek() == '|' { self.advance(); TokenType::LogOr }
                   else if self.peek() == '=' { self.advance(); TokenType::OrEq }
                   else { TokenType::Pipe },
                   
            '^' => if self.peek() == '=' { self.advance(); TokenType::XorEq } else { TokenType::Caret },
            
            '.' => if self.peek() == '.' {
                        self.advance();
                        if self.peek() == '.' { self.advance(); TokenType::Ellipsis } 
                        else { TokenType::Range } 
                   } else { TokenType::Dot },

            ';' => TokenType::Semi,
            ':' => TokenType::Colon,
            ',' => TokenType::Comma,
            '{' => TokenType::LBrace, '}' => TokenType::RBrace,
            '(' => TokenType::LParen, ')' => TokenType::RParen,
            '[' => TokenType::LBracket, ']' => TokenType::RBracket,
            '~' => TokenType::Tilde,
            '?' => TokenType::Question,
            
            // Hash (#) artık scan_preprocessor tarafından işleniyor, 
            // buraya düşmesi beklenmez ama düşerse Illegal yerine Hash dönebiliriz.
            // Şimdilik Illegal bırakıyorum.
            
            _ => TokenType::Illegal(c.to_string()),
        };

        Token::new(kind, self.line)
    }
}