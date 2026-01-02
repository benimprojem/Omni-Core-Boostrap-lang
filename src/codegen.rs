// src/codegen.rs

use crate::ast::{Decl, Stmt, Expr, LiteralValue, TargetPlatform, Type, BinOp, UnOp};
use crate::type_checker::TypeChecker;

// Platforma özel kod üretimi modülleri
//mod windows;
//mod linux;
//mod macos;

#[derive(Debug, Clone)]
struct VariableLocation {
    stack_offset: i32,
    // Değişkenin tipini de burada saklayacağız.
    // Bu, TypeChecker'a tekrar sormamızı engeller.
    ty: Type,
    // Array uzunluğu (Type::Arr için gerekli)
    array_len: Option<usize>,
}

// Veri segmentindeki farklı öğeleri temsil etmek için.
#[derive(Debug, Clone)]
enum DataItem {
    String(String),
    Float64(f64),
}

pub struct Codegen<'a, 'b> {
    pub program: &'a [Decl], // Reference to the whole program AST
    pub type_checker: &'b mut TypeChecker<'a>, // Reference to the TypeChecker (mutable)
    pub target_platform: TargetPlatform,
    pub current_function_name: String, // Hangi fonksiyonun kodunu ürettiğimizi takip etmek için
    // string_literals yerine data_items kullanıyoruz.
    data_items: Vec<DataItem>,
    #[allow(dead_code)] // Şimdilik kullanılmıyor, ileride kontrol akışı için kullanılacak.
    pub label_counter: usize, // Benzersiz etiketler oluşturmak için
    variable_locations: std::collections::HashMap<String, VariableLocation>, // Değişkenlerin konumları
    stack_pointer: i32, // Mevcut stack offset'i
    loop_labels: Vec<(String, String)>, // (start_label, end_label)
}


impl<'a, 'b> Codegen<'a, 'b> {
    pub fn new(
        program: &'a [Decl],
        type_checker: &'b mut TypeChecker<'a>, // Changed to &mut
        target_platform: TargetPlatform,
    ) -> Self {
        Self {
            program,
            type_checker,
            target_platform,
            current_function_name: String::new(),
            data_items: Vec::new(),
            label_counter: 0,
            variable_locations: std::collections::HashMap::new(),
            stack_pointer: 0,
            loop_labels: Vec::new(),
        }
    }

    pub fn generate(&mut self) -> Result<String, String> {
        let mut full_asm = String::new();

        // 1. GAS (GNU Assembler) için Intel sözdizimi ve prefix ayarları
        full_asm.push_str(".intel_syntax noprefix\n\n");

        // 2. Metin segmentini (kod) oluştur. Bu aşamada string ve float literalleri toplanır.
        let text_segment = self.generate_text_segment()?;

        // 3. Toplanan literallerle veri segmentini oluştur ve ekle.
        full_asm.push_str(&self.generate_data_segment());
        full_asm.push_str(&text_segment);

        Ok(full_asm)
    }

    fn generate_data_segment(&mut self) -> String {
        let mut asm = String::new();
        asm.push_str(".section .data\n");
        for (i, item) in self.data_items.iter().enumerate() {
            match item {
                DataItem::String(s) => {
                    // GAS için .asciz kullanımı (null-terminated string)
                    let escaped = s.replace("\\", "\\\\")
                                    .replace("\"", "\\\"")
                                    .replace("\n", "\\n")
                                    .replace("\r", "\\r")
                                    .replace("\x1b", "\\033"); // GAS için octal escape
                    asm.push_str(&format!("str_{}: .asciz \"{}\"\n", i, escaped));
                }
                DataItem::Float64(f) => {
                    // GAS için .double (8 byte float)
                    asm.push_str(&format!("float_{}: .double {}\n", i, f));
                }
            }
        }
        asm.push_str("\n");
        asm
    }

    fn generate_text_segment(&mut self) -> Result<String, String> {
        let mut asm = String::new();
        asm.push_str(".section .text\n");

        // Platforma özel giriş noktası ve dış fonksiyon bildirimleri
        match self.target_platform {
            TargetPlatform::Windows => {
                // GAS'ta harici semboller için özel 'extern' gerekmez (direkt call edilebilir),
                // ama '.global' veya sembol kullanımı yeterlidir.
                asm.push_str(".global main\n\n");
            }
            TargetPlatform::Linux => {
                asm.push_str(".global _start\n\n");
            }
            TargetPlatform::Macos => {
                asm.push_str(".global _main\n\n");
            }
            _ => {}
        }

        // Ana program AST'sini gez ve tüm fonksiyonları üret
        for decl in self.program.iter() {
            if let Decl::Function { name, params, body, .. } = decl {
                self.current_function_name = name.clone();
                self.stack_pointer = 0;
                self.variable_locations.clear();

                let label = if name == "main" { self.get_entry_point_label() } else { name.clone() };
                asm.push_str(&format!("{}:\n", label));
                
                // Fonksiyon başlangıcı (prologue)
                asm.push_str("    push rbp\n");
                asm.push_str("    mov rbp, rsp\n");
                asm.push_str("    sub rsp, 256 # Geçici sabit stack alanı\n\n");

                if name == "main" && self.target_platform == TargetPlatform::Windows {
                    asm.push_str("    mov ecx, 65001 # CP_UTF8\n    call SetConsoleOutputCP\n");
                    
                    asm.push_str("    # ANSI Renk Desteğini Etkinleştir (Virtual Terminal Processing)\n");
                    asm.push_str("    mov rcx, -11 # STD_OUTPUT_HANDLE\n");
                    asm.push_str("    call GetStdHandle\n");
                    asm.push_str("    mov [rbp - 248], rax # Handle'ı sakla\n");
                    asm.push_str("    lea rdx, [rbp - 240] # Mode için geçici alan\n");
                    asm.push_str("    mov rcx, [rbp - 248]\n");
                    asm.push_str("    call GetConsoleMode\n");
                    asm.push_str("    mov eax, [rbp - 240]\n");
                    asm.push_str("    or eax, 4 # ENABLE_VIRTUAL_TERMINAL_PROCESSING\n");
                    asm.push_str("    mov rdx, rax\n");
                    asm.push_str("    mov rcx, [rbp - 248]\n");
                    asm.push_str("    call SetConsoleMode\n");
                }

                // Kod üretimi sırasında tip kontrolcü kapsamını da yönetiyoruz.
                self.type_checker.push_scope();

                // Nimble'ın özel main yapısı (argc, argv)
                if name == "main" {
                    // RCX = argc, RDX = argv (Windows x64 ABI)
                    self.stack_pointer += 16;
                    let argc_offset = self.stack_pointer - 8;
                    let argv_offset = self.stack_pointer;
                    
                    self.variable_locations.insert("argc".to_string(), VariableLocation { stack_offset: argc_offset, ty: Type::I32, array_len: None });
                    self.variable_locations.insert("argv".to_string(), VariableLocation { stack_offset: argv_offset, ty: Type::Ptr(Box::new(Type::Str(None))), array_len: None });
                    
                    self.type_checker.define_variable("argc".to_string(), crate::type_checker::VarInfo { ty: Type::I32, is_const: false, _is_mutable: false })?;
                    self.type_checker.define_variable("argv".to_string(), crate::type_checker::VarInfo { ty: Type::Ptr(Box::new(Type::Str(None))), is_const: false, _is_mutable: false })?;

                    asm.push_str(&format!("    mov [rbp - {}], rcx # Store argc\n", argc_offset));
                    asm.push_str(&format!("    mov [rbp - {}], rdx # Store argv\n", argv_offset));
                } else {
                    // Normal Fonksiyon Parametreleri
                    let arg_regs = ["rcx", "rdx", "r8", "r9"];
                    for (i, (p_name, p_ty, _)) in params.iter().enumerate() {
                        self.stack_pointer += 8;
                        let offset = self.stack_pointer;
                        self.variable_locations.insert(p_name.clone(), VariableLocation { stack_offset: offset, ty: p_ty.clone(), array_len: None });
                        
                        self.type_checker.define_variable(p_name.clone(), crate::type_checker::VarInfo { ty: p_ty.clone(), is_const: false, _is_mutable: true })?;

                        if i < 4 {
                            asm.push_str(&format!("    mov [rbp - {}], {} # Store parameter '{}'\n", offset, arg_regs[i], p_name));
                        } else {
                            // 5. ve sonrası stack: [rbp + 16], [rbp + 24]...
                            let param_offset = 16 + (i - 4) * 8;
                            asm.push_str(&format!("    mov rax, [rbp + {}]\n", param_offset));
                            asm.push_str(&format!("    mov [rbp - {}], rax # Store stack parameter '{}'\n", offset, p_name));
                        }
                    }
                }

                asm.push_str(&self.generate_stmt(body)?);
                self.type_checker.pop_scope()?;

                // Fonksiyonu sonlandır (epilogue)
                if name == "main" {
                    asm.push_str(&self.generate_exit_code());
                } else {
                    asm.push_str(&format!(".fn_exit_{}:\n", name)); // Return'lerin atlaması için
                    asm.push_str("    add rsp, 256\n");
                    asm.push_str("    pop rbp\n");
                    asm.push_str("    ret\n");
                }
            }
        }

        // NOT: Artık harici _print.obj kullanılıyor.
        
        // Yardımcı kütüphaneleri (atoi, itoa vb.) ekle
        asm.push_str(&self.generate_builtins_library());

        Ok(asm)
    }

