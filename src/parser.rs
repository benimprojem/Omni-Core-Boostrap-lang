// src/parser.rs 
use crate::token::{Token, TokenType};
use crate::ast::{Decl, Stmt, Expr, Type, BinOp, UnOp, LiteralValue}; 

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    scope_depth: u32,
    errors: Vec<String>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0, scope_depth: 0, errors: Vec::new() }
    }

    // --- Yardımcı Fonksiyonlar ---

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || matches!(self.peek_kind(), TokenType::Eof)
    }

    fn peek_kind(&self) -> TokenType {
        if self.current >= self.tokens.len() {
            let index = self.tokens.len().saturating_sub(1);
            return self.tokens[index].kind.clone();
        }
        self.tokens[self.current].kind.clone()
    }
    
    fn peek(&self) -> &Token {
        if self.is_at_end() {
            return &self.tokens[self.tokens.len() - 1]
        }
        &self.tokens[self.current]
    }

    fn check(&self, kind: &TokenType) -> bool {
        if self.is_at_end() { 
            return false; 
        }
        return self.tokens[self.current].kind == *kind; 
    }

    fn advance(&mut self) -> &Token {
        if self.current < self.tokens.len() {
            self.current += 1;
        }
        &self.tokens[self.current - 1] 
    }

    fn consume(&mut self, expected: TokenType, message: &str) -> &Token {
        if self.check(&expected) {
            return self.advance();
        }
        let current_token = &self.tokens[self.current];
        let error_msg = format!("Sözdizimi Hatası (Satır {}): {}, Beklenen: {:?}, Bulunan: {:?}", 
               current_token.line, message, expected, current_token.kind);
        self.errors.push(error_msg);
        current_token
    }
    
    fn check_next(&self, kind: &TokenType) -> bool {
        if self.current + 1 >= self.tokens.len() {
            return false;
        }
        self.tokens[self.current + 1].kind == *kind
    }

    // --- Ana Giriş Noktası ---

    pub fn parse(&mut self) -> (Decl, Vec<String>) { 
        let mut declarations = Vec::new();
        
        while !self.is_at_end() {
            let start_pos = self.current;
            if let Some(decl) = self.parse_declaration() {
                declarations.push(decl);
            } else {
                // parse_declaration, bir hata bulup senkronize olduğunda None döndürür.
                // Bu durumda, AST'ye bir şey eklemeden döngünün bir sonraki turuna geçmeliyiz.
                continue;
            }

            // SONSUZ DÖNGÜ KORUMASI:
            // Eğer parse_declaration çağrısından sonra hiç ilerleme kaydedilmediyse,
            // bu bir döngüye girdiğimiz anlamına gelir. Döngüyü manuel olarak kır.
            if self.current == start_pos {
                self.errors.push(format!("İç Hata (Satır {}): Ayrıştırıcı beklenmedik bir token üzerinde takılı kaldı: {:?}. İlerleme sağlanamıyor.", self.peek().line, self.peek_kind()));
                self.advance(); 
            }
        }

        (Decl::Program(declarations), self.errors.clone()) 
    }
    
    fn synchronize(&mut self) {
        while !self.is_at_end() {
            // Bir önceki token noktalı virgül ise, muhtemelen bir deyimin sonundayız.
            if self.tokens[self.current - 1].kind == TokenType::Semi {
                return;
            }
            // Bir sonraki token yeni bir deyim başlatıyor olabilir.
            match self.peek_kind() {
                TokenType::Fn | TokenType::Var | TokenType::Const | TokenType::For | TokenType::If | TokenType::While | TokenType::Return => return,
                _ => { self.advance(); }
            }
        }
    }

    #[allow(dead_code)]
    // --- Declarations (Tanımlamalar) ---

    fn parse_declaration(&mut self) -> Option<Decl> {
        // 'export' anahtar kelimesini kontrol et
        let is_export = if self.check(&TokenType::Export) {
            self.advance(); // 'export' token'ını tüket
            true
        } else {
            false
        };

        // YENİ: `pub` anahtar kelimesini de kontrol et.
        // Eğer `pub` varsa, onu tüket ve bir sonraki token'a bak.
        let is_public = if self.check(&TokenType::Pub) {
            self.advance();
            true
        } else {
            false
        };

        let result = if self.check(&TokenType::Fn) || self.check(&TokenType::Async) {
            Some(self.parse_function(is_export, is_public))
        } else if self.check(&TokenType::Var) || self.check(&TokenType::Const) || self.check(&TokenType::Let) {
            let var_stmt = self.parse_var_decl_logic(is_public);
            self.consume(TokenType::Semi, "';' bekleniyor");
            Some(Decl::StmtDecl(Box::new(var_stmt)))
        } else if self.check(&TokenType::Typedef) {
            Some(self.parse_typedef_decl(is_public))
        } else if self.check(&TokenType::Enum) {
            Some(self.parse_enum_decl(is_export, is_public))
        } else if self.check(&TokenType::Struct) {
            Some(self.parse_struct_decl(is_export, is_public))
        } else if self.check(&TokenType::Group) {
            Some(self.parse_group_decl(is_export, is_public))
        } else if self.check(&TokenType::Use) {
            Some(self.parse_use_decl(is_export, is_public))
        } else if self.check(&TokenType::Extern) {
            Some(self.parse_extern_decl(is_public, is_export))
        } else if self.check(&TokenType::Style) {
            Some(self.parse_style_decl())
        } else if self.check(&TokenType::RBrace) {
            // Global alanda beklenmedik bir '}' varsa, hata ver ve tüket.
            self.errors.push(format!("Sözdizimi Hatası (Satır {}): Global alanda beklenmedik '}}'.", self.peek().line));
            self.advance();
            None // Hata sonrası AST'ye bir şey ekleme.
        } else {
            // Eğer global kapsamda değilsek (bir fonksiyon içindeysek), bu durum
            // bir önceki `parse_statement` çağrısının bir hata bulup senkronize
            // etmesinden kaynaklanıyor olabilir. Bu yüzden burada tekrar hata üretme.
            if self.scope_depth == 0 {
                let bad_token = self.peek();
                let error_msg = format!("Sözdizimi Hatası (Satır {}): '{{' eksik olabilir. Bulunan: {:?}", bad_token.line, bad_token.kind);
                self.errors.push(error_msg);

                // Hata sonrası kurtarma: Bir sonraki üst seviye bildirime kadar atla.
                while !self.is_at_end() {
                    match self.peek_kind() {
                        TokenType::Fn | TokenType::Var | TokenType::Const | TokenType::Group | 
                        TokenType::Export | TokenType::Struct | TokenType::Enum | TokenType::Use | TokenType::Extern => {
                            break; // Güvenli bir başlangıç noktası bulduk, döngüyü kır.
                        },
                        _ => { self.advance(); } // Diğer her şeyi atla.
                    }
                }
            }
            None // Hata sonrası AST'ye bir şey ekleme.
        };
        result
    }

    fn parse_style_decl(&mut self) -> Decl {
        self.consume(TokenType::Style, "'style' bekleniyor");
        let name_token = self.advance().clone();
        
        let name = match name_token.kind {
            TokenType::Ident(n) => n,
            _ => {
                self.errors.push(format!("Sözdizimi Hatası (Satır {}): Stil adı bekleniyor.", name_token.line));
                "unknown".to_string()
            }
        };

        self.consume(TokenType::Assign, "'=' bekleniyor");
        
        // Değer bir string olmalı
        let value_token = self.advance().clone();
        let code = match value_token.kind {
            TokenType::StrLit(s) => s,
            _ => {
                self.errors.push(format!("Sözdizimi Hatası (Satır {}): Stil tanımı için string bekleniyor.", value_token.line));
                "".to_string()
            }
        };
        
        self.consume(TokenType::Semi, "';' bekleniyor");
        
        Decl::Style { name, code }
    }

    fn parse_function(&mut self, is_export: bool, is_public_decl: bool) -> Decl {
        // `pub` veya `export` varsa, fonksiyon public'tir.
        let is_public = is_public_decl || is_export;

        let is_inline = if self.check(&TokenType::Inline) {
            self.advance();
            true
        } else {
            false
        };

        let is_async = if self.check(&TokenType::Async) {
            self.advance();
            true
        } else {
            false
        };

        self.consume(TokenType::Fn, "'fn' bekleniyor");
        
        let name = match self.advance().kind.clone() {
            TokenType::Ident(n) => n,
            _ => {
                self.errors.push(format!("Sözdizimi Hatası (Satır {}): Fonksiyon adı bekleniyor.", self.peek().line));
                // Hata durumunda senkronize et ve boş bir bildirim döndür.
                self.synchronize();
                return Decl::StmtDecl(Box::new(Stmt::Empty)); // Bu özel durum kalabilir.
            }
        };

        let mut return_type = Type::Void; 

        if self.check(&TokenType::Colon) {
            self.advance(); 
            return_type = self.parse_type();
        }

        self.consume(TokenType::LParen, "'(' bekleniyor");
        let params = self.parse_function_params();
        self.consume(TokenType::RParen, "')' bekleniyor");
        
        if self.check(&TokenType::Colon) {
            self.advance();
            return_type = self.parse_type(); 
        }
        
        let body = self.parse_block(); 
        Decl::Function { name, params, return_type, body, is_inline, is_async, is_public }
    }

    fn parse_function_params(&mut self) -> Vec<(String, Type, Option<Expr>)> {
        let mut params = Vec::new();
        while !self.check(&TokenType::RParen) && !self.is_at_end() {
            // YENİ: Parametre adı olarak 'self' anahtar kelimesini de kabul et.
            let pname = if let TokenType::Ident(name) = self.peek_kind() {
                name
            } else if self.check(&TokenType::Self_) {
                "self".to_string()
            } else {
                break; // Tanımlayıcı veya 'self' değilse, parametre listesi bitmiştir.
            };
                self.advance();
                self.consume(TokenType::Colon, "':' bekleniyor");
                let ptype = self.parse_type();

                let mut default_value = None;
                if self.check(&TokenType::Assign) {
                    self.advance(); 
                    default_value = Some(self.parse_expression());
                }

                params.push((pname, ptype, default_value));

                if self.check(&TokenType::Comma) { self.advance(); }
        }
        params
    }

    fn parse_group_decl(&mut self, is_export: bool, _is_public_decl: bool) -> Decl {
        self.consume(TokenType::Group, "'group' bekleniyor");

        let name = match self.advance().kind.clone() {
            TokenType::Ident(n) => n,
            _ => {
                self.errors.push(format!("Sözdizimi Hatası (Satır {}): Grup adı bekleniyor.", self.peek().line));
                self.synchronize();
                return Decl::StmtDecl(Box::new(Stmt::Empty)); // Bu özel durum kalabilir.
            }
        };

        let mut params = Vec::new();
        let mut return_type = Type::Void;

        // YENİ: Grup tanımının parametre alıp almadığını kontrol et.
        // `group Point { ... }` (metot bloğu) vs `group MyModule(...) { ... }` (modül)
        if self.check(&TokenType::LParen) {
            // Parametre alan normal grup
            self.consume(TokenType::LParen, "'(' bekleniyor");
            params = self.parse_function_params();
            self.consume(TokenType::RParen, "')' bekleniyor");

            if self.check(&TokenType::Colon) {
                self.advance();
                return_type = self.parse_type();
            }
        }

        // YENİ: Grup gövdesini, üst düzey bildirimleri (fonksiyon, const, extern vb.)
        // içerebilecek şekilde ayrıştır.
        self.consume(TokenType::LBrace, "Grup gövdesi için '{' bekleniyor.");
        let mut body_decls = Vec::new();
        while !self.check(&TokenType::RBrace) && !self.is_at_end() {
            if let Some(decl) = self.parse_declaration() {
                body_decls.push(decl);
            } else {
                // Hata durumunda ilerleme sağlamak için
                self.advance();
            }
        }
        self.consume(TokenType::RBrace, "Grup gövdesini kapatmak için '}' bekleniyor.");

        Decl::Group {
            name, is_export, params, return_type, body: body_decls,
        }
    }

    fn parse_struct_decl(&mut self, is_export: bool, is_public_decl: bool) -> Decl {
        let is_public = is_public_decl || is_export;

        self.consume(TokenType::Struct, "'struct' bekleniyor.");
        let name = match self.peek_kind() {
            TokenType::Ident(n) => {
                self.advance();
                n
            }
            _ => {
                self.errors.push(format!("Sözdizimi Hatası (Satır {}): Struct adı bekleniyor.", self.peek().line));
                self.synchronize();
                return Decl::StmtDecl(Box::new(Stmt::Empty));
            }
        };

        self.consume(TokenType::LBrace, "Struct gövdesi için '{' bekleniyor.");

        let mut fields = Vec::new();

        while !self.check(&TokenType::RBrace) && !self.is_at_end() {
            if let TokenType::Ident(field_name) = self.peek_kind() {
                // Alan ayrıştırılıyor
                self.advance();
                self.consume(TokenType::Colon, "Struct alanından sonra ':' bekleniyor.");
                let field_type = self.parse_type();
                fields.push((field_name, field_type));
                self.consume(TokenType::Semi, "Struct alanı tanımından sonra ';' bekleniyor.");
            } else {
                self.errors.push(format!("Sözdizimi Hatası (Satır {}): Struct içinde alan adı bekleniyor.", self.peek().line));
                self.synchronize();
                break;
            }
        }
        self.consume(TokenType::RBrace, "Struct gövdesini kapatmak için '}' bekleniyor.");
        Decl::Struct { name, fields, is_public }
    }

    fn parse_typedef_decl(&mut self, is_public_decl: bool) -> Decl {
        let is_public = is_public_decl;

        self.consume(TokenType::Typedef, "'typedef' bekleniyor.");
        let alias_name = match self.peek_kind() {
            TokenType::Ident(n) => {
                self.advance();
                n
            }
            _ => {
                self.errors.push(format!("Sözdizimi Hatası (Satır {}): Tip takma adı bekleniyor.", self.peek().line));
                self.synchronize();
                return Decl::StmtDecl(Box::new(Stmt::Empty));
            }
        };

        // YENİ: Dizi takma adı için '[]' kontrolü. Örn: typedef MyIntArray[]: i32;
        let is_array_alias = if self.check(&TokenType::LBracket) {
            self.advance();
            self.consume(TokenType::RBracket, "Dizi takma adı için ']' bekleniyor.");
            true
        } else {
            false
        };

        self.consume(TokenType::Colon, "Tip takma adından sonra ':' bekleniyor.");

        let base_type = self.parse_type();

        // Eğer '[]' varsa, hedef tipi bir dizi tipine sar.
        let final_target_type = if is_array_alias {
            Type::Array(Box::new(base_type), None) // Dinamik dizi olarak tanımla
        } else {
            base_type
        };

        self.consume(TokenType::Semi, "';' bekleniyor.");

        Decl::Typedef { name: alias_name, target: final_target_type, is_public }
    }

    fn parse_enum_decl(&mut self, is_export: bool, is_public_decl: bool) -> Decl {
        let is_public = is_public_decl || is_export;

        self.consume(TokenType::Enum, "'enum' bekleniyor.");
        let name = match self.peek_kind() {
            TokenType::Ident(n) => {
                self.advance();
                n
            }
            _ => {
                self.errors.push(format!("Sözdizimi Hatası (Satır {}): Enum adı bekleniyor.", self.peek().line));
                self.synchronize();
                return Decl::StmtDecl(Box::new(Stmt::Empty));
            }
        };

        self.consume(TokenType::LBrace, "Enum gövdesi için '{' bekleniyor.");

        let mut variants = Vec::new();
        while !self.check(&TokenType::RBrace) && !self.is_at_end() {
            let variant_name = match self.peek_kind() {
                TokenType::Ident(n) => {
                    self.advance();
                    n
                }
                _ => {
                    self.errors.push(format!("Sözdizimi Hatası (Satır {}): Enum üyesi adı bekleniyor.", self.peek().line));
                    break;
                }
            };

            let mut value = None;
            if self.check(&TokenType::Assign) {
                self.advance();
                value = Some(self.parse_expression());
            }
            variants.push((variant_name, value));

            if self.check(&TokenType::Comma) { self.advance(); }
        }
        self.consume(TokenType::RBrace, "Enum gövdesini kapatmak için '}' bekleniyor.");
        Decl::Enum { name, variants, is_public }
    }

    fn parse_use_decl(&mut self, is_export: bool, _is_public_decl: bool) -> Decl {
        self.consume(TokenType::Use, "'use' bekleniyor.");

        let mut path = Vec::new();
        let mut spec = crate::ast::UseSpec::Wildcard; // Varsayılan: `use module;` -> `use module::*;` gibi davranır.

        loop {
            // YENİ: 'self' ve 'super' anahtar kelimelerini de yol parçası olarak kabul et.
            if let TokenType::Ident(part) = self.peek_kind() { // Normal tanımlayıcı
                self.advance();
                path.push(part);
            } else if self.check(&TokenType::Self_) { // 'self' anahtar kelimesi
                self.advance();
                path.push("self".to_string());
            } else if self.check(&TokenType::Super) { // 'super' anahtar kelimesi
                self.advance();
                path.push("super".to_string());
            } else {
                // Eğer yolun başında bir tanımlayıcı, 'self' veya 'super' yoksa, hata ver.
                if path.is_empty() {
                    self.errors.push(format!("Sözdizimi Hatası (Satır {}): 'use' bildiriminde yol bölümü (identifier, 'self' veya 'super') bekleniyor.", self.peek().line));
                    self.synchronize();
                    return Decl::StmtDecl(Box::new(Stmt::Empty));
                }
                break; // Yolun sonuna geldik.
            }

            if self.check(&TokenType::Colon) && self.check_next(&TokenType::Colon) {
                self.advance(); // ::
                self.advance();

                // YENİ: `*` veya `{` kontrolü
                if self.check(&TokenType::Star) {
                    // `use my_module::*;` durumu
                    self.advance(); // '*'
                    spec = crate::ast::UseSpec::Wildcard;
                    break; // Yolun sonuna geldik.
                } else if self.check(&TokenType::LBrace) {
                    // `use my_module::{...}` durumu
                    self.advance(); // {
                    let mut item_list = Vec::new();
                    while !self.check(&TokenType::RBrace) && !self.is_at_end() {
                        if let TokenType::Ident(item_name) = self.peek_kind() {
                            self.advance(); // öğe adını tüket
                            if self.check(&TokenType::As) {
                                self.advance(); // 'as' tüket
                                if let TokenType::Ident(alias) = self.peek_kind() {
                                    self.advance(); // takma adı tüket
                                    item_list.push(crate::ast::UseSpecItem::RenamedItem(item_name, alias));
                                } else {
                                    self.errors.push(format!("Sözdizimi Hatası (Satır {}): 'as' anahtar kelimesinden sonra bir takma ad bekleniyor.", self.peek().line));
                                }
                            } else {
                                item_list.push(crate::ast::UseSpecItem::Item(item_name));
                            }
                        } else {
                            self.errors.push(format!("Sözdizimi Hatası (Satır {}): 'use' bloğu içinde öğe adı bekleniyor.", self.peek().line));
                            break;
                        }
                        if self.check(&TokenType::Comma) {
                            self.advance();
                        }
                    }
                    self.consume(TokenType::RBrace, "'use' bloğunu kapatmak için '}' bekleniyor.");
                    spec = crate::ast::UseSpec::Specific(item_list);
                    break; // Yolun sonuna geldik.
                }
            } else if self.check(&TokenType::Slash) {
                // YENİ: İç içe modüller için yol ayırıcı.
                // `my/utils` gibi bir yolda devam etmek için `/`'ı tüket.
                self.advance();
                continue;
            } else {
                break;
            }
        }

        // `use my_module as my;` durumu için 'as' kontrolü
        if self.check(&TokenType::As) {
            self.advance(); // 'as' tüket
            if let TokenType::Ident(alias) = self.peek_kind() { // `spec`'in `Wildcard` olduğunu varsayarak
                self.advance(); // takma adı tüket
                spec = crate::ast::UseSpec::All(Some(alias));
            } else {
                self.errors.push(format!("Sözdizimi Hatası (Satır {}): 'as' anahtar kelimesinden sonra bir modül takma adı bekleniyor.", self.peek().line));
            }
        }

        self.consume(TokenType::Semi, "'use' bildiriminden sonra ';' bekleniyor.");

        Decl::Use { path, spec, is_export }
    }

    // YENİ: Sadece `extern fn` için, parametre isimlerini zorunlu kılmayan ayrıştırıcı.
    fn parse_extern_function_params(&mut self) -> Vec<(String, Type, Option<Expr>)> {
        let mut params = Vec::new();
        while !self.check(&TokenType::RParen) && !self.is_at_end() {
            let mut pname = String::new(); // Parametre adı varsayılan olarak boş.

            // Eğer bir `isim: tip` yapısı varsa, ismi al.
            if let TokenType::Ident(name) = self.peek_kind() {
                if self.check_next(&TokenType::Colon) {
                    pname = name;
                    self.advance(); // ismi tüket
                    self.advance(); // ':' tüket
                }
            }

            let ptype = self.parse_type();
            params.push((pname, ptype, None)); // Dış fonksiyonlarda varsayılan değer olmaz.

            if self.check(&TokenType::Comma) { self.advance(); }
        }
        params
    }
    // YENİ: `extern` bildirimlerini ayrıştırır.
    fn parse_extern_decl(&mut self, is_public_decl: bool, is_export: bool) -> Decl {
        self.consume(TokenType::Extern, "'extern' bekleniyor.");

        let is_public = is_public_decl || is_export;

        if self.check(&TokenType::Fn) {
            self.consume(TokenType::Fn, "'fn' bekleniyor.");

            let name = match self.advance().kind.clone() {
                TokenType::Ident(n) => n,
                _ => {
                    self.errors.push(format!("Sözdizimi Hatası (Satır {}): Dış fonksiyon adı bekleniyor.", self.peek().line));
                    self.synchronize();
                    return Decl::StmtDecl(Box::new(Stmt::Empty));
                }
            };

            self.consume(TokenType::LParen, "'(' bekleniyor");
            let params = self.parse_extern_function_params(); // YENİ: Özel extern param ayrıştırıcısını kullan.
            self.consume(TokenType::RParen, "')' bekleniyor");

            let mut return_type = Type::Void;
            if self.check(&TokenType::Colon) {
                self.advance();
                return_type = self.parse_type();
            }

            self.consume(TokenType::Semi, "Dış fonksiyon bildiriminden sonra ';' bekleniyor.");

            Decl::ExternFn { name, params, return_type, is_public }
        } else {
            self.errors.push(format!("Sözdizimi Hatası (Satır {}): 'extern' sonrası sadece 'fn' desteklenmektedir.", self.peek().line));
            self.synchronize();
            Decl::StmtDecl(Box::new(Stmt::Empty))
        }
    }

    // --- Statements (Deyimler) ---

    fn parse_statement(&mut self) -> Stmt {
        match self.peek_kind() {
            TokenType::Var | TokenType::Const | TokenType::Mut => self.parse_var_decl(), 
            TokenType::If => self.parse_if_stmt(),
            TokenType::Match => self.parse_match_stmt(), 
            TokenType::While => self.parse_while_stmt(),
            TokenType::Loop => self.parse_loop_stmt(),
            TokenType::For => self.parse_for_stmt(),
            // YENİ: Yerleşik fonksiyonlar için özel ayrıştırma kuralları
            TokenType::Print | 
            TokenType::Input | 
            TokenType::Strlen | 
            TokenType::Arrlen | 
            TokenType::Panic | 
            TokenType::Exit => self.parse_builtin_call_stmt(),
            TokenType::Echo => self.parse_echo_stmt(),
            TokenType::Break => {
                self.advance();
                self.consume(TokenType::Semi, "'break' deyiminden sonra ';' bekleniyor.");
                Stmt::Break
            },
            TokenType::RollingTag => {
                self.advance();
                self.consume(TokenType::Colon, "':' bekleniyor");
                let tag = if let TokenType::Ident(n) = self.peek_kind() {
                    self.advance();
                    n
                } else {
                    self.errors.push(format!("Sözdizimi Hatası (Satır {}): Etiket adı bekleniyor.", self.peek().line));
                    "__invalid_tag__".to_string()
                };
                self.consume(TokenType::Semi, "';' bekleniyor");
                Stmt::Rolling(tag)
            },
            TokenType::Routine => {
                self.advance();
                let call = self.parse_call();
                self.consume(TokenType::Semi, "';' bekleniyor");
                Stmt::Routine(Box::new(call))
            },
            TokenType::Unsafe => {
                self.advance();
                let block = self.parse_block();
                Stmt::Unsafe(Box::new(block))
            },
            TokenType::FastExec => {
                self.advance();
                let block = self.parse_block();
                Stmt::FastExec(Box::new(block))
            },
            TokenType::Asm => {
                self.advance();
                self.consume(TokenType::Colon, "'asm' sonrası ':' bekleniyor.");
                let tag = match self.peek_kind() {
                    TokenType::Ident(n) => { self.advance(); n },
                    _ => {
                        self.errors.push(format!("Sözdizimi Hatası (Satır {}): 'asm:' sonrası bir etiket (TAG) bekleniyor.", self.peek().line));
                        "__invalid_asm_tag__".to_string()
                    }
                };
                self.consume(TokenType::LBrace, "'asm' bloğu için '{' bekleniyor.");
                // `asm` bloğunun içeriğini ham bir string olarak alacağız.
                // Şimdilik basitçe bir sonraki '}' karakterine kadar olan her şeyi alıyoruz.
                // Bu, daha sonra daha akıllı bir ayrıştırma gerektirebilir.
                let body = "/* asm body placeholder */".to_string(); // TODO: Gerçek içeriği ayrıştır.
                self.consume(TokenType::RBrace, "'asm' bloğunu kapatmak için '}' bekleniyor.");
                Stmt::Asm { tag, body }
            },
            TokenType::Continue => {
                self.advance();
                self.consume(TokenType::Semi, "'continue' deyiminden sonra ';' bekleniyor.");
                Stmt::Continue
            },

            TokenType::Return => self.parse_return(),   
            TokenType::LBrace => self.parse_block(), 
            
            TokenType::Semi => {
                 self.advance();
                Stmt::Empty 
            },

            // YENİ: `pub method => ...` veya `method => ...` yapılarını işle
            _ if {
                let is_pub = self.check(&TokenType::Pub);
                let ident_pos = if is_pub { 1 } else { 0 };
                self.tokens.get(self.current + ident_pos).map_or(false, |t| matches!(t.kind, TokenType::Ident(_))) &&
                self.tokens.get(self.current + ident_pos + 1).map_or(false, |t| t.kind == TokenType::FatArrow)
            } => {
                 let is_public = if self.check(&TokenType::Pub) {
                     self.advance();
                     true
                 } else {
                     false
                 };

                 let label = match self.advance().kind.clone() {
                     TokenType::Ident(n) => n,
                     _ => unreachable!(), // Yukarıdaki kontrol bunu garantiler.
                 };
                 self.consume(TokenType::FatArrow, "=> bekleniyor");

                 // '=>' sonrası hem bir ifade (örn: lambda) hem de tam bir deyim (örn: blok) gelebilir.
                 let next_stmt = if self.check(&TokenType::LBrace) {
                     self.parse_statement()
                 } else {
                     Stmt::ExprStmt(self.parse_expression())
                 };

                 // `is_public` alanını AST'ye ekle.
                 // NOT: Bu, `Stmt::LabeledStmt`'in `is_public: bool` alanına sahip olmasını gerektirir.
                 Stmt::LabeledStmt { label, stmt: Box::new(next_stmt), is_public: is_public }
            },
            // Yukarıdaki özel durumlar (if, while, var vb.) dışındaki her şey
            // bir ifade deyimi olarak kabul edilir. Bu, atamaları, fonksiyon çağrılarını,
            // artırma/azaltma işlemlerini vb. kapsar.
            _ => {
                let expr = self.parse_assignment_expression();
                self.consume(TokenType::Semi, "İfade deyiminden sonra ';' bekleniyor");
                Stmt::ExprStmt(expr)
            },
        }
    }

    fn parse_block(&mut self) -> Stmt {
        self.scope_depth += 1; // Kapsam derinliğini artır
        self.consume(TokenType::LBrace, "'{' bekleniyor");
        let mut stmts = Vec::new();

        while !self.check(&TokenType::RBrace) && !self.is_at_end() {
            stmts.push(self.parse_statement());
        }
        
        self.consume(TokenType::RBrace, "'}' bekleniyor");
        self.scope_depth -= 1; // Kapsam derinliğini azalt
        Stmt::Block(stmts)
    }

    fn parse_var_decl_logic(&mut self, is_public_decl: bool) -> Stmt {
        let mut is_mutable = false; 
        let mut is_let = false;
        // `pub` anahtar kelimesi sadece global kapsamdaki `const` için anlamlıdır.
        let is_public = is_public_decl;

        if self.check(&TokenType::Mut) {
            self.advance();
            is_mutable = true;
        }

        let is_const = self.check(&TokenType::Const); 
        if is_const {
            self.advance();
            // 'const' ile 'mut' birlikte kullanılamaz.
            if is_mutable {
                self.errors.push(format!("Sözdizimi Hatası (Satır {}): Bir değişken hem 'mut' hem de 'const' olamaz.", self.peek().line));
            }
            is_mutable = false; 
        } else if self.check(&TokenType::Let) {
            self.advance();
            is_let = true;
            // 'let' ile 'mut' kullanılmadıysa, varsayılan olarak değiştirilemez.
            // 'is_mutable' zaten false, bu yüzden ek bir şey yapmaya gerek yok.
        } else if self.check(&TokenType::Var) { 
            self.advance();
            is_mutable = true; // 'var' her zaman değiştirilebilirdir.
        }

        let name = match self.advance().kind.clone() {
            TokenType::Ident(n) => n,
            _ => {
                self.errors.push(format!("Sözdizimi Hatası (Satır {}): Değişken adı bekleniyor.", self.peek().line));
                self.synchronize();
                return Stmt::Empty;
            }
        };

        let mut is_array = false;
        let mut array_size = None;
        if self.check(&TokenType::LBracket) {
            is_array = true;
            self.advance();
            // Eğer `[]` ise dinamik dizi, `[5]` ise sabit boyutlu dizidir.
            if !self.check(&TokenType::RBracket) {
                let size_expr = self.parse_expression();
                if let Expr::Literal(LiteralValue::Int(n)) = size_expr {
                    array_size = Some(n as usize);
                } else {
                    self.errors.push(format!("Sözdizimi Hatası (Satır {}): Dizi boyutu bir tamsayı literali olmalıdır.", self.peek().line));
                }
            }
            self.consume(TokenType::RBracket, "Dizi tanımı için ']' bekleniyor.");
        }
        
        let var_type = if self.check(&TokenType::Colon) {
            self.advance();
            let parsed_type = self.parse_type();
            if is_array {
                Type::Array(Box::new(parsed_type), array_size)
            } else {
                parsed_type
            }
        } else {
            // Tip belirtilmemişse Any (Çıkarılacak tip) ata.
            if is_array {
                Type::Array(Box::new(Type::Any), array_size)
            } else {
                Type::Any
            }
        };

        let mut init = None;
        if self.check(&TokenType::Assign) {
            self.advance();
            // Parantezli bir ifade (özellikle ternary) ile başlayan bir atama varsa,
            // bunu `parse_call` yerine doğrudan `parse_expression` ile ayrıştırmalıyız.
            // Bu, `var x = (a > b) ? 1 : 0;` gibi ifadelerin doğru ayrıştırılmasını sağlar.
            init = Some(self.parse_expression()); // Önceden `parse_precedence(0)` idi, `parse_expression` daha genel.

        }        
        Stmt::VarDecl { 
            name, 
            ty: var_type, 
            init, 
            is_const, 
            is_let,
            is_mutable,
            is_public,
        }
    }

    fn parse_var_decl(&mut self) -> Stmt {
        let stmt = self.parse_var_decl_logic(false); // Deyimler içinde `pub` beklenmez.
        self.consume(TokenType::Semi, "';' bekleniyor");
        stmt
    }

    fn parse_match_stmt(&mut self) -> Stmt {
        let expr = self.parse_match_expr();
        Stmt::ExprStmt(expr)
    }
    
    fn parse_match_expr(&mut self) -> Expr {
        self.consume(TokenType::Match, "Match ifadesi 'match' ile başlamalı.");

        let discriminant = Box::new(self.parse_expression());

        self.consume(TokenType::LBrace, "Match ifadesinden sonra '{' bekleniyor.");

        let mut cases = Vec::new();

        while !self.check(&TokenType::RBrace) && !self.is_at_end() {
            let pattern = self.parse_primary(); 
            
            self.consume(TokenType::FatArrow, "Match durumundan sonra '=>' bekleniyor.");
            
            let result: Box<Expr>;
            if self.check(&TokenType::LBrace) {
                result = Box::new(self.parse_block_expr()); 
            } else {
                result = Box::new(self.parse_expression()); 
            }
            
            cases.push((pattern, result));
            if self.check(&TokenType::Comma) {
                self.advance();
            } else if !self.check(&TokenType::RBrace) {
                self.errors.push(format!("Sözdizimi Hatası (Satır {}): Match kollarını ayırmak için ',' veya '}}' bekleniyor. Bulunan token: {:?}", self.peek().line, self.peek_kind()));
                self.synchronize();
                break;
            }
        }

        self.consume(TokenType::RBrace, "Match ifadesi '}' ile bitmeli.");

        Expr::Match { discriminant, cases }
    }

    fn parse_while_stmt(&mut self) -> Stmt {
        self.advance(); 

        self.consume(TokenType::LParen, "'while' döngüsü için '(' bekleniyor");
        let condition = self.parse_expression(); 
        self.consume(TokenType::RParen, "'while' koşulundan sonra ')' bekleniyor");

        let body = Box::new(self.parse_block()); 

        Stmt::While { condition, body }
    }

    fn parse_loop_stmt(&mut self) -> Stmt {
        self.advance(); 
        let body = Box::new(self.parse_block());
        Stmt::Loop { body }
    }
    
    fn parse_for_stmt(&mut self) -> Stmt {
        self.advance(); // 'for' tüket
        
        // Parantezli mi değil mi kontrol et
        let has_lparen = if self.check(&TokenType::LParen) {
            self.advance();
            true
        } else {
            false
        };

        let mut has_in_keyword = false;
        let mut temp_pos = self.current;
        let terminator = if has_lparen { TokenType::RParen } else { TokenType::LBrace };
        
        while temp_pos < self.tokens.len() && self.tokens[temp_pos].kind != terminator {
            if self.tokens[temp_pos].kind == TokenType::In {
                has_in_keyword = true;
                break;
            }
            temp_pos += 1;
        }

        if has_in_keyword {
            let variable = match self.peek_kind() {
                TokenType::Ident(name) => {
                    self.advance(); 
                    name
                },
                _ => {
                    self.errors.push(format!("Sözdizimi Hatası (Satır {}): for-in döngüsü için bir değişken adı bekleniyor.", self.peek().line));
                    self.synchronize();
                    return Stmt::Empty;
                }
            };
            self.consume(TokenType::In, "for-in döngüsü için 'in' anahtar kelimesi bekleniyor.");
            let iterable = self.parse_expression();
            
            if has_lparen {
                self.consume(TokenType::RParen, "for döngüsü başlığından sonra ')' bekleniyor.");
            }
            let body = Box::new(self.parse_block());

            Stmt::For {
                initializer: None, condition: None, increment: None, 
                variable: Some(variable),
                iterable: Some(iterable),
                body,
            }
        } else {
            let initializer = if !self.check(&TokenType::Comma) {
                if self.check(&TokenType::Var) || self.check(&TokenType::Const) {
                    let var_stmt = self.parse_var_decl_logic(false);
                    self.consume(TokenType::Comma, "for döngüsü başlatıcısından sonra ',' bekleniyor.");
                    Some(Box::new(var_stmt))
                } else {
                    let expr = self.parse_expression();
                    self.consume(TokenType::Comma, "for döngüsü başlatıcısından sonra ',' bekleniyor.");
                    Some(Box::new(Stmt::ExprStmt(expr)))
                }
            } else {
                self.consume(TokenType::Comma, "for döngüsü başlatıcısından sonra ',' bekleniyor.");
                None
            };

            let condition = if !self.check(&TokenType::Comma) { Some(self.parse_expression()) } else { None };
            self.consume(TokenType::Comma, "for döngüsü koşulundan sonra ',' bekleniyor.");

            let increment = if !self.check(&TokenType::RParen) && !self.check(&TokenType::LBrace) { 
                Some(self.parse_expression()) 
            } else { 
                None 
            };

            if has_lparen {
                self.consume(TokenType::RParen, "for döngüsü başlığından sonra ')' bekleniyor.");
            }

            let body = Box::new(self.parse_block());

            Stmt::For {
                initializer, condition, increment,
                variable: None, iterable: None, 
                body,
            }
        }
    }

    fn parse_echo_stmt(&mut self) -> Stmt {
        self.advance(); // 'echo' tüket
        self.consume(TokenType::LParen, "'echo' için '(' bekleniyor");
        let expr = self.parse_expression();
        self.consume(TokenType::RParen, "'echo' için ')' bekleniyor");
        self.consume(TokenType::Semi, "'echo' deyiminden sonra ';' bekleniyor");
        Stmt::Echo(expr)
    }

    // YENİ: print, input gibi yerleşik fonksiyon çağrılarını ayrıştıran fonksiyon
    fn parse_builtin_call_stmt(&mut self) -> Stmt {
        // Bu fonksiyon bir ifade deyimi gibi çalışır: `print(...)` bir `Expr::Call`'dur.
        let expr = self.parse_expression();
        self.consume(TokenType::Semi, "İfade deyiminden sonra ';' bekleniyor");
        Stmt::ExprStmt(expr)
    }

    fn parse_if_stmt(&mut self) -> Stmt {
        self.consume(TokenType::If, "if bekleniyor"); 

        self.consume(TokenType::LParen, "'if' ifadesinden sonra '(' bekleniyor");
        let cond = self.parse_expression(); 
        self.consume(TokenType::RParen, "')' bekleniyor");

        let then_branch = if self.check(&TokenType::LBrace) {
            Box::new(self.parse_block())
        } else {
            Box::new(self.parse_statement())
        };
        
        let else_branch = self.parse_remaining_if_else_chain();

        Stmt::If { cond, then_branch, else_branch }
    }
    
    fn parse_remaining_if_else_chain(&mut self) -> Option<Box<Stmt>> {
        if self.check(&TokenType::ElseIf) {
            self.advance(); 
            
            self.consume(TokenType::LParen, "'elseif' ifadesinden sonra '(' bekleniyor");
            let elseif_cond = self.parse_expression();
            self.consume(TokenType::RParen, "')' bekleniyor");
            
            let elseif_then_branch = if self.check(&TokenType::LBrace) {
                Box::new(self.parse_block())
            } else {
                Box::new(self.parse_statement())
            };

            let elseif_else_branch = self.parse_remaining_if_else_chain();

            return Some(Box::new(Stmt::If { 
                cond: elseif_cond, 
                then_branch: elseif_then_branch, 
                else_branch: elseif_else_branch 
            }));

        } else if self.check(&TokenType::Else) {
            self.advance(); 
            if self.check(&TokenType::LBrace) {
                return Some(Box::new(self.parse_block()));
            } else {
                return Some(Box::new(self.parse_statement()));
            }
        }

        None
    }

    fn parse_block_expr(&mut self) -> Expr {
        self.scope_depth += 1;
        self.consume(TokenType::LBrace, "Blok '{' ile başlamalıdır."); 

        let mut statements = Vec::new();

        while !self.check(&TokenType::RBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()); 
        }

        self.consume(TokenType::RBrace, "Blok '}' ile bitmelidir.");

        self.scope_depth -= 1;
        Expr::Block { statements }
    }
    
    fn parse_return(&mut self) -> Stmt {
        self.consume(TokenType::Return, "'return' bekleniyor");
        
        let value = if !self.check(&TokenType::Semi) {
            Some(self.parse_expression())
        } else {
            None
        };
        
        self.consume(TokenType::Semi, "';' bekleniyor");
        Stmt::Return(value) 
    }

    // --- TDOP: İfade Ayrıştırma Mekanizması ---

    fn map_op(&mut self, kind: &TokenType) -> BinOp {
        match kind {
            TokenType::Plus => BinOp::Add,
            TokenType::Minus => BinOp::Sub,
            TokenType::Star => BinOp::Mul,
            TokenType::Slash => BinOp::Div,
            TokenType::Modulo => BinOp::Mod,
            
            TokenType::Eq => BinOp::Equal,
            TokenType::Ne => BinOp::NotEqual,
            TokenType::Identical => BinOp::Identical,
            TokenType::NotIdentical => BinOp::NotIdentical,
            TokenType::LessGreater => BinOp::NotEqual,
            TokenType::LogAnd => BinOp::And,
            TokenType::LogOr => BinOp::Or,
            
            TokenType::Ampersand => BinOp::BitwiseAnd,
            TokenType::Pipe => BinOp::BitwiseOr,
            TokenType::Caret => BinOp::BitwiseXor,
            TokenType::LShift => BinOp::LShift,
            TokenType::RShift => BinOp::RShift,
            
            TokenType::Gt => BinOp::Greater,
            TokenType::Ge => BinOp::GreaterEqual,
            TokenType::Lt => BinOp::Less,
            TokenType::Le => BinOp::LessEqual,
            
            TokenType::PlusEq => BinOp::Add, 
            TokenType::MinusEq => BinOp::Sub, 
            TokenType::StarEq => BinOp::Mul, 
            TokenType::SlashEq => BinOp::Div, 
            TokenType::PercentEq => BinOp::Mod, 
            
            TokenType::Assign => {
                self.errors.push("İç Hata: Atama operatörü '=' beklenmedik bir şekilde map_op içinde işlendi.".to_string());
                BinOp::Add // Hata durumunda varsayılan bir değer döndür
            },
            _ => {
                self.errors.push(format!("İç Hata: Desteklenmeyen ikili operatör: {:?}", kind));
                BinOp::Add // Hata durumunda varsayılan bir değer döndür
            }
        }
    }

    #[allow(dead_code)]
    fn map_un_op(&mut self, kind: &TokenType) -> UnOp {
        match kind {
            TokenType::Minus => UnOp::Neg,
            TokenType::Exclamation => UnOp::Not, 
            TokenType::Tilde => UnOp::BitwiseNot,
            TokenType::Inc => UnOp::PreInc, 
            TokenType::Dec => UnOp::PreDec, 
            
            _ => {
                self.errors.push(format!("İç Hata: Desteklenmeyen tekli operatör: {:?}", kind));
                UnOp::Not // Hata durumunda varsayılan bir değer döndür
            }
        }
    }

    fn parse_assignment_expression(&mut self) -> Expr {
        let left = self.parse_logical_or_expression();

        let token_kind = self.peek_kind();
        if matches!(token_kind, TokenType::Assign | TokenType::PlusEq | TokenType::MinusEq | TokenType::StarEq | TokenType::SlashEq | TokenType::PercentEq | TokenType::AndEq | TokenType::OrEq | TokenType::XorEq | TokenType::LShiftEq | TokenType::RShiftEq) {
            self.advance(); // atama operatörünü tüket
            let right = self.parse_assignment_expression();

            if token_kind == TokenType::Assign {
                // Basit atama: a = b
                return Expr::Assign { left: Box::new(left), value: Box::new(right) };
            } else {
                // Bileşik atama: a += b -> a = a + b
                let op = self.map_op(&token_kind);
                let new_right = Expr::Binary {
                    left: Box::new(left.clone()),
                    op,
                    right: Box::new(right),
                };
                return Expr::Assign { left: Box::new(left), value: Box::new(new_right) };
            }
        } else if self.check(&TokenType::Recv) { // YENİ: Kanal gönderme işlemi (ch <- value)
            self.advance(); // '<-' token'ını tüket
            let value = self.parse_assignment_expression();
            return Expr::Send {
                channel: Box::new(left),
                value: Box::new(value),
            }
        }
        left
    }

    fn parse_logical_or_expression(&mut self) -> Expr {
        let mut expr = self.parse_logical_and_expression();
        while self.check(&TokenType::LogOr) {
            let op_token = self.advance().kind.clone();
            let right = self.parse_logical_and_expression();
            expr = Expr::Binary {
                left: Box::new(expr),
                op: self.map_op(&op_token),
                right: Box::new(right),
            };
        }
        expr
    }

    fn parse_logical_and_expression(&mut self) -> Expr {
        let mut expr = self.parse_bitwise_or_expression();
        while self.check(&TokenType::LogAnd) {
            let op_token = self.advance().kind.clone();
            let right = self.parse_bitwise_or_expression();
            expr = Expr::Binary { left: Box::new(expr), op: self.map_op(&op_token), right: Box::new(right) };
        }
        expr
    }

    fn parse_bitwise_or_expression(&mut self) -> Expr {
        let mut expr = self.parse_bitwise_xor_expression();
        while self.check(&TokenType::Pipe) {
            let op_token = self.advance().kind.clone();
            let right = self.parse_bitwise_xor_expression();
            expr = Expr::Binary { left: Box::new(expr), op: self.map_op(&op_token), right: Box::new(right) };
        }
        expr
    }

    fn parse_bitwise_xor_expression(&mut self) -> Expr {
        let mut expr = self.parse_bitwise_and_expression();
        while self.check(&TokenType::Caret) {
            let op_token = self.advance().kind.clone();
            let right = self.parse_bitwise_and_expression();
            expr = Expr::Binary { left: Box::new(expr), op: self.map_op(&op_token), right: Box::new(right) };
        }
        expr
    }

    fn parse_bitwise_and_expression(&mut self) -> Expr {
        let mut expr = self.parse_equality_expression();
        while self.check(&TokenType::Ampersand) {
            let op_token = self.advance().kind.clone();
            let right = self.parse_equality_expression();
            expr = Expr::Binary { left: Box::new(expr), op: self.map_op(&op_token), right: Box::new(right) };
        }
        expr
    }

    fn parse_equality_expression(&mut self) -> Expr {
        let mut expr = self.parse_comparison_expression();
        while self.check(&TokenType::Eq) || self.check(&TokenType::Ne) || self.check(&TokenType::Identical) || self.check(&TokenType::NotIdentical) {
            let op_token = self.advance().kind.clone();
            let right = self.parse_comparison_expression();
            expr = Expr::Binary { left: Box::new(expr), op: self.map_op(&op_token), right: Box::new(right) };
        }
        expr
    }

    fn parse_comparison_expression(&mut self) -> Expr {
        let mut expr = self.parse_range_expression(); // YENİ: Zincirde `parse_range_expression`'ı çağır.
        while self.check(&TokenType::Gt) || self.check(&TokenType::Ge) || self.check(&TokenType::Lt) || self.check(&TokenType::Le) {
            let op_token = self.advance().kind.clone();
            let right = self.parse_range_expression();
            expr = Expr::Binary { left: Box::new(expr), op: self.map_op(&op_token), right: Box::new(right) };
        }
        expr
    }

    // YENİ: `..` (range) operatörünü ayrıştıran fonksiyon.
    fn parse_range_expression(&mut self) -> Expr {
        let mut expr = self.parse_shift_expression();
        if self.check(&TokenType::Range) {
            self.advance(); // '..' token'ını tüket.
            let end = self.parse_shift_expression();
            expr = Expr::Range { start: Box::new(expr), end: Box::new(end) };
        }
        expr
    }

    // YENİ: Bitsel kaydırma operatörleri için fonksiyon
    fn parse_shift_expression(&mut self) -> Expr {
        let mut expr = self.parse_term_expression();
        while self.check(&TokenType::LShift) || self.check(&TokenType::RShift) {
            let op_token = self.advance().kind.clone();
            let right = self.parse_term_expression();
            expr = Expr::Binary { left: Box::new(expr), op: self.map_op(&op_token), right: Box::new(right) };
        }
        expr
    }

    fn parse_term_expression(&mut self) -> Expr {
        let mut expr = self.parse_factor_expression();
        while self.check(&TokenType::Plus) || self.check(&TokenType::Minus) {
            let op_token = self.advance().kind.clone();
            let right = self.parse_factor_expression();
            expr = Expr::Binary { left: Box::new(expr), op: self.map_op(&op_token), right: Box::new(right) };
        }
        expr
    }

    fn parse_factor_expression(&mut self) -> Expr {
        let mut expr = self.parse_unary();
        while self.check(&TokenType::Star) || self.check(&TokenType::Slash) || self.check(&TokenType::Modulo) {
            let op_token = self.advance().kind.clone();
            let right = self.parse_unary();
            expr = Expr::Binary { left: Box::new(expr), op: self.map_op(&op_token), right: Box::new(right) };
        }
        expr
    }

    fn parse_expression(&mut self) -> Expr {
        self.parse_assignment_expression()
    }

    fn parse_unary(&mut self) -> Expr {
        match self.peek_kind() {
            TokenType::Minus => {
                self.advance(); 
                let right = Box::new(self.parse_unary());
                Expr::Unary { op: UnOp::Neg, right }
            },
            TokenType::Exclamation => { 
                self.advance(); 
                let right = Box::new(self.parse_unary());
                Expr::Unary { op: UnOp::Not, right }
            },
            TokenType::Tilde => { 
                self.advance(); 
                let right = Box::new(self.parse_unary());
                Expr::Unary { op: UnOp::BitwiseNot, right }
            },
            TokenType::Ampersand => { // Address-of operator
                self.advance();
                let right = Box::new(self.parse_unary());
                Expr::Unary { op: UnOp::AddressOf, right }
            },
            TokenType::Star => { // Dereference operator
                self.advance();
                let right = Box::new(self.parse_unary());
                Expr::Unary { op: UnOp::Deref, right }
            },
            // YENİ: Kanal alma işlemi (<-ch)
            TokenType::Recv => {
                self.advance(); // '<-' token'ını tüket
                let channel_expr = self.parse_unary();
                Expr::Recv(Box::new(channel_expr))
            },
            
            TokenType::Inc => { 
                self.advance(); 
                let right = Box::new(self.parse_unary());
                Expr::Unary { op: UnOp::PreInc, right } 
            },
            
            TokenType::Await => {
                self.advance();
                let expr = self.parse_unary();
                Expr::Await(Box::new(expr))
            },
            TokenType::Dec => { 
                self.advance(); 
                let right = Box::new(self.parse_unary());
                Expr::Unary { op: UnOp::PreDec, right } 
            },

            // YENİ: print(), input() gibi yerleşik fonksiyon çağrılarını
            // normal bir fonksiyon çağrısı gibi ele alabilmek için bu durumu ekliyoruz.
            // Bu token'ları bir değişken adı gibi ayrıştırıp parse_call'a iletiyoruz.
            TokenType::Print |
            TokenType::Input |
            TokenType::Strlen |
            TokenType::Arrlen |
            TokenType::Panic |
            TokenType::Exit => self.parse_call(),

            _ => self.parse_call(), 
        }
    }

    fn parse_call(&mut self) -> Expr {
        let mut expr = self.parse_primary();

        loop {
            match self.peek_kind() {
                TokenType::Inc => { 
                    self.advance(); 
                    expr = Expr::Unary { 
                        op: UnOp::PostInc, 
                        right: Box::new(expr) 
                    }; 
                    continue; 
                },
                
                TokenType::Dec => { 
                    self.advance(); 
                    expr = Expr::Unary { 
                        op: UnOp::PostDec, 
                        right: Box::new(expr)
                    }; 
                    continue; 
                },
                TokenType::Question => {
                    self.advance();
                    expr = Expr::Try(Box::new(expr));
                    // `?` operatöründen sonra başka bir `?` veya `()` gelebilir,
                    // bu yüzden döngüye devam ediyoruz.
                    continue;
                },
                
                _ => {},
            }
            
            if self.check(&TokenType::LBracket) {
                self.advance(); 
                
                let index_expr = self.parse_expression(); 
                
                self.consume(TokenType::RBracket, "Dizi erişimi için ']' bekleniyor.");

                let array_name = if let Expr::Variable(name) = expr {
                    name
                } else {
                    self.errors.push(format!(
                        "Sözdizimi Hatası (Satır {}): Dizi erişimi şimdilik sadece basit değişken adları üzerinden yapılabilir.", self.peek().line
                    ));
                    "__invalid_array__".to_string()
                };
                
                expr = Expr::ArrayAccess { 
                    name: array_name,               
                    index: Box::new(index_expr)     
                };
                
            } else if self.check(&TokenType::LParen) {
                self.advance(); 

                let mut args = Vec::new();
                if !self.check(&TokenType::RParen) {
                    loop {
                        let arg_name = if self.check_next(&TokenType::Colon) {
                            if let TokenType::Ident(name) = self.peek_kind() {
                                self.advance(); 
                                self.advance(); 
                                Some(name)
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        args.push((arg_name, self.parse_expression()));

                        if self.check(&TokenType::Comma) {
                            self.advance(); 
                            if self.check(&TokenType::RParen) { 
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
                self.consume(TokenType::RParen, "Fonksiyon çağrısı için ')' bekleniyor.");

                expr = Expr::Call { 
                    callee: Box::new(expr), 
                    args 
                };
            } else if self.check(&TokenType::Colon) && self.check_next(&TokenType::Colon) {
                // :: operatörü için (statik metodlar/namespace erişimi)
                self.advance(); // :
                self.advance(); // :
                // Şimdilik bunu normal üye erişimi gibi ele alalım, ileride ayrıştırılabilir.
                // Bu, `expr = self.parse_static_member_access(expr);` gibi bir yapıya dönüşebilir.
                
            } else if self.check(&TokenType::Dot) {
                self.advance(); 

                match self.peek_kind() {
                    TokenType::Ident(name) => {
                        self.advance(); 
                        expr = Expr::MemberAccess { 
                            object: Box::new(expr), 
                            member: name 
                        };
                    },
                    _ => {
                        self.errors.push(format!("Sözdizimi Hatası (Satır {}): Üye erişiminden sonra bir tanımlayıcı bekleniyor. Bulunan token: {:?}", self.peek().line, self.peek_kind()));
                        self.synchronize(); 
                    }
                }

            } else {
                break;
            }
        }
        
        expr
    }
    
    fn parse_struct_literal(&mut self, name: String) -> Expr {
        self.consume(TokenType::LBrace, "Struct literal'ı için '{' bekleniyor.");

        let mut fields = Vec::new();
        while !self.check(&TokenType::RBrace) && !self.is_at_end() {
            let field_name = match self.peek_kind() {
                TokenType::Ident(n) => {
                    self.advance();
                    n
                }
                _ => {
                    self.errors.push(format!("Sözdizimi Hatası (Satır {}): Struct literal'ında alan adı bekleniyor.", self.peek().line));
                    break;
                }
            };
            self.consume(TokenType::Colon, "Struct literal alanından sonra ':' bekleniyor.");
            let value = self.parse_expression();
            fields.push((field_name, value));

            if self.check(&TokenType::Comma) {
                self.advance();
            }
        }
        self.consume(TokenType::RBrace, "Struct literal'ını kapatmak için '}' bekleniyor.");

        Expr::StructLiteral { name, fields }
    }

    fn parse_primary(&mut self) -> Expr {
        match self.peek_kind() {
            TokenType::IntLit(i) => { self.advance(); Expr::Literal(LiteralValue::Int(i)) },
            TokenType::FloatLit(f) => { self.advance(); Expr::Literal(LiteralValue::Float(f)) },
            TokenType::HexLit(h) => { self.advance(); Expr::Literal(LiteralValue::Hex(h as u64)) },
            TokenType::StrLit(s) => { self.advance(); Expr::Literal(LiteralValue::Str(s)) },
            TokenType::CharLit(c) => { self.advance(); Expr::Literal(LiteralValue::Char(c)) },
            TokenType::True => { self.advance(); Expr::Literal(LiteralValue::Bool(true)) },
            TokenType::False => { self.advance(); Expr::Literal(LiteralValue::Bool(false)) },
            TokenType::Null => { self.advance(); Expr::Literal(LiteralValue::Null) },

            TokenType::InterpolatedStr(full_string) => {
                self.advance();
                let mut parts = Vec::new();
                let mut last_end = 0;

                while let Some(start) = full_string[last_end..].find('{') {
                    let absolute_start = last_end + start;

                    // '{' karakterinden önceki literal kısmı ekle
                    if absolute_start > last_end {
                        parts.push(Expr::Literal(LiteralValue::Str(full_string[last_end..absolute_start].to_string())));
                    }

                    // Dengeli '}' karakterini bul
                    let mut balance = 1;
                    let mut end = None;
                    for (i, c) in full_string[absolute_start + 1..].char_indices() {
                        if c == '{' {
                            balance += 1;
                        } else if c == '}' {
                            balance -= 1;
                            if balance == 0 {
                                end = Some(absolute_start + 1 + i);
                                break;
                            }
                        }
                    }

                    if let Some(absolute_end) = end {
                        let expr_str = &full_string[absolute_start + 1 .. absolute_end];
                        
                        // İfadeyi ayrıştır
                        let mut temp_lexer = crate::lexer::Lexer::new(expr_str.trim());
                        let mut tokens = Vec::new();
                        loop {
                            let token = temp_lexer.next_token();
                            if token.kind == TokenType::Eof { break; }
                            tokens.push(token);
                        }
                        
                        // YENİ: Geçici parser'a sadece ifadeyi ayrıştırmasını söyle.
                        let mut temp_parser = Parser::new(tokens);
                        let expr = temp_parser.parse_expression();
                        parts.push(expr);

                        last_end = absolute_end + 1;
                    } else {
                        // Eşleşmeyen '{' hatası
                        self.errors.push(format!("Sözdizimi Hatası (Satır {}): İnterpolasyonlu string içinde kapanmamış '{{' bulundu.", self.peek().line));
                        last_end = full_string.len();
                        break;
                    }
                }

                // String'in geri kalanını ekle
                if last_end < full_string.len() {
                    parts.push(Expr::Literal(LiteralValue::Str(full_string[last_end..].to_string())));
                }
                Expr::InterpolatedString(parts)
            },
            
            TokenType::Def => {
                self.advance(); 
                Expr::DefaultCase 
            },
            TokenType::Sizeof => {
                self.advance();
                self.consume(TokenType::LParen, "'sizeof' sonrası '(' bekleniyor.");
                let ty = self.parse_type();
                self.consume(TokenType::RParen, "'sizeof' sonrası ')' bekleniyor.");
                Expr::SizeOf(ty)
            },
            TokenType::Ident(name) => {
                self.advance(); 
                // YENİ: Struct literal için '{' kontrolü
                if self.check(&TokenType::LBrace) {
                    return self.parse_struct_literal(name);
                }
                // YENİ: Enum üye erişimi için `::` kontrolü
                if self.check(&TokenType::Colon) && self.check_next(&TokenType::Colon) {
                    self.advance(); // :
                    self.advance(); // :
                    let variant_name = match self.peek_kind() {
                        TokenType::Ident(v_name) => { self.advance(); v_name },
                        _ => {
                            self.errors.push(format!("Sözdizimi Hatası (Satır {}): '::' operatöründen sonra bir enum üyesi bekleniyor.", self.peek().line));
                            "__invalid_variant__".to_string()
                        }
                    };
                    return Expr::EnumAccess { enum_name: name, variant_name };
                }
                Expr::Variable(name)
            },
            // YENİ: 'self' anahtar kelimesini bir ifade olarak tanı.
            TokenType::Self_ => {
                self.advance();
                Expr::Variable("self".to_string())
            },
            // YENİ: print, input gibi yerleşik fonksiyonları bir değişken adı gibi ele al.
            // Bu, parse_call'un onları bir fonksiyon çağrısı olarak işlemesini sağlar.
            TokenType::Print => { self.advance(); Expr::Variable("print".to_string()) },
            TokenType::Input => {
                self.advance(); // 'input' kelimesini tüket
                
                // Parantezleri ve opsiyonel argümanı işle
                self.consume(TokenType::LParen, "input'tan sonra '(' bekleniyor.");
                
                let mut prompt = None;
                if self.peek_kind() != TokenType::RParen {
                    prompt = Some(Box::new(self.parse_expression())); // İçerideki ifadeyi parse et
                }
                
                self.consume(TokenType::RParen, "input'tan sonra ')' bekleniyor.");
                
                // Doğru dönüş yapısı:
                Expr::Input(prompt) 
            }
            TokenType::Strlen => { self.advance(); Expr::Variable("strlen".to_string()) },
            TokenType::Arrlen => { self.advance(); Expr::Variable("arrlen".to_string()) },
            TokenType::Panic => { self.advance(); Expr::Variable("panic".to_string()) },
            TokenType::Exit => { self.advance(); Expr::Variable("exit".to_string()) },
            TokenType::Echo => { self.advance(); Expr::Variable("echo".to_string()) },

            TokenType::LParen => {
                self.advance(); 

                if self.check(&TokenType::RParen) {
                    self.consume(TokenType::RParen, "')' bekleniyor");
                    return Expr::Tuple(Vec::new()); 
                }

                let first_expr = self.parse_expression();

                if self.check(&TokenType::Comma) {
                    self.advance(); 
                    let mut elements = vec![first_expr];
                    
                    while !self.check(&TokenType::RParen) {
                        elements.push(self.parse_expression());

                        if self.check(&TokenType::RParen) {
                            break;
                        }
                        
                        self.consume(TokenType::Comma, "Tuple elemanları arasında ',' bekleniyor.");
                    }

                    self.consume(TokenType::RParen, "Tuple için son ')' bekleniyor");
                    Expr::Tuple(elements)
                } else {
                    self.consume(TokenType::RParen, "Normal parantezli ifade için ')' bekleniyor");
                    first_expr
                }
            },
            
            TokenType::Match => {
                self.advance(); 
                return self.parse_match_expr(); 
            },
            
            TokenType::LBracket => {
                self.advance(); 
                let mut elements = Vec::new();

                if !self.check(&TokenType::RBracket) {
                    loop {
                        elements.push(self.parse_expression());
                        if !self.check(&TokenType::Comma) {
                            break; 
                        }
                        self.advance(); 
                        if self.check(&TokenType::RBracket) {
                            break;
                        }
                    }
                }

                self.consume(TokenType::RBracket, "Dizi tanımı için ']' bekleniyor.");
                Expr::ArrayLiteral(elements)
            },

            TokenType::Fn => {
                self.advance();
                self.consume(TokenType::LParen, "Lambda için '(' bekleniyor");
                let params = self.parse_function_params();
                self.consume(TokenType::RParen, "Lambda için ')' bekleniyor");
                self.consume(TokenType::Colon, "Lambda dönüş tipi için ':' bekleniyor");
                let return_type = self.parse_type();
                self.consume(TokenType::Arrow, "Lambda gövdesi için '->' bekleniyor");
                
                // YENİ: Lambda gövdesi bir blok veya tek bir ifade olabilir.
                let body = if self.check(&TokenType::LBrace) {
                    self.parse_block_expr()
                } else {
                    self.parse_expression()
                };

                Expr::Lambda {
                    params,
                    return_type,
                    body: Box::new(body),
                }
            },

            _ => {
                self.errors.push(format!("Sözdizimi Hatası (Satır {}): Birincil ifade bekleniyor. Bulunan token: {:?}", self.peek().line, self.peek_kind()));
                self.synchronize();
                // Hata durumunda Null döndürerek devam et
                Expr::Literal(LiteralValue::Null)
            }
        }
    }

    // --- Tip Ayrıştırma ---
    
    fn parse_type(&mut self) -> Type {
        if self.check(&TokenType::Ampersand) {
            self.advance(); 
            let inner_type = Box::new(self.parse_type());
            Type::Ref(inner_type)

        } else if self.check(&TokenType::Star) {
            self.advance(); 
            let inner_type = Box::new(self.parse_type());
            Type::Ptr(inner_type)
            
        } else if self.check(&TokenType::LBracket) {
            self.errors.push(format!(
                "Sözdizimi Hatası (Satır {}): Köşeli parantez ('[') tip tanımının başında beklenmiyor. Dizi tipi 'var my_array[10]: i32;' şeklinde tanımlanmalıdır.", self.peek().line
            ));
            return Type::Unknown;
        
        } else if self.check(&TokenType::LParen) {
            self.advance(); 
            let mut types = Vec::new();

            if !self.check(&TokenType::RParen) {
                loop {
                    types.push(self.parse_type());
                    if !self.check(&TokenType::Comma) { break; }
                    self.advance(); 
                }
            }
            self.consume(TokenType::RParen, "Tuple tipi için son ')' bekleniyor");
            Type::Tuple(types)

        } else {
            self.parse_base_type()
        }
    }

    fn parse_base_type(&mut self) -> Type {
        let t = match self.peek_kind() {
            TokenType::TypeI8 => Type::I8,
            TokenType::TypeI16 => Type::I16,
            TokenType::TypeI32 => Type::I32,
            TokenType::TypeI64 => Type::I64,
            TokenType::TypeI128 => Type::I128,

            TokenType::TypeU8 => Type::U8,
            TokenType::TypeU16 => Type::U16,
            TokenType::TypeU32 => Type::U32,
            TokenType::TypeU64 => Type::U64,
            TokenType::TypeU128 => Type::U128,

            TokenType::TypeF32 => Type::F32,
            TokenType::TypeF64 => Type::F64,
            TokenType::TypeF80 => Type::F80,
            TokenType::TypeF128 => Type::F128,

            TokenType::TypeD32 => Type::D32,
            TokenType::TypeD64 => Type::D64,
            TokenType::TypeD128 => Type::D128,

            TokenType::TypeBool => Type::Bool,
            TokenType::TypeChar => Type::Char,
            TokenType::TypeBit => Type::Bit,
            TokenType::TypeByte => Type::Byte,
            TokenType::TypeHex => Type::Hex,
            TokenType::TypeDec => Type::D32, 

            TokenType::TypeVoid => Type::Void,
            TokenType::TypeAny => Type::Any,
            TokenType::TypeStr => Type::Str(None), 
            TokenType::TypeArr => Type::Arr, 
            TokenType::TypePtr => {
                self.errors.push(format!("Sözdizimi Hatası (Satır {}): 'ptr' anahtar kelimesi yerine '*' kullanılmalıdır.", self.peek().line));
                Type::Unknown
            },
            TokenType::TypeRef => {
                self.errors.push(format!("Sözdizimi Hatası (Satır {}): 'ref' anahtar kelimesi yerine '&' kullanılmalıdır.", self.peek().line));
                Type::Unknown
            },
            
            TokenType::Fn => {
                self.advance();
                self.consume(TokenType::LParen, "'(' bekleniyor");
                let mut param_types = Vec::new();
                while !self.check(&TokenType::RParen) {
                    param_types.push(self.parse_type());
                    if !self.check(&TokenType::Comma) {
                        break;
                    }
                    self.advance();
                }
                self.consume(TokenType::RParen, "')' bekleniyor");
                self.consume(TokenType::Arrow, "'->' bekleniyor");
                let return_type = self.parse_type();
                Type::Fn(param_types, Box::new(return_type))
            }
            TokenType::Ident(s) => {
                if s == "Channel" && self.check_next(&TokenType::Lt) {
                    self.advance(); // 'Channel'
                    self.advance(); // '<'
                    let inner_type = self.parse_type();
                    self.consume(TokenType::Gt, "'>' bekleniyor");
                    return Type::Channel(Box::new(inner_type));
                }
                if s == "Result" && self.check_next(&TokenType::Lt) {
                    self.advance(); // 'Result'
                    self.advance(); // '<'
                    let ok_type = self.parse_type();
                    self.consume(TokenType::Comma, "Result tipi için ',' bekleniyor.");
                    let err_type = self.parse_type();
                    self.consume(TokenType::Gt, "'>' bekleniyor");
                    return Type::Result(Box::new(ok_type), Box::new(err_type));
                } else {
                    // Normal bir struct veya enum adı.
                    // Type checker, bunun bir enum olup olmadığını ve temel tipini belirleyecektir.
                    // Şimdilik Custom olarak işaretliyoruz, type checker bunu Enum'a dönüştürecek.
                    Type::Custom(s.clone())
                }
            },

            _ => {
                self.errors.push(format!("Sözdizimi Hatası (Satır {}): Geçerli bir tip bekleniyor, bulundu: {:?}", self.peek().line, self.peek_kind()));
                self.synchronize();
                // Hata durumunda Unknown döndürerek devam et
                return Type::Unknown;
            }
        };
        self.advance();
        t
    }
}