    // Platforma özel giriş noktası etiketini döndürür
    fn get_entry_point_label(&self) -> String {
        match self.target_platform {
            TargetPlatform::Windows => "main".to_string(),
            TargetPlatform::Linux => "_start".to_string(),
            TargetPlatform::Macos => "_main".to_string(),
            _ => "main".to_string(), // Varsayılan
        }
    }

    // Programı sonlandıran platforma özel assembly kodu
    fn generate_exit_code(&self) -> String {
        match self.target_platform {
            TargetPlatform::Windows => {
                "    xor rcx, rcx     # ExitProcess için çıkış kodu 0\n    call ExitProcess\n".to_string()
            }
            TargetPlatform::Linux => {
                "    mov rax, 60        # exit için syscall numarası\n    xor rdi, rdi       # çıkış kodu 0\n    syscall\n".to_string()
            }
            TargetPlatform::Macos => {
                "    xor rdi, rdi     # exit için çıkış kodu 0\n    call _exit\n".to_string()
            }
            _ => "".to_string(),
        }
    }

    // Deyimleri assembly koduna çevirir
    fn generate_stmt(&mut self, stmt: &Stmt) -> Result<String, String> {
        match stmt {
            Stmt::Block(stmts) => {
                self.type_checker.push_scope();
                let mut asm = String::new();
                for s in stmts {
                    asm.push_str(&self.generate_stmt(s)?);
                }
                self.type_checker.pop_scope()?;
                Ok(asm)
            }
            Stmt::VarDecl { name, ty, init, .. } => {
                let mut code = String::new();
                if let Some(init_expr) = init {
                    // 1. Evaluate the initializer expression. The result will be in RAX.
                    code.push_str(&self.generate_expr(init_expr)?);
                    
                    // Type::Arr veya Type::Array için array logic
                    if *ty == Type::Arr || matches!(ty, Type::Array(_, _)) {
                        // ArrayLiteral'den boyutu al
                        let len = if let Expr::ArrayLiteral(elements) = init_expr {
                            elements.len()
                        } else {
                            0 // Boş array veya başka bir ifade
                        };
                        
                        // Değişken için alan ayır: len * 8 byte
                        let array_start_offset = self.stack_pointer + 8; // İlk eleman
                        self.stack_pointer += (len * 8) as i32;
                        
                        let location = VariableLocation { stack_offset: array_start_offset, ty: ty.clone(), array_len: Some(len) };
                        self.variable_locations.insert(name.clone(), location);
                        self.type_checker.define_variable(name.clone(), crate::type_checker::VarInfo { ty: ty.clone(), is_const: false, _is_mutable: true })?;
                        
                        // RAX: Source Address (ArrayLiteral sonucu)
                        // Kopyalama döngüsü
                        code.push_str("    mov rsi, rax        # Source address\n");
                        
                        code.push_str("    mov rcx, 0\n");
                        let copy_loop = self.generate_label("copy_loop");
                        let copy_done = self.generate_label("copy_done");
                        code.push_str(&format!("{}:\n", copy_loop));
                        code.push_str(&format!("    cmp rcx, {}\n", len));
                        code.push_str(&format!("    jge {}\n", copy_done));
                        
                        // Load from Source (RSI + rcx*8)
                        code.push_str("    mov rbx, [rsi + rcx*8]\n");
                        // Store to Dest (RBP - offset + rcx*8)
                        code.push_str(&format!("    mov [rbp - {} + rcx*8], rbx\n", array_start_offset));
                        
                        code.push_str("    inc rcx\n");
                        code.push_str(&format!("    jmp {}\n", copy_loop));
                        code.push_str(&format!("{}:\n", copy_done));
                        
                    } else {
                        // Tamsayı, Pointer veya Float
                        self.stack_pointer += 8;
                        let offset = self.stack_pointer;
                        let location = VariableLocation { stack_offset: offset, ty: ty.clone(), array_len: None };
                        self.variable_locations.insert(name.clone(), location);
                        self.type_checker.define_variable(name.clone(), crate::type_checker::VarInfo { ty: ty.clone(), is_const: false, _is_mutable: true })?;

                        if ty.is_float() {
                            code.push_str(&format!("    movsd [rbp - {}], xmm0 # Store float variable '{}'\n", offset, name));
                        } else {
                            code.push_str(&format!("    mov [rbp - {}], rax # Store integer/pointer variable '{}'\n", offset, name));
                        }
                    }
                }
                Ok(code)
            }
            Stmt::Return(expr_opt) => {
                let mut code = String::new();
                if let Some(expr) = expr_opt {
                    // Dönüş değerini RAX (veya XMM0) üzerine yükle
                    code.push_str(&self.generate_expr(expr)?);
                }
                
                if self.current_function_name == "main" {
                    code.push_str(&self.generate_exit_code());
                } else {
                    // Fonksiyon epiloguna atla
                    code.push_str(&format!("    jmp .fn_exit_{}\n", self.current_function_name));
                }
                Ok(code)
            }
            Stmt::Echo(expr) => self.generate_echo_call(expr),
            Stmt::Assign { left, value } => {
                let mut code = String::new();
                // 1. Sağ tarafı değerlendir
                code.push_str(&self.generate_expr(value)?);
                
                // 2. Sol tarafın konumunu bul ve ata
                if let Expr::Variable(name) = &*left {
                    let loc = self.variable_locations.get(name).ok_or_else(|| format!("Atama hatası: Bilinmeyen değişken '{}'", name))?;
                    if loc.ty.is_float() {
                        code.push_str(&format!("    movsd [rbp - {}], xmm0 # Assign to float variable '{}'\n", loc.stack_offset, name));
                    } else {
                        code.push_str(&format!("    mov [rbp - {}], rax # Assign to integer/pointer variable '{}'\n", loc.stack_offset, name));
                    }
                } else {
                    return Err("Şimdilik sadece değişkenlere atama destekleniyor.".to_string());
                }
                Ok(code)
            }
            Stmt::If { cond, then_branch, else_branch } => {
                let mut code = String::new();
                let else_label = self.generate_label("L_else");
                let end_label = self.generate_label("L_if_end");

                // 1. Koşulu değerlendir
                code.push_str(&self.generate_expr(cond)?);
                code.push_str("    test rax, rax\n");
                
                if else_branch.is_some() {
                    code.push_str(&format!("    jz {}\n", else_label));
                } else {
                    code.push_str(&format!("    jz {}\n", end_label));
                }

                // 2. 'Then' bloğu
                code.push_str(&self.generate_stmt(then_branch)?);
                
                if else_branch.is_some() {
                    code.push_str(&format!("    jmp {}\n", end_label));
                    // 3. 'Else' bloğu
                    code.push_str(&format!("{}:\n", else_label));
                    if let Some(else_stmt) = else_branch {
                        code.push_str(&self.generate_stmt(else_stmt)?);
                    }
                }

                code.push_str(&format!("{}:\n", end_label));
                Ok(code)
            }
            Stmt::While { condition, body } => {
                let start_label = self.generate_label("L_while_start");
                let end_label = self.generate_label("L_while_end");
                let mut code = String::new();

                code.push_str(&format!("{}:\n", start_label));
                // 1. Koşulu değerlendir
                code.push_str(&self.generate_expr(condition)?);
                code.push_str("    test rax, rax\n");
                code.push_str(&format!("    jz {}\n", end_label));

                // 2. Döngü gövdesi
                code.push_str(&self.generate_stmt(body)?);
                code.push_str(&format!("    jmp {}\n", start_label));

                code.push_str(&format!("{}:\n", end_label));
                Ok(code)
            }
            Stmt::For { initializer, condition, increment, variable, iterable, body } => {
                let start_label = self.generate_label("L_for_start");
                let end_label = self.generate_label("L_for_end");
                let mut code = String::new();

                // 1. Initializer / Iterator Setup
                if let (Some(var_name), Some(iter_expr)) = (variable, iterable) {
                    // Range for: for i in 0..10
                    if let Expr::Range { start, end } = iter_expr {
                        // Mevcut Range mantığı
                        let old_location = self.variable_locations.get(var_name).cloned();
                        let target_offset = if let Some(loc) = &old_location {
                            loc.stack_offset
                        } else {
                            self.stack_pointer += 8;
                            let offset = self.stack_pointer;
                            self.variable_locations.insert(var_name.clone(), VariableLocation { stack_offset: offset, ty: Type::I32, array_len: None });
                            offset
                        };
                        
                        self.type_checker.push_scope();
                        
                        // 0'dan başlat (start)
                        code.push_str(&self.generate_expr(start)?);
                        code.push_str(&format!("    mov [rbp - {}], rax\n", target_offset));
                        
                        code.push_str(&format!("{}:\n", start_label));
                        
                        // Bitiş değerini hesapla
                        code.push_str(&self.generate_expr(end)?);
                        code.push_str("    push rax\n"); 
                        code.push_str(&format!("    mov rax, [rbp - {}]\n", target_offset));
                        code.push_str("    pop rbx\n");
                        code.push_str("    cmp rax, rbx\n");
                        code.push_str(&format!("    jge {}\n", end_label)); 
                        
                        code.push_str(&self.generate_stmt(body)?);
                        
                        code.push_str(&format!("    mov rax, [rbp - {}]\n", target_offset));
                        code.push_str("    inc rax\n");
                        code.push_str(&format!("    mov [rbp - {}], rax\n", target_offset));
                        code.push_str(&format!("    jmp {}\n", start_label));

                        self.type_checker.pop_scope()?;

                        if let Some(loc) = old_location {
                            self.variable_locations.insert(var_name.clone(), loc);
                        } else {
                            self.variable_locations.remove(var_name);
                        }
                    } else {
                        // FOREACH: Array Iteration
                        let iter_type = self.type_checker.type_of_expr(iter_expr).map_err(|e| format!("Döngü hatası: {e}"))?;
                        
                        // Type::Arr veya Type::Array ile çalış
                        let (elem_type, len) = match iter_type {
                            Type::Array(inner, len_opt) => (*inner, len_opt.unwrap_or(0)),
                            Type::Arr => {
                                // Type::Arr için: variable_locations'dan boyut bilgisini al
                                if let Expr::Variable(arr_var_name) = iter_expr {
                                    let var_loc = self.variable_locations.get(arr_var_name)
                                        .ok_or_else(|| format!("Dizi değişkeni bulunamadı: {arr_var_name}"))?;
                                    
                                    // array_len metadata'sını kullan
                                    let len = var_loc.array_len.ok_or_else(|| 
                                        format!("Type::Arr değişkeni '{}' için boyut bilgisi yok. Dinamik diziler için arrlen kullanın.", arr_var_name)
                                    )?;
                                    
                                    // Type::Arr heterogeneous olduğu için elem_type = Type::Any
                                    (Type::Any, len)
                                } else {
                                    return Err("Type::Arr sadece değişkenler için kullanılabilir.".to_string());
                                }
                            },
                            _ => return Err(format!("For in döngüsü {:?} tipi üzerinde çalışmaz. Sadece dizi veya range.", iter_type))
                        };
                        
                        // 1. Array başlangıç adresini al
                        let array_base_loc = if let Expr::Variable(arr_var_name) = iter_expr {
                            self.variable_locations.get(arr_var_name).ok_or_else(|| format!("Dizi bulunamadı: {arr_var_name}"))?.clone()
                        } else {
                            return Err("For in şimdilik sadece dizi değişkenleri üzerinde çalışıyor.".to_string());
                        };

                        let start_label = self.generate_label("L_for_arr_start");
                        let end_label = self.generate_label("L_for_arr_end");
                        self.loop_labels.push((start_label.clone(), end_label.clone()));

                        // Gizli indeks değişkeni (idx)
                        self.stack_pointer += 8;
                        let idx_offset = self.stack_pointer;
                        
                        // idx = 0
                        code.push_str("    xor rax, rax\n");
                        code.push_str(&format!("    mov [rbp - {}], rax\n", idx_offset));

                        self.type_checker.push_scope();

                        // Loop değişkeni (x) için alan
                        self.stack_pointer += 8;
                        let loop_var_offset = self.stack_pointer;
                        self.variable_locations.insert(var_name.clone(), VariableLocation { stack_offset: loop_var_offset, ty: elem_type.clone(), array_len: None });
                        self.type_checker.define_variable(var_name.clone(), crate::type_checker::VarInfo{ ty: elem_type.clone(), is_const: false, _is_mutable: false })?;

                        // LABEL START
                        code.push_str(&format!("{}:\n", start_label));

                        // 2. Koşul: idx < len
                        code.push_str(&format!("    mov rax, [rbp - {}]\n", idx_offset));
                        code.push_str(&format!("    cmp rax, {}\n", len));
                        code.push_str(&format!("    jge {}\n", end_label));

                        // 3. Elemanı Yükle: x = arr[idx]
                        code.push_str(&format!("    mov rax, [rbp - {}]\n", idx_offset)); // RAX = idx
                        
                        let base_off = array_base_loc.stack_offset;
                        
                        if elem_type.is_float() {
                            // Float Load
                            code.push_str(&format!("    movsd xmm0, [rbp - {} + rax*8]\n", base_off));
                            code.push_str(&format!("    movsd [rbp - {}], xmm0\n", loop_var_offset));
                        } else {
                            // Int Load (Type::Arr heterogeneous olduğu için her elemanı int gibi işle)
                            code.push_str(&format!("    mov rax, [rbp - {} + rax*8]\n", base_off));
                            code.push_str(&format!("    mov [rbp - {}], rax\n", loop_var_offset));
                        }

                        // 4. Body
                        code.push_str(&self.generate_stmt(body)?);

                        // 5. Increment idx
                        code.push_str(&format!("    mov rax, [rbp - {}]\n", idx_offset));
                        code.push_str("    inc rax\n");
                        code.push_str(&format!("    mov [rbp - {}], rax\n", idx_offset));
                        code.push_str(&format!("    jmp {}\n", start_label));

                        // LABEL END
                        code.push_str(&format!("{}:\n", end_label));

                        self.type_checker.pop_scope()?;
                        self.loop_labels.pop();

                        // Cleanup Stack (idx + loop_var)
                        self.variable_locations.remove(var_name);
                        self.stack_pointer -= 16; 
                    }
                } else if let Some(init) = initializer {
                    // Nim Style for: for (i=0, i<10, i++)
                    // Eğer başlatıcı sadece bir değişken ismiyse (i), onu 0'a init edelim.
                    if let Stmt::ExprStmt(Expr::Variable(name)) = init.as_ref() {
                        let loc = self.variable_locations.get(name).ok_or_else(|| format!("Döngü değişkeni bulunamadı: {name}"))?;
                        code.push_str("    xor rax, rax\n");
                        code.push_str(&format!("    mov [rbp - {}], rax # Varsayılan 0 ilklendirmesi\n", loc.stack_offset));
                    } else {
                        code.push_str(&self.generate_stmt(init)?);
                    }
                    
                    code.push_str(&format!("{}:\n", start_label));
                    
                    if let Some(cond) = condition {
                        code.push_str(&self.generate_expr(cond)?);
                        code.push_str("    test rax, rax\n");
                        code.push_str(&format!("    jz {}\n", end_label));
                    }
                    
                    code.push_str(&self.generate_stmt(body)?);
                    
                    if let Some(inc) = increment {
                        code.push_str(&self.generate_expr(inc)?);
                    }
                    
                    code.push_str(&format!("    jmp {}\n", start_label));
                }

                code.push_str(&format!("{}:\n", end_label));
                Ok(code)
            }
            Stmt::ExprStmt(expr) => self.generate_expr(expr),
            Stmt::Loop { body } => {
                let start_label = self.generate_label("L_loop_start");
                let mut code = String::new();
                code.push_str(&format!("{}:\n", start_label));
                code.push_str(&self.generate_stmt(body)?);
                code.push_str(&format!("    jmp {}\n", start_label));
                Ok(code)
            }
            Stmt::Empty => Ok("".to_string()),
            _ => Ok("".to_string()),
        }
    }

    // YARDIMCI FONKSİYON: Benzersiz bir etiket oluşturur.
    fn generate_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }

    // Echo çağrısı üretir.
    fn generate_echo_call(&mut self, expr: &Expr) -> Result<String, String> {
        self.generate_print_op(expr, None, false)
    }

    // Genel print operasyonu (print, println, echo için)
    fn generate_print_op(&mut self, expr: &Expr, style_expr: Option<&Expr>, newline: bool) -> Result<String, String> {
        let mut code = String::new();
        
        let expr_type = self.type_checker.type_of_expr(expr).map_err(|e| format!("Print hatası: {e}"))?;
        let format_spec = self.get_format_specifier(&expr_type).to_string();
        
        // Stil Kodlarını Belirle (ANSI)
        let mut prefix = String::new();
        let mut suffix = String::new();
        
        if let Some(style_e) = style_expr {
             if let Expr::Literal(LiteralValue::Str(style_name)) = style_e {
                 let style_str = style_name.as_str();
                 
                 // 1. Önce kullanıcı tanımlı stillere bak
                 if let Some(custom_code) = self.type_checker.styles.get(style_str) {
                     prefix = custom_code.clone();
                     suffix = "\x1b[0m".to_string();
                 } 
                 // 2. Doğrudan ANSI kodu mu? (\x1b ile başlıyorsa)
                 else if style_str.starts_with("\x1b") {
                     prefix = style_str.to_string();
                     suffix = "\x1b[0m".to_string();
                 }
                 // 3. Yerleşik stilleri kontrol et
                 else {
                     match style_str {
                         "error" => { prefix = "\x1b[31m".to_string(); suffix = "\x1b[0m".to_string(); },
                         "warn"  => { prefix = "\x1b[33m".to_string(); suffix = "\x1b[0m".to_string(); },
                         "info"  => { prefix = "\x1b[36m".to_string(); suffix = "\x1b[0m".to_string(); },
                         "success" => { prefix = "\x1b[32m".to_string(); suffix = "\x1b[0m".to_string(); },
                         _ => {}
                     }
                 }
             }
        }

        let line_end = if newline { "\r\n" } else { "" };
        
        match expr {
            Expr::InterpolatedString(parts) => {
                let mut full_format = prefix;
                let mut args_to_pass = Vec::new();
                for part in parts {
                    match part {
                        Expr::Literal(LiteralValue::Str(s)) => full_format.push_str(s),
                        _ => {
                            let p_ty = self.type_checker.type_of_expr(part).unwrap_or(Type::Str(None));
                            full_format.push_str(self.get_format_specifier(&p_ty));
                            args_to_pass.push((p_ty, self.generate_expr(part)?));
                        }
                    }
                }
                full_format.push_str(&suffix);
                full_format.push_str(line_end);
                let fmt_idx = self.add_string_literal(full_format);
                code.push_str(&self.generate_printf_call_multi_arg(fmt_idx, args_to_pass)?);
            }
            _ => {
                code.push_str(&self.generate_expr(expr)?);
                let mut args = Vec::new();
                let load_code = if expr_type.is_float() {
                    // Float değeri (XMM0) stringe çevir (_ftoa)
                    // _print/_printf %s bekliyor olacak (aşağıda format çevrilecek)
                    let mut c = String::new();
                    c.push_str("    movq xmm0, xmm0\n"); // (Gereksiz ama açıklayıcı: değer xmm0'da)
                    c.push_str("    call _ftoa\n");      // RAX <- String pointer
                    // Şimdi RAX'ta string pointer var.
                    // printf çağrısı için standart integer/pointer argümanı gibi davranacağız.
                    c
                } else {
                    "    # RAX'ta\n".to_string()
                };
                
                // Float ise format %s olmalı, çünkü stringe çevirdik.
                let effective_format = if expr_type.is_float() { "%s" } else { format_spec.as_str() };
                
                // args listesine push ederken tipini de Str olarak kandırabiliriz veya ignore edilir.
                // printf üreticisi sadece load_code'u kullanıyor.
                args.push((expr_type, load_code));
                
                let final_format = format!("{}{}{}{}", prefix, effective_format, suffix, line_end);
                let format_str_index = self.add_string_literal(final_format);
                code.push_str(&self.generate_printf_call_multi_arg(format_str_index, args)?);
            }
        }
        Ok(code)
    }

    fn get_format_specifier(&self, ty: &crate::ast::Type) -> &'static str {
        match ty {
            Type::Str(_) => "%s",
            Type::Char => "%c",
            t if t.is_float() => "%f", // float sting olarak gösteriliyor o yüzden %s, ama bu kezde var olmayan döngüye giriyor.. özellikle echo içerisinde  işlem yapılırken.
            _ => "%d",
        }
    }

    // format_str_index: format string'in data_items listesindeki indeksi.
    // args_to_pass: (tip, değeri RAX/XMM0'a yükleyen assembly kodu) çiftlerinin vektörü.
    fn generate_printf_call_multi_arg(&mut self, format_str_index: usize, args_to_pass: Vec<(crate::ast::Type, String)>) -> Result<String, String> {
        let mut code = String::new();
        
        let arg_int_regs = ["rcx", "rdx", "r8", "r9"];
        let arg_float_regs = ["xmm0", "xmm1", "xmm2", "xmm3"];

        // 1. AŞAMA: Hesaplama ve Stack'e Doğru Register'dan Kaydetme
        let mut temp_stack_offsets = Vec::new();
        for (ty, arg_gen_code) in &args_to_pass {
            code.push_str(arg_gen_code); // Hesaplama kodunu bas (Sonuç RAX veya XMM0'da)
            
            self.stack_pointer += 8;
            let offset = self.stack_pointer;
            temp_stack_offsets.push(offset);

            if ty.is_float() {
                // Float ise XMM0'daki 64-bit değeri stack'e kopyala
                code.push_str(&format!("    movq qword ptr [rbp - {}], xmm0\n", offset));
            } else {
                // Int ise RAX'ı kopyala
                code.push_str(&format!("    mov qword ptr [rbp - {}], rax\n", offset));
            }
        }

        // 2. AŞAMA: Argümanları Yerleştirme
        // Not: 5. ve sonraki argümanlar stack'te olmalı (sağdan sola doğru pushlanmalı).
        let mut stack_args_pushed = 0;
        if args_to_pass.len() > 3 { // 1 format + 3 regs = 4 total
            for i in (3..args_to_pass.iter().len()).rev() {
                 let offset = temp_stack_offsets[i];
                 code.push_str(&format!("    mov rax, [rbp - {}]\n", offset));
                 code.push_str("    push rax\n");
                 stack_args_pushed += 1;
            }
        }

        // Format ve ilk 3 argümanı yükle
        code.push_str(&format!("    lea rcx, [str_{}]\n", format_str_index));
        for i in 0..std::cmp::min(args_to_pass.len(), 3) {
            let offset = temp_stack_offsets[i];
            let int_reg = arg_int_regs[i + 1];
            let float_reg = arg_float_regs[i + 1];
            let is_float = args_to_pass[i].0.is_float();

            if is_float {
                code.push_str(&format!("    movsd {}, [rbp - {}]\n", float_reg, offset));
                // Windows x64 ABI Varargs: Float argümanlar hem XMM hem de Int register'ına (shadow) kopyalanmalı.
                if self.target_platform == TargetPlatform::Windows {
                    code.push_str(&format!("    movq {}, {}\n", int_reg, float_reg));
                }
            } else {
                code.push_str(&format!("    mov {}, [rbp - {}]\n", int_reg, offset));
            }
        }

        // 3. ÇAĞRI (Windows x64 için düzeltilmiş)
        if self.target_platform == TargetPlatform::Windows {
            // Toplam gereken alan: 32 (Shadow) + Ekstra Argümanlar (stack_args_pushed * 8)
            // Bunu 16 byte hizasına uydurmalıyız. 
            let extra_space = stack_args_pushed * 8;
            let total_sub = 32 + extra_space;
            
            // Hizalama Kontrolü: Eğer total_sub 16'nın katı değilse, 8 byte daha ekle (Padding)
            let padding = if (total_sub + 8) % 16 != 0 { 0 } else { 8 }; // +8 call dönüş adresinden gelir
            let final_sub = total_sub + padding;

            code.push_str(&format!("    sub rsp, {}      # Shadow + Args + Alignment\n", final_sub));
            code.push_str("    call _print\n");
            code.push_str(&format!("    add rsp, {}      # Cleanup\n", final_sub));
        }

        self.stack_pointer -= (temp_stack_offsets.len() * 8) as i32;
        Ok(code)
    }
    

    fn generate_expr(&mut self, expr: &Expr) -> Result<String, String> {
        //eprintln!("DEBUG: Codegen: Generating expression: {:?}", expr);
        match expr {
            Expr::Literal(LiteralValue::Int(val)) => {
                Ok(format!("    mov rax, {}\n", val))
            }
            Expr::Literal(LiteralValue::Char(c)) => {
                Ok(format!("    mov rax, {}\n", *c as u32))
            }
            Expr::Literal(LiteralValue::Float(val)) => { // f32, f64, f128 için
                // Kayan noktalı literali bellekte bir yere koyup oradan XMM0'a yükle.
                // Tüm float tiplerini şimdilik f64 olarak işliyoruz.
                // Değeri data_items'a ekle ve indeksini al.
                let float_index = self.add_data_item(DataItem::Float64(*val));
                // Etiketi indekse göre oluştur ve değeri yükle.
                Ok(format!("    movsd xmm0, [float_{}]\n", float_index)) // burada etiketleri float_float_index olarak kaydediyoruz. Ama aşağıda name olarak ele alınıyor.!!
            }
            Expr::Literal(LiteralValue::Str(s)) => {
                let str_index = self.add_string_literal(s.clone());
                Ok(format!("    lea rax, [str_{}]\n", str_index))
            }
            Expr::Literal(LiteralValue::Bool(b)) => {
                let val = if *b { 1 } else { 0 };
                Ok(format!("    mov rax, {}\n", val))
            }
            Expr::Variable(name) => {
                //eprintln!("DEBUG: Codegen: Looking up variable '{}'", name);
                if let Some(loc) = self.variable_locations.get(name) {
                    //eprintln!("DEBUG: Codegen: Found variable '{}' at offset {}", name, loc.stack_offset);
                    if loc.ty.is_float() {
                        Ok(format!("    movsd xmm0, [rbp - {}] # Load float variable '{}'\n", loc.stack_offset, name))
                    } else {
                        Ok(format!("    mov rax, [rbp - {}] # Load integer/pointer variable '{}'\n", loc.stack_offset, name))
                    }
                } else {
                    //eprintln!("DEBUG: Codegen: Variable '{}' NOT FOUND in variable_locations. Current map: {:?}", name, self.variable_locations);
                    Err(format!("Kod üretimi hatası: Bilinmeyen değişken '{}'", name))
                }
            }
            Expr::Input(prompt_opt) => {
                let mut input_code = String::new();

                // 1. Prompt (Mesaj) varsa değerlendir ve RCX'e yükle
                if let Some(prompt_expr) = prompt_opt {
                    // prompt_expr bir Box<Expr> olduğu için generate_expr'e referansını gönderiyoruz
                    input_code.push_str(&self.generate_expr(prompt_expr)?);
                    input_code.push_str("    mov rcx, rax     # Prompt adresini RCX'e al\n");
                } else {
                    // Prompt yoksa RCX = 0 (NULL)
                    input_code.push_str("    xor rcx, rcx     # Prompt yok\n");
                }

                // 2. Windows x64 ABI Shadow Space (32 byte)
                // Dış fonksiyon çağrılmadan önce stack hizalaması ve gölge alan
                input_code.push_str("    sub rsp, 32\n");
                input_code.push_str("    call _input\n");
                input_code.push_str("    add rsp, 32\n");

                // Sonuç zaten _input'tan RAX register'ında döner.
                Ok(input_code)
            },
            Expr::Binary { left, op, right } => {
                // TypeChecker'ı Codegen'in mevcut durumuyla senkronize et.
                // doğru değişkenlerle doldurulmasını sağlar.
                self.type_checker.push_scope(); // Yeni bir kapsam aç
                for (name, loc) in &self.variable_locations {
                    let var_info = crate::type_checker::VarInfo {
                        ty: loc.ty.clone(),
                        is_const: false, // Bu aşamada const/mut bilgisi kritik değil
                        _is_mutable: true,
                    };
                    self.type_checker.define_variable(name.clone(), var_info).unwrap(); // Hata beklemiyoruz
                }

                let left_type = self.type_checker.type_of_expr(left).map_err(|e| format!("Kod üretimi hatası: {}", e))?;
                let right_type = self.type_checker.type_of_expr(right).map_err(|e| format!("Kod üretimi hatası: {}", e))?;

                let mut code = String::new();

                // Eğer operasyon kayan noktalı ise
                if left_type.is_float() || right_type.is_float() { // f32, f64, f80, f128
                    // Kayan noktalı sayı aritmetiği (XMM register'ları kullanılır)
                    // 1. Sağ tarafı değerlendir ve stack'e sakla (XMM registerlarını korumak için)
                    
                    code.push_str(&self.generate_expr(right)?);
                    code.push_str("    sub rsp, 8\n    movsd [rsp], xmm0\n"); 
                    //

                    // 2. Sol tarafı değerlendir.
                    code.push_str(&self.generate_expr(left)?);
                    code.push_str("    movsd xmm1, [rsp]\n    add rsp, 8\n");

                    // 2. ve 1. argümanlar yer değiştirdi (sol XMM0, sağ XMM1)
                    match op {
                        BinOp::Add => code.push_str("    addsd xmm0, xmm1\n"),
                        BinOp::Sub => code.push_str("    subsd xmm0, xmm1\n"),
                        BinOp::Mul => code.push_str("    mulsd xmm0, xmm1\n"),
                        BinOp::Div => code.push_str("    divsd xmm0, xmm1\n"),
                        // YENİ: Mod operatörü desteği
                        BinOp::Mod => {
                            // Windows x64 ABI gereği fonksiyon çağrısından önce stack hizalama ve shadow space
                            code.push_str("    sub rsp, 32\n"); 
                            code.push_str("    call _fmod\n");
                            code.push_str("    add rsp, 32\n");
                        },
                        _ => return Err(format!("Desteklenmeyen ikili operatör (float): {:?}. Sadece +, -, *, / desteklenir.", op)),
                    }
                    self.type_checker.pop_scope()?;
                    return Ok(code);
                } else if left_type.is_integer() || right_type.is_integer() {
                    // Tamsayı aritmetiği (RAX, RBX register'ları kullanılır)
                    // 1. Sağ tarafı değerlendir ve stack'e push'la.
                    code.push_str(&self.generate_expr(right)?);
                    code.push_str("    push rax\n");

                    // 2. Sol tarafı değerlendir. Sonuç RAX'ta.
                    code.push_str(&self.generate_expr(left)?);
                    // 3. Sağ tarafı stack'ten RBX'e pop'la.
                    code.push_str("    pop rbx\n");

                    // 4. İşlemi yap.
                    match op {
                        BinOp::Add => code.push_str("    add rax, rbx\n"),
                        BinOp::Sub => code.push_str("    sub rax, rbx\n"),
                        BinOp::Mul => code.push_str("    imul rbx # Signed multiplication\n"), 
                        BinOp::Div => {
                            code.push_str("    cqo             # Sign-extend RAX to RDX:RAX\n");
                            code.push_str("    idiv rbx        # Signed division\n");
                        }
                        BinOp::Mod => {
                            // Mevcut Tam Sayı mod
                            code.push_str("    cqo\n");
                            code.push_str("    idiv rbx\n");
                            code.push_str("    mov rax, rdx # Remainder is in RDX\n");
                        }
                        // Karşılaştırma Operatörleri
                        BinOp::Equal | BinOp::Eq => {
                            code.push_str("    cmp rax, rbx\n");
                            code.push_str("    sete al\n");
                            code.push_str("    movzx rax, al\n");
                        }
                        BinOp::NotEqual | BinOp::Ne => {
                            code.push_str("    cmp rax, rbx\n");
                            code.push_str("    setne al\n");
                            code.push_str("    movzx rax, al\n");
                        }
                        BinOp::Less | BinOp::Lt => {
                            code.push_str("    cmp rax, rbx\n");
                            code.push_str("    setl al\n");
                            code.push_str("    movzx rax, al\n");
                        }
                        BinOp::Greater | BinOp::Gt => {
                            code.push_str("    cmp rax, rbx\n");
                            code.push_str("    setg al\n");
                            code.push_str("    movzx rax, al\n");
                        }
                        BinOp::LessEqual | BinOp::Le => {
                            code.push_str("    cmp rax, rbx\n");
                            code.push_str("    setle al\n");
                            code.push_str("    movzx rax, al\n");
                        }
                        BinOp::GreaterEqual | BinOp::Ge => {
                            code.push_str("    cmp rax, rbx\n");
                            code.push_str("    setge al\n");
                            code.push_str("    movzx rax, al\n");
                        }
                        // Mantıksal VE (Short-circuiting)
                        BinOp::And => {
                            let false_label = self.generate_label("L_and_false");
                            let end_label = self.generate_label("L_and_end");
                            let mut and_code = String::new();
                            
                            // Sol tarafı değerlendir
                            and_code.push_str(&self.generate_expr(left)?);
                            and_code.push_str("    test rax, rax\n");
                            and_code.push_str(&format!("    jz {}\n", false_label));
                            
                            // Sağ tarafı değerlendir
                            and_code.push_str(&self.generate_expr(right)?);
                            and_code.push_str("    test rax, rax\n");
                            and_code.push_str(&format!("    jz {}\n", false_label));
                            
                            and_code.push_str("    mov rax, 1\n");
                            and_code.push_str(&format!("    jmp {}\n", end_label));
                            
                            and_code.push_str(&format!("{}:\n", false_label));
                            and_code.push_str("    xor rax, rax\n");
                            and_code.push_str(&format!("{}:\n", end_label));
                            
                            self.type_checker.pop_scope()?;
                            return Ok(and_code); 
                        }
                        // Mantıksal VEYA (Short-circuiting)
                        BinOp::Or => {
                            let true_label = self.generate_label("L_or_true");
                            let end_label = self.generate_label("L_or_end");
                            let mut or_code = String::new();
                            
                            // Sol tarafı değerlendir
                            or_code.push_str(&self.generate_expr(left)?);
                            or_code.push_str("    test rax, rax\n");
                            or_code.push_str(&format!("    jnz {}\n", true_label));
                            
                            // Sağ tarafı değerlendir
                            or_code.push_str(&self.generate_expr(right)?);
                            or_code.push_str("    test rax, rax\n");
                            or_code.push_str(&format!("    jnz {}\n", true_label));
                            
                            or_code.push_str("    xor rax, rax\n");
                            or_code.push_str(&format!("    jmp {}\n", end_label));
                            
                            or_code.push_str(&format!("{}:\n", true_label));
                            or_code.push_str("    mov rax, 1\n");
                            or_code.push_str(&format!("{}:\n", end_label));
                            
                            self.type_checker.pop_scope()?;
                            return Ok(or_code);
                        }
                        _ => return Err(format!("Desteklenmeyen ikili operatör (int): {:?}", op)),
                    }
                } else {
                    return Err(format!("Desteklenmeyen ikili operatör tipleri: {:?} ve {:?}.", left_type, right_type));
                }

                // Senkronizasyon için açılan kapsamı temizle.
                self.type_checker.pop_scope()?;
                Ok(code)
            }
            
            Expr::Call { callee, args } => {
                let mut code = String::new();
                
                if let Expr::Variable(fn_name) = &**callee {
                    match fn_name.as_str() {
                        "print" => {
                            if args.len() < 1 { return Err("print en az 1 argüman bekler.".to_string()); }
                            let msg_expr = &args[0].1;
                            let style_expr = if args.len() > 1 { Some(&args[1].1) } else { None };
                            return self.generate_print_op(msg_expr, style_expr, false);
                        }
                        "println" => {
                            if args.len() < 1 { return Err("println en az 1 argüman bekler.".to_string()); }
                            let msg_expr = &args[0].1;
                            let style_expr = if args.len() > 1 { Some(&args[1].1) } else { None };
                            return self.generate_print_op(msg_expr, style_expr, true);
                        }
                        "eprint" => {
                            if args.len() < 1 { return Err("eprint en az 1 argüman bekler.".to_string()); }
                            return self.generate_print_op(&args[0].1, Some(&Expr::Literal(LiteralValue::Str("error".to_string()))), true);
                        }
                        _ => {}
                    }
                }

                // Normal Fonksiyonlar veya diğer Builtinler için mevcut mantık:
                // 1. Argümanları değerlendir ve geçici olarak stack'e sakla
                let mut temp_offsets = Vec::new();
                for (_, arg_expr) in args {
                    code.push_str(&self.generate_expr(arg_expr)?);
                    self.stack_pointer += 8;
                    let offset = self.stack_pointer;
                    temp_offsets.push(offset);
                    code.push_str(&format!("    mov [rbp - {}], rax\n", offset));
                }

                // 2. Register ve Stack argümanlarını hazırla
                let arg_regs = ["rcx", "rdx", "r8", "r9"];
                let mut stack_pushed_count = 0;
                
                // 5+ argümanlar stack'e (sağdan sola)
                if args.len() > 4 {
                    for i in (4..args.len()).rev() {
                        let offset = temp_offsets[i];
                        code.push_str(&format!("    mov rax, [rbp - {}]\n", offset));
                        code.push_str("    push rax\n");
                        stack_pushed_count += 1;
                    }
                }

                // İlk 4 argüman registerlara
                for i in 0..std::cmp::min(args.len(), 4) {
                    let offset = temp_offsets[i];
                    code.push_str(&format!("    mov {}, [rbp - {}]\n", arg_regs[i], offset));
                }

                // 3. Shadow Space
                code.push_str("    sub rsp, 32\n");
                
                // 4. Call
                if let Expr::Variable(fn_name) = &**callee {
                    match fn_name.as_str() {
                        "strlen" => {
                            if args.len() != 1 { return Err("strlen için 1 argüman bekleniyor.".to_string()); }
                            code.push_str("    call _strlen\n");
                        }
                        "exit" => {
                            if args.len() != 1 { return Err("exit için 1 argüman bekleniyor.".to_string()); }
                            code.push_str("    call ExitProcess\n");
                        }
                        "panic" => {
                            if args.len() != 1 { return Err("panic için 1 argüman bekleniyor.".to_string()); }
                            code.push_str("    call _print\n");
                            code.push_str("    mov rcx, 1\n    call ExitProcess\n");
                        }
                        "print" | "println" | "eprint" => unreachable!(), // Yukarıda halledildi
                        "_int" => {
                            if args.len() != 1 { return Err("_int için 1 argüman bekleniyor.".to_string()); }
                            let arg_ty = self.type_checker.type_of_expr(&args[0].1).map_err(|e| format!("_int hatası: {e}"))?;
                            // Argüman zaten değerlendirildi ve register'da (RAX veya XMM0)
                            if arg_ty.is_float() {
                                code.push_str("    cvttsd2si rax, xmm0 # Float to Int\n");
                            } else if arg_ty == Type::Char {
                                // RAX zaten char değerini tutuyor.
                            } else if arg_ty == Type::Str(None) {
                                code.push_str("    mov rcx, rax # String pointer\n");
                                code.push_str("    call _atoi\n");
                            } else if arg_ty.is_integer() {
                                // Değişiklik gerekmez.
                            } else {
                                return Err(format!("_int: {:?} tipi desteklenmiyor.", arg_ty));
                            }
                            // Cleanup call stack (shadow space vs) argüman değerlendirilirken yapılmamıştı, 
                            // normal function call mantığından çıkıp inline dönüşüm yaptık.
                            // Ancak biz Call şablonu içindeyiz. Bu yüzden özel dönüş yapmalıyız.
                            code.push_str(&format!("    add rsp, {}\n", 32 + stack_pushed_count * 8));
                            self.stack_pointer -= (args.len() * 8) as i32;
                            return Ok(code);
                        }
                        "_float" => {
                            if args.len() != 1 { return Err("_float için 1 argüman bekleniyor.".to_string()); }
                            let arg_ty = self.type_checker.type_of_expr(&args[0].1).map_err(|e| format!("_float hatası: {e}"))?;
                            if arg_ty.is_integer() || arg_ty == Type::Char {
                                code.push_str("    cvtsi2sd xmm0, rax # Int to Float\n");
                            } else if arg_ty.is_float() {
                                // Zaten XMM0'da.
                            } else {
                                return Err(format!("_float: {:?} tipi desteklenmiyor.", arg_ty));
                            }
                            code.push_str(&format!("    add rsp, {}\n", 32 + stack_pushed_count * 8));
                            self.stack_pointer -= (args.len() * 8) as i32;
                            return Ok(code);
                        }
                        "_str" => {
                            if args.len() != 1 { return Err("_str için 1 argüman bekleniyor.".to_string()); }
                            let arg_ty = self.type_checker.type_of_expr(&args[0].1).map_err(|e| format!("_str hatası: {e}"))?;
                            if arg_ty.is_float() {
                                code.push_str("    movq rdx, xmm0\n"); // Arg2: float bits
                                code.push_str("    call _ftoa\n");
                            } else {
                                code.push_str("    mov rcx, rax\n");
                                code.push_str("    call _itoa\n");
                            }
                            code.push_str(&format!("    add rsp, {}\n", 32 + stack_pushed_count * 8));
                            self.stack_pointer -= (args.len() * 8) as i32;
                            return Ok(code);
                        }
						"arrlen" => {
                             if args.len() != 1 { return Err("arrlen için 1 argüman bekleniyor.".to_string()); }
                             let arg_ty = self.type_checker.type_of_expr(&args[0].1).map_err(|e| format!("arrlen hatası: {e}"))?;
                             
                             if let Type::Array(_, Some(len)) = arg_ty {
                                 // Statik boyut biliniyor, direkt sabiti RAX'a yükle.
                                 code.push_str(&format!("    mov rax, {}\n", len));
                             } else if let Type::Array(_, None) = arg_ty {
                                 // Boyut bilinmiyor (Dinamik/Slice) - Şu an desteklenmiyor ama
                                 // Header'dan okuma eklenebilir.
                                 return Err("Dinamik boyutu bilinmeyen diziler için arrlen henüz desteklenmiyor.".to_string());
                             } else {
                                 return Err("arrlen sadece diziler için kullanılabilir.".to_string());
                             }
                             // Cleanup
                             code.push_str(&format!("    add rsp, {}\n", 32 + stack_pushed_count * 8));
                             self.stack_pointer -= (args.len() * 8) as i32;
                             return Ok(code);
                        }
                        _ => {
                            code.push_str(&format!("    call {}\n", fn_name));
                        }
                    }
                } else {
                    return Err("Sadece doğrudan fonksiyon isimleri ile çağrı destekleniyor.".to_string());
                }

                // 5. Cleanup
                code.push_str(&format!("    add rsp, {}\n", 32 + stack_pushed_count * 8));
                
                // Geçici stack imlecini geri al
                self.stack_pointer -= (args.len() * 8) as i32;
                
                Ok(code)
            }
            Expr::Unary { op, right } => {
                let mut code = String::new();
                code.push_str(&self.generate_expr(right)?);
                match op {
                    UnOp::Not => {
                        code.push_str("    test rax, rax\n");
                        code.push_str("    setz al\n");
                        code.push_str("    movzx rax, al\n");
                    }
                    UnOp::Neg => {
                        code.push_str("    neg rax\n");
                    }
                    UnOp::PostInc => {
                        if let Expr::Variable(name) = right.as_ref() {
                            let loc = self.variable_locations.get(name).ok_or_else(|| format!("Değişken bulunamadı: {name}"))?;
                            code.push_str(&format!("    mov rax, [rbp - {}]\n", loc.stack_offset));
                            code.push_str("    push rax\n");
                            code.push_str("    inc rax\n");
                            code.push_str(&format!("    mov [rbp - {}], rax\n", loc.stack_offset));
                            code.push_str("    pop rax\n");
                        } else {
                            return Err("Post-increment sadece değişkenlere uygulanabilir.".to_string());
                        }
                    }
                    UnOp::PostDec => {
                        if let Expr::Variable(name) = right.as_ref() {
                            let loc = self.variable_locations.get(name).ok_or_else(|| format!("Değişken bulunamadı: {name}"))?;
                            code.push_str(&format!("    mov rax, [rbp - {}]\n", loc.stack_offset));
                            code.push_str("    push rax\n");
                            code.push_str("    dec rax\n");
                            code.push_str(&format!("    mov [rbp - {}], rax\n", loc.stack_offset));
                            code.push_str("    pop rax\n");
                        } else {
                            return Err("Post-decrement sadece değişkenlere uygulanabilir.".to_string());
                        }
                    }
                    _ => return Err(format!("Desteklenmeyen tekli operatör: {:?}", op)),
                }
                Ok(code)
            }
            Expr::ArrayLiteral(elements) => {
                 let mut code = String::new();
                 let len = elements.len();
                 // Elemanlar için stack alanı ayır
                 self.stack_pointer += (len * 8) as i32;
                 let array_base_offset = self.stack_pointer;
                 
                 // array_base_offset şu an dizinin "son" (en düşük adresli - stack aşağı büyüyor) elemanını işaret ediyor olabilir mi?
                 // Sistemimizde: stack_pointer ofset olarak tutuluyor. Erişim [rbp - offset].
                 // SP arttıkça, [rbp - SP] adresi KÜÇÜLÜYOR.
                 // Bellekteki en düşük adres: RBP - array_base_offset.
                 // Bellekteki en yüksek adres: RBP - (array_base_offset - (len-1)*8).
                 // Array Layout: Low Addr -> High Addr = arr[0] -> arr[N].
                 // Yani arr[0] -> RBP - array_base_offset.
                 // arr[i] -> RBP - array_base_offset + i*8.
                 
                 for (i, elem) in elements.iter().enumerate() {
                     code.push_str(&self.generate_expr(elem)?);
                     
                     // Hesaplanan değeri (RAX/XMM0) dizi slotuna yaz
                     let target_addr = format!("rbp - {} + {}", array_base_offset, i * 8);
                     
                     let elem_ty = self.type_checker.type_of_expr(elem).unwrap_or(Type::I32);
                     if elem_ty.is_float() {
                         code.push_str(&format!("    movsd [{}], xmm0\n", target_addr));
                     } else {
                         code.push_str(&format!("    mov [{}], rax\n", target_addr));
                     }
                 }
                 
                 // Sonuç: Dizinin başlangıç adresi (arr[0]) RAX'ta döndürülmeli.
                 code.push_str(&format!("    lea rax, [rbp - {}]\n", array_base_offset));
                 Ok(code)
            }
            Expr::MemberAccess { object, member } => {
                let mut code = String::new();
                let obj_type = self.type_checker.type_of_expr(object).map_err(|e| format!("Üye erişim hatası: {e}"))?;
                
                if let Type::Custom(struct_name) = obj_type {
                    let offset = self.get_struct_member_offset(&struct_name, member)?;
                    let m_type = self.get_struct_member_type(&struct_name, member)?;
                    
                    // Eğer object bir değişkense, adresini al. Değerini değil.
                    if let Expr::Variable(name) = &**object {
                        let loc = self.variable_locations.get(name).ok_or_else(|| format!("Değişken bulunamadı: {name}"))?;
                        if m_type.is_float() {
                            code.push_str(&format!("    movsd xmm0, [rbp - {} + {}] # {}.{}\n", loc.stack_offset, offset, name, member));
                        } else {
                            code.push_str(&format!("    mov rax, [rbp - {} + {}] # {}.{}\n", loc.stack_offset, offset, name, member));
                        }
                    } else {
                        // Eğer object bir ifade ise (örn: get_struct().member)
                        // Önce adresi RAX'a yükle (bu kısım daha detaylanacak)
                        code.push_str(&self.generate_expr(object)?); 
                        if m_type.is_float() {
                            code.push_str(&format!("    movsd xmm0, [rax + {}]\n", offset));
                        } else {
                            code.push_str(&format!("    mov rax, [rax + {}]\n", offset));
                        }
                    }
                }
                Ok(code)
            }
            Expr::ArrayAccess { name, index } => {
                let mut code = String::new();
                let loc = self.variable_locations.get(name).cloned().ok_or_else(|| format!("Dizi bulunamadı: {name}"))?;
                
                // İndeksi RAX'a yükle
                code.push_str(&self.generate_expr(index)?);
                
                // Adres = Başlangıç + (İndeks * 8)
                if loc.ty.is_float() {
                     code.push_str(&format!("    movsd xmm0, [rbp - {} + rax*8]\n", loc.stack_offset));
                } else {
                     code.push_str(&format!("    mov rax, [rbp - {} + rax*8]\n", loc.stack_offset));
                }
                Ok(code)
            }
            Expr::Assign { left, value } => {
                let mut code = String::new();
                // 1. Sağ tarafı değerlendir (sonuç RAX veya XMM0)
                code.push_str(&self.generate_expr(value)?);
                
                // 2. Sol tarafın konumunu bul ve ata
                if let Expr::Variable(name) = &**left {
                    // Eğer for döngüsü başlatıcısında 'i' gibi bir değişken sadece adıyla geçiyorsa,
                    // ama henüz tanımlanmamışsa veya değer atanmamışsa 0'a init edelim.
                    // (Ancak Expr::Assign zaten bir atama olduğu için burada her zaman tanımlı olmalı)
                    let loc = self.variable_locations.get(name).ok_or_else(|| format!("Atama hatası: Bilinmeyen değişken '{}'", name))?;
                    if loc.ty.is_float() {
                        code.push_str(&format!("    movsd [rbp - {}], xmm0 # Assign to float variable '{}'\n", loc.stack_offset, name));
                    } else {
                        code.push_str(&format!("    mov [rbp - {}], rax # Assign to integer/pointer variable '{}'\n", loc.stack_offset, name));
                    }
                } else {
                    return Err("Şimdilik sadece değişkenlere atama destekleniyor.".to_string());
                }
                Ok(code)
            }
            _ => Err(format!("Bu ifade tipi için kod üretimi henüz desteklenmiyor: {:?}", expr)),
        }
    }

    // String literal'ini kaydeder ve indeksini döndürür
    fn add_string_literal(&mut self, s: String) -> usize {
        // Sadece stringleri kontrol et
        if let Some(pos) = self.data_items.iter().position(|item| matches!(item, DataItem::String(existing_s) if existing_s == &s)) {
            return pos;
        } else {
            self.data_items.push(DataItem::String(s));
            return self.data_items.len() - 1;
        }
    }

    // Genel bir veri öğesi ekler ve indeksini döndürür.
    fn add_data_item(&mut self, item: DataItem) -> usize {
        self.data_items.push(item);
        self.data_items.len() - 1
    }


    // Struct üyesinin ofsetini hesaplar (Her alan 8 byte varsayılıyor)
    fn get_struct_member_offset(&self, struct_name: &str, member_name: &str) -> Result<i32, String> {
        for decl in self.program {
            if let Decl::Struct { name, fields, .. } = decl {
                if name == struct_name {
                    let mut offset = 0;
                    for (f_name, _) in fields {
                        if f_name == member_name {
                            return Ok(offset);
                        }
                        offset += 8;
                    }
                    return Err(format!("Hata: '{}' struct'ında '{}' alanı bulunamadı.", struct_name, member_name));
                }
            }
        }
        Err(format!("Hata: '{}' struct'ı tanımlanmamış.", struct_name))
    }

    // Struct üyesinin tipini döndürür
    fn get_struct_member_type(&self, struct_name: &str, member_name: &str) -> Result<Type, String> {
        for decl in self.program {
            if let Decl::Struct { name, fields, .. } = decl {
                if name == struct_name {
                    for (f_name, f_ty) in fields {
                        if f_name == member_name {
                            return Ok(f_ty.clone());
                        }
                    }
                }
            }
        }
        Err(format!("Hata: '{}' üye tipi bulunamadı.", member_name))
    }

    // Dahili yardımcı rutinleri oluşturur
    fn generate_builtins_library(&self) -> String {
        let mut lib = String::new();
        lib.push_str("\n# --- Built-in Helpers ---\n");
        lib.push_str(".section .data\n");
        lib.push_str("_conv_buffer: .space 1024\n");
        lib.push_str("_fmt_float_str: .asciz \"%f\"\n");
        lib.push_str(".section .text\n");

        // Windows'ta kullanıcıdan veri almak için ReadFile veya scanf benzeri 
        // bir yapı kullanmalısın. Eğer core.s kullanıyorsan oraya bakmalıyız.
        lib.push_str("    # Burada ReadFile lojiği veya dış bir C fonksiyonu çağrısı olmalı\n");
        lib.push_str("    add rsp, 40\n");
        lib.push_str("    ret\n");

        // _atoi: rcx = string -> rax = integer
        lib.push_str("_atoi:\n");
        lib.push_str("    xor rax, rax\n    xor r8, r8\n    mov r9, 1\n");
        lib.push_str("    movzx r8, byte ptr [rcx]\n    cmp r8b, '-'\n    jne .Latoi_loop\n");
        lib.push_str("    mov r9, -1\n    inc rcx\n");
        lib.push_str(".Latoi_loop:\n    movzx r8, byte ptr [rcx]\n    test r8b, r8b\n    jz .Latoi_done\n");
        lib.push_str("    cmp r8b, '0'\n    jl .Latoi_done\n    cmp r8b, '9'\n    jg .Latoi_done\n");
        lib.push_str("    sub r8b, '0'\n    imul rax, 10\n    add rax, r8\n    inc rcx\n    jmp .Latoi_loop\n");
        lib.push_str(".Latoi_done:\n    imul rax, r9\n    ret\n\n");

        // _itoa: rcx = integer -> rax = string pointer (temporary buffer)
        lib.push_str("_itoa:\n");
        lib.push_str("    lea rax, [_conv_buffer]\n    add rax, 64\n    mov byte ptr [rax], 0\n");
        lib.push_str("    mov r8, rcx\n    mov r10, 10\n    mov r11, rax\n    test r8, r8\n    jns .Litoa_loop\n    neg r8\n");
        lib.push_str(".Litoa_loop:\n    xor rdx, rdx\n    mov rax, r8\n    div r10\n    mov r8, rax\n");
        lib.push_str("    add dl, '0'\n    dec r11\n    mov [r11], dl\n    test r8, r8\n    jnz .Litoa_loop\n");
        lib.push_str("    cmp rcx, 0\n    jge .Litoa_done\n    dec r11\n    mov byte ptr [r11], '-'\n");
        lib.push_str(".Litoa_done:\n    mov rax, r11\n    ret\n\n");
        
        
        // _ftoa: xmm0 = float -> rax = string pointer //dönüşüm problemli .
        // C standard library sprintf yerine _sprint kullanarak float dönüşümü
        lib.push_str("_ftoa:\n");
        lib.push_str("    sub rsp, 48\n"); // Shadow space (32) + alignment
        // _sprint(buffer, "%f", val)
        // RCX = buffer
        lib.push_str("    lea rcx, [_conv_buffer]\n");
        // RDX = format string
        lib.push_str("    lea rdx, [_fmt_float_str]\n");
        // R8/XMM2 = val (Windows x64 calling convention: 3. argüman XMM2/R8)
        lib.push_str("    movaps xmm2, xmm0\n"); 
        lib.push_str("    movq r8, xmm0\n"); // Shadowing
        lib.push_str("    call _sprint\n");
        lib.push_str("    lea rax, [_conv_buffer]\n");
        lib.push_str("    add rsp, 48\n");
        lib.push_str("    ret\n");

        lib
    }
}
