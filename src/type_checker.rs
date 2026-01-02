use std::collections::{HashMap, HashSet};
use std::fs;
use crate::ast::{Decl, Expr, LiteralValue, Stmt, Type, BinOp, UnOp, TargetPlatform}; //  TargetPlatform'u ast'den al.
use crate::{lexer::Lexer, parser::Parser};

// YENİ YAPI: Değişkenin tipini ve özelliklerini tutar
#[derive(Debug, Clone)]
pub struct VarInfo {
    pub ty: Type,
    pub is_const: bool,
    pub _is_mutable: bool,
}

//  Normal bir 'group' bloğunun içeriğini saklamak için.
#[derive(Debug, Clone, Default)]
pub struct GroupContent {
    // Grup içindeki fonksiyonlar: isim -> imza
    pub functions: HashMap<String, (Vec<(String, Type, bool)>, Type, bool, bool)>,
    // Grup içindeki sabitler: isim -> bilgi
    pub constants: HashMap<String, VarInfo>,
}
// Tip Kontrolcüsü Struct'ı
pub struct TypeChecker<'a> {
	// Fonksiyon imzalarını (parametre tipleri, dönüş tipi) sakla
	pub function_signatures: HashMap<String, (Vec<(String, Type, bool)>, Type, bool, bool)>, // (params, return_type, is_inline, is_public)
    //  Struct tanımlarını sakla: Struct Adı -> Alan Adı -> Alan Tipi
    pub struct_definitions: HashMap<String, HashMap<String, Type>>,
    //  Enum tanımlarını sakla: Enum Adı -> (Üye Adı -> Üye Tipi)
    pub enum_definitions: HashMap<String, HashMap<String, Type>>,
    //  Metot imzalarını sakla: Struct Adı -> Metot Adı -> (Parametreler, Dönüş Tipi)
    pub method_signatures: HashMap<String, HashMap<String, (Vec<(String, Type, bool)>, Type, bool)>>, // NEW: is_public eklendi
    //  Tip takma adlarını sakla: Takma Ad -> Gerçek Tip
    pub type_aliases: HashMap<String, Type>,
    //  Normal grup tanımlarını sakla: Grup Adı -> Grup İçeriği
    pub group_definitions: HashMap<String, GroupContent>,
    //  Modül takma adlarını sakla: Takma Ad -> Gerçek Modül Adı
    pub module_aliases: HashMap<String, String>,
    pub expected_return_type: Type,
    //  Hangi modüllerin zaten yüklendiğini takip et.
    pub loaded_modules: HashSet<String>,
    //  Modül arama yolları
    //  Modül arama yolları
    pub include_paths: Vec<String>,
    //  Kullanıcı tanımlı stiller: Stil Adı -> Stil Kodu (ANSI)
    pub styles: HashMap<String, String>,
    
    pub scopes: Vec<HashMap<String, VarInfo>>,
    //  Mevcut kontrol edilen fonksiyonun bilgilerini sakla.
    current_function_name: Option<String>,
    current_function_params: Vec<(String, Type, Option<Expr>)>,
    //  `fastexec` bloğu içinde olup olmadığımızı takip et.
    in_fastexec_block: bool,

    pub labels: Vec<HashSet<String>>,
    program: &'a [Decl], // Reference to the whole program AST
    target_platform: TargetPlatform, //  Hedef platformu sakla.
}

impl<'a> TypeChecker<'a> {
    pub fn new(program: &'a [Decl], include_paths: Vec<String>, target_platform: TargetPlatform) -> Self {
		let mut checker = TypeChecker {
			function_signatures: HashMap::new(), 
            enum_definitions: HashMap::new(),
            struct_definitions: HashMap::new(),
            method_signatures: HashMap::new(),
            type_aliases: HashMap::new(),
            group_definitions: HashMap::new(),
            module_aliases: HashMap::new(),
            loaded_modules: HashSet::new(),
            include_paths,
            styles: HashMap::new(),
            expected_return_type: Type::Void,
            current_function_name: None,
            current_function_params: Vec::new(),
            in_fastexec_block: false,
			scopes: Vec::new(),
            labels: Vec::new(),
            program,
            target_platform,
		};
		
		// Yerleşik fonksiyonları kaydet
        checker.function_signatures.insert("echo".to_string(),   (vec![("value".to_string(), Type::Any, false)], Type::Void, false, true));
        checker.function_signatures.insert("print".to_string(),  (vec![("message".to_string(), Type::Any, false), ("style".to_string(), Type::Str(None), true)], Type::Void, false, true));
        checker.function_signatures.insert("println".to_string(),(vec![("message".to_string(), Type::Any, false), ("style".to_string(), Type::Str(None), true)], Type::Void, false, true));
        checker.function_signatures.insert("eprint".to_string(), (vec![("message".to_string(), Type::Any, false)], Type::Void, false, true));
        checker.function_signatures.insert("input".to_string(),  (vec![("prompt".to_string(), Type::Str(None), false)], Type::Str(None), false, true));
        checker.function_signatures.insert("strlen".to_string(), (vec![("s".to_string(), Type::Str(None), false)], Type::I32, false, true));
        checker.function_signatures.insert("arrlen".to_string(), (vec![("arr".to_string(), Type::Array(Box::new(Type::Any), None), false)], Type::I32, false, true));
        checker.function_signatures.insert("panic".to_string(),  (vec![("message".to_string(), Type::Str(None), false)], Type::Never, false, true));
        checker.function_signatures.insert("exit".to_string(),   (vec![("code".to_string(), Type::I32, false)], Type::Never, false, true));
		//  Kanal oluşturma fonksiyonu
        checker.function_signatures.insert("make_channel".to_string(), (vec![("capacity".to_string(), Type::I32, true)], Type::Any, false, true));

        //  Komut satırı argüman fonksiyonları artık yerleşik ve global.
        checker.function_signatures.insert("args".to_string(), (vec![], Type::Array(Box::new(Type::Str(None)), None), false, true));
        checker.function_signatures.insert("arg_count".to_string(), (vec![], Type::I32, false, true));

        //  Tip dönüşüm fonksiyonları
        checker.function_signatures.insert("_int".to_string(),   (vec![("val".to_string(), Type::Any, false)], Type::I64, false, true));
        checker.function_signatures.insert("_str".to_string(),   (vec![("val".to_string(), Type::Any, false)], Type::Str(None), false, true));
        checker.function_signatures.insert("_float".to_string(), (vec![("val".to_string(), Type::Any, false)], Type::F64, false, true));

		checker.push_scope(); 

		checker
	}
	
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.labels.push(HashSet::new());
    }

    pub fn pop_scope(&mut self) -> Result<(), String> {
        if self.scopes.len() <= 1 {
            return Err("Global kapsam çıkarılamaz!".to_string());
        }
        self.scopes.pop();
        self.labels.pop();
        Ok(())
    }

    pub fn define_label(&mut self, name: String) -> Result<(), String> {
        let current_scope = self.labels.last_mut().expect("Etiket kapsam yığını boş olamaz.");
        if !current_scope.insert(name.clone()) {
            return Err(format!("Hata: '{}' etiketi bu kapsamda zaten tanımlı.", name));
        }
        Ok(())
    }

    pub fn get_label(&self, name: &str) -> bool {
        for scope in self.labels.iter().rev() {
            if scope.contains(name) {
                return true;
            }
        }
        false
    }


    pub fn define_variable(&mut self, name: String, info: VarInfo) -> Result<(), String> {
		let current_scope = self.scopes.last_mut().expect("Kapsam yığını boş olamaz.");

		if current_scope.contains_key(&name) {
			return Err(format!("Hata: '{}' değişkeni bu kapsamda zaten tanımlı.", name));
		}
		current_scope.insert(name, info);
		Ok(())
	}

    pub fn get_variable_info(&self, name: &str) -> Result<VarInfo, String> {
        // 1. Değişken olarak ara
		for scope in self.scopes.iter().rev() {
			if let Some(info) = scope.get(name) {
				return Ok(info.clone());
			}
		}

        // 2. Değişken değilse, fonksiyon olabilir mi?
        if let Some((params, ret_type, _, _)) = self.function_signatures.get(name) {
            let param_types = params.iter().map(|(_, ty, _)| ty.clone()).collect();
            let fn_type = Type::Fn(param_types, Box::new(ret_type.clone()));
            // Fonksiyonu bir "sabit" değişken gibi döndür.
            return Ok(VarInfo { ty: fn_type, is_const: true, _is_mutable: false });
        }

        // 3. Hiçbiri değilse hata ver.
        Err(format!("Hata: Tanımlanmamış isim: '{}'. Bu isim bir değişken, fonksiyon veya tip değil.", name))
    }
        
	pub fn check_program(&mut self) -> Result<(), String> {
    
		for decl in self.program {
			if let Decl::Function { name, params, return_type, is_async, is_inline, is_public, .. } = decl {
				// (param name, param type, has default)
				let param_info: Vec<(String, Type, bool)> = params.iter().map(|(n, t, d)| (n.clone(), t.clone(), d.is_some())).collect();
                //  Eğer fonksiyon 'async' ise, dönüş tipini Future<T> olarak sarmala.
                let final_return_type = if *is_async {
                    Type::Future(Box::new(return_type.clone()))
                } else {
                    return_type.clone()
                };
				self.function_signatures.insert(name.clone(), (param_info, final_return_type, *is_inline, *is_public));
			} else if let Decl::Struct { name, fields, .. } = decl {
                //  Struct tanımını kaydet
                if self.struct_definitions.contains_key(name) {
                    return Err(format!("Hata: '{}' struct'ı zaten tanımlanmış.", name));
                }
                let mut field_map = HashMap::new();
                for (field_name, field_type) in fields {
                    // TODO: Alan tiplerinin de geçerli olup olmadığını kontrol et.
                    field_map.insert(field_name.clone(), field_type.clone());
                }
                self.struct_definitions.insert(name.clone(), field_map);
			} else if let Decl::Enum { name, variants, .. } = decl {
                if self.enum_definitions.contains_key(name) {
                    return Err(format!("Hata: '{}' enum'u zaten tanımlanmış.", name));
                }
                let mut variant_map = HashMap::new();
                let mut enum_base_type = Type::I32; // Varsayılan tip
                for (variant_name, value_expr_opt) in variants {
                    if let Some(value_expr) = value_expr_opt {
                        // Değer atanmışsa, tipini kontrol et ve enum'un temel tipini belirle
                        let variant_type = self.type_of_expr(value_expr)?;
                        if !variant_type.is_integer() {
                            return Err(format!("Hata: '{}' enum üyesine sadece tamsayı değer atanabilir, bulundu: {:?}.", variant_name, variant_type));
                        }
                        enum_base_type = variant_type; // Enum'un tipini son atanan üyenin tipi olarak al
                    }
                    // Tüm üyeler aynı temel tipe sahip olmalı
                    variant_map.insert(variant_name.clone(), Type::Enum(name.clone(), Box::new(enum_base_type.clone())));
                }
                self.enum_definitions.insert(name.clone(), variant_map);

			} else if let Decl::Typedef { name, target, is_public: _ } = decl {
                if self.type_aliases.contains_key(name) {
                    return Err(format!("Hata: '{}' tip takma adı zaten tanımlanmış.", name));
                }
                self.type_aliases.insert(name.clone(), target.clone());
			} else if let Decl::Group { name, body, .. } = decl {
                //  Eğer group adı bir struct adıyla eşleşiyorsa, bunu bir metot bloğu olarak işle.
                if self.struct_definitions.contains_key(name) {
                    let mut new_methods = Vec::new();
                    for decl in body {
                        if let Decl::StmtDecl(stmt) = decl {
                            if let Stmt::LabeledStmt { label: method_name, stmt: method_body, is_public } = &**stmt {
                                if let Stmt::ExprStmt(expr) = &**method_body {
                                    if let Expr::Lambda { params, return_type, .. } = expr {
                                        if params.is_empty() || params[0].0 != "self" {
                                            return Err(format!("Hata: '{}' struct'ının '{}' metodu 'self' parametresi ile başlamalıdır.", name, method_name.clone()));
                                        }
                                        // 'self' parametresinin tipinin struct'ın kendisi olduğunu doğrula
                                        if self.resolve_type(&params[0].1)? != Type::Custom(name.clone()) {
                                            return Err(format!("Hata: '{}' metodunun 'self' parametresi '{}' tipinde olmalıdır, bulundu: {:?}.", method_name.clone(), name, &params[0].1));
                                        }
                                        let param_info: Vec<(String, Type, bool)> = params.iter().map(|(n, t, d)| (n.clone(), t.clone(), d.is_some())).collect();
                                        new_methods.push((method_name.clone(), (param_info, return_type.clone(), *is_public)));
                                    }
                                }
                            }
                        }
                    }
                    // Döngü bittikten sonra toplu ekleme yaparak borrow checker hatasını çöz.
                    let method_map = self.method_signatures.entry(name.clone()).or_insert_with(HashMap::new);
                    for (name, signature) in new_methods {
                        method_map.insert(name, signature);
                    }
                } else {
                    //  Normal 'group' bloklarını işle.
                    if self.group_definitions.contains_key(name) {
                        return Err(format!("Hata: '{}' grubu zaten tanımlanmış.", name));
                    }
                    let mut content = GroupContent::default();
                    for decl in body {
                        if let Decl::StmtDecl(stmt) = decl {
                            if let Stmt::LabeledStmt { label, stmt: inner_stmt, is_public } = &**stmt {
                                if let Stmt::ExprStmt(expr) = &**inner_stmt {
                                    if let Expr::Lambda { params, return_type, .. } = expr {
                                        // Grup içindeki bir fonksiyon
                                        let param_info: Vec<(String, Type, bool)> = params.iter().map(|(n, t, d)| (n.clone(), t.clone(), d.is_some())).collect();
                                        content.functions.insert(label.clone(), (param_info, return_type.clone(), false, *is_public));
                                    }
                                }
                            }
                        } else if let Decl::StmtDecl(stmt) = decl {
                            if let Stmt::VarDecl { name: var_name, ty, init: _, is_const, .. } = &**stmt {
                                if *is_const {
                                    // Grup içindeki bir sabit
                                    let info = VarInfo { ty: ty.clone(), is_const: true, _is_mutable: false };
                                    content.constants.insert(var_name.clone(), info);
                                } else {
                                    return Err(format!("Hata: '{}' grubu içinde sadece 'const' tanımlamalara izin verilir, 'var' veya 'let' kullanılamaz.", name));
                                }
                            }
                        }
                    }
                    self.group_definitions.insert(name.clone(), content);
                }
            } else if let Decl::ExternFn { name, params, return_type, is_public } = decl {
                let param_info: Vec<(String, Type, bool)> = params.iter().map(|(n, t, d)| (n.clone(), t.clone(), d.is_some())).collect();
                // Dış fonksiyonlar 'async' veya 'inline' olamaz.
                self.function_signatures.insert(name.clone(), (param_info, return_type.clone(), false, *is_public));

            } else if let Decl::Use { path, spec, is_export: _is_export } = decl {
                // `use my::module` için modül adı "module" olur.
                let module_path_str = path.join("/");

                match &spec { // spec'e referans alıyoruz çünkü taşımak istemiyoruz.
                    crate::ast::UseSpec::All(alias_opt) => {
                        if let Some(alias) = alias_opt {
                            // `use my_module as my;` durumu.
                            self.module_aliases.insert(alias.clone(), module_path_str.clone());
                            // Sadece takma ad oluşturulur, tüm öğeler global kapsama dahil edilmez.
                            continue;
                        }
                        self.import_all_from_module(&module_path_str)?;
                    }
                    crate::ast::UseSpec::Wildcard => {
                        self.import_all_from_module(&module_path_str)?;
                    }
                    crate::ast::UseSpec::Specific(item_list) => {
                        // `use my_module::{item1, item2};` durumu
                        for item in item_list.iter() {
                            match item {
                                crate::ast::UseSpecItem::Item(item_name) => self.import_item_from_module(&module_path_str, item_name, Some(item_name))?,
                                crate::ast::UseSpecItem::RenamedItem(original_name, alias) => self.import_item_from_module(&module_path_str, original_name, Some(alias))?,
                            }
                        }
                    }
                }
            }
		}
        //  Çıktıyı daha anlamlı hale getir. Sadece kullanıcı tanımlı ve içe aktarılan fonksiyonları listele.
        let built_in_functions: HashSet<_> = ["echo", "print", "input", "strlen", "arrlen", "panic", "exit", "make_channel"].iter().cloned().collect();
        let user_defined_functions: Vec<_> = self.function_signatures.keys()
            .filter(|&name| !built_in_functions.contains(name.as_str()))
            .collect();

        if !user_defined_functions.is_empty() {
		    println!("✅ Tanımlanan Global Fonksiyonlar: {:?}", user_defined_functions);
        }

        // Grupları ve grup fonksiyonlarını yazdırmaya devam et.
        if !self.group_definitions.is_empty() {
            println!("✅ Kaydedilen Gruplar ve Fonksiyonları:");
            for (group_name, content) in &self.group_definitions {
                println!("  - Grup '{}': {:?}", group_name, content.functions.keys());
            }
        }

		for decl in self.program.iter() {
			match decl {
				Decl::Function { name, params, return_type, body, is_async, .. } => {
					// 'async' bir fonksiyonun İÇİNDEKİ return'ler Future<T> değil, T döndürür.
                    //  Mevcut fonksiyon bilgilerini güncelle.
                    self.current_function_name = Some(name.clone());
                    self.current_function_params = params.clone();
					self.expected_return_type = return_type.clone();

					self.push_scope(); 
					
					for (param_name, param_type, _) in params {
						let info = VarInfo { 
							ty: param_type.clone(), 
							is_const: false, 
							_is_mutable: false 
						};
						self.define_variable(param_name.clone(), info)?;
					}

					if let Err(e) = self.check_stmt(body) {
						let _ = self.pop_scope(); 
						return Err(e);
					}
					
					if !is_async && self.expected_return_type != Type::Void && !self.body_has_return(body) {
						return Err(format!("Hata: '{}' fonksiyonu bir değer döndürmelidir, ancak bazı yollar 'return' ifadesi olmadan bitiyor.", name));
					}

					self.pop_scope()?;
                    //  Fonksiyon kontrolü bitti, bilgileri temizle.
                    self.current_function_name = None;
                    self.current_function_params.clear();
				},
                Decl::Group { body, .. } => {
                    // Bir 'group' bloğu, kendi başına bir fonksiyon gibi davranmaz,
                    // sadece bir kapsayıcıdır. Bu yüzden 'expected_return_type'ı
                    // değiştirmemeli ve yeni bir fonksiyon kapsamı açmamalıyız.
                    // Sadece içindeki deyimlerin geçerli olup olmadığını kontrol etmeliyiz.
                    self.push_scope();
                    for decl in body {
                        if let Decl::StmtDecl(stmt) = decl {
                            self.check_stmt(stmt)?;
                        } else {
                            // Handle other Decl types if they can appear in a group
                        }
                    }
                    self.pop_scope()?;
                }
                Decl::Style { name, code } => {
                    if self.styles.contains_key(name) {
                        return Err(format!("Hata: '{}' stili zaten tanımlı.", name));
                    }
                    self.styles.insert(name.clone(), code.clone());
                }
				Decl::StmtDecl(stmt) => {
					self.check_stmt(stmt)?;
				}
				_ => {}
			}
		}
        Ok(())
    }

    //  Modül yükleme mantığı
    fn load_module(&mut self, module_path: &str) -> Result<Decl, String> {
        if self.loaded_modules.contains(module_path) {
            // Modül zaten işlendi, ancak AST'sini yeniden ayrıştırmamız gerekebilir.
            // Şimdilik basit tutuyoruz ve tekrar ayrıştırıyoruz.
        }

        // Dosya adını oluştur (örn: "my/utils" -> "my/utils.nim")
        let file_name = format!("{}.nim", module_path);

        //  Tüm include yollarında modülü ara.
        let mut source: Option<String> = None;
        for path_prefix in &self.include_paths {
            let full_path = std::path::Path::new(path_prefix).join(&file_name);
            if let Ok(content) = fs::read_to_string(&full_path) {
                source = Some(content);
                println!("ℹ️ '{}' modülü yükleniyor...", full_path.display());
                break;
            }
        }

        let source = source.ok_or_else(|| {
            format!("Hata: '{}' modülü arama yollarında bulunamadı: {:?}", file_name, self.include_paths)
        })?;


        // Modülün kaynak kodunu ayrıştır
        let mut lexer = Lexer::new(&source);
        //  Token toplama mantığını, Eof'u da içerecek ve sonsuz döngüyü önleyecek şekilde düzelt.
        let mut tokens = Vec::new();
        loop {
            let token = lexer.next_token();
            let is_eof = token.kind == crate::token::TokenType::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        let mut parser = Parser::new(tokens);
        let (ast, errors) = parser.parse();

        if !errors.is_empty() {
            return Err(format!("'{}' modülü ayrıştırılırken hatalar oluştu:\n{}", file_name, errors.join("\n")));
        }

        self.loaded_modules.insert(module_path.to_string());
        Ok(ast)
    }

    //  Bir modüldeki tüm `pub` öğeleri içe aktarır.
    fn import_all_from_module(&mut self, module_path: &str) -> Result<(), String> {
        let ast = self.load_module(module_path)?;

        // AST'den `pub` öğeleri topla
        if let Decl::Program(declarations) = ast {
            for decl in declarations {
                if let Decl::Function { name, params, return_type, is_async, is_inline, is_public, .. } = decl {
                    if is_public {
                        let param_info: Vec<(String, Type, bool)> = params.iter().map(|(n, t, d)| (n.clone(), t.clone(), d.is_some())).collect();
                        let final_return_type = if is_async { Type::Future(Box::new(return_type.clone())) } else { return_type.clone() };
                        self.function_signatures.insert(name.clone(), (param_info, final_return_type, is_inline, is_public));
                    }
                } else if let Decl::Struct { name, fields, is_public } = decl {
                    if is_public {
                        if self.struct_definitions.contains_key(&name) {
                            println!("Uyarı: '{}' modülünden içe aktarılan '{}' struct'ı zaten tanımlı, üzerine yazılmıyor.", module_path, name);
                            continue;
                        }
                        let mut field_map = HashMap::new();
                        for (field_name, field_type) in fields {
                            field_map.insert(field_name.clone(), field_type.clone());
                        }
                        self.struct_definitions.insert(name.clone(), field_map);
                    }
                } else if let Decl::Enum { name, variants, is_public } = decl {
                    if is_public {
                        if self.enum_definitions.contains_key(&name) {
                            continue;
                        }
                        let mut variant_map = HashMap::new();
                        let enum_base_type = Type::I32; // Varsayılan
                        for (variant_name, _) in variants {
                            variant_map.insert(variant_name.clone(), Type::Enum(name.clone(), Box::new(enum_base_type.clone())));
                        }
                        self.enum_definitions.insert(name.clone(), variant_map);
                    }
                } else if let Decl::Typedef { name, target, is_public } = decl {
                    if is_public {
                        if self.type_aliases.contains_key(&name) {
                            continue;
                        }
                        self.type_aliases.insert(name.clone(), target.clone());
                    }
                } else if let Decl::StmtDecl(stmt) = decl {
                    //  `pub const` gibi üst düzey deyimleri işle.
                    if let Stmt::VarDecl { name, ty, is_const, is_let, is_public, .. } = &*stmt {
                         if *is_public {
                            if *is_const || *is_let {
                                // Sabitleri (`const`) ve değiştirilemez `let` değişkenlerini global kapsama ekle.
                                if self.scopes.is_empty() {
                                    return Err("İç Hata: Global kapsam bulunamadı.".to_string());
                                }
                                // `let` de değiştirilemez olduğu için `is_const: true` olarak işaretleyebiliriz.
                                let info = VarInfo { ty: ty.clone(), is_const: *is_const || *is_let, _is_mutable: false };
                                // Global kapsama (ilk kapsama) doğrudan ekle.
                                self.scopes[0].insert(name.clone(), info);
                            } else {
                                // `pub var` veya `pub let` henüz desteklenmiyor.
                            }
                        }
                    }
                } else if let Decl::Use { path, spec, is_export } = decl {
                    //  Eğer `export use ...` varsa, bu modülün öğelerini de içe aktar.
                    if is_export {
                        self.handle_use_declaration(&path, &spec, true)?;
                    }
                } else if let Decl::Group { name, is_export, body, .. } = decl {
                    //  `export group ...` bildirimini işle.
                    if is_export {
                        if self.group_definitions.contains_key(&name) {
                            println!("Uyarı: '{}' modülünden içe aktarılan '{}' grubu zaten tanımlı, üzerine yazılmıyor.", module_path, name);
                            continue;
                        }
                        let mut content = GroupContent::default();
                        // Grup gövdesindeki bildirimleri (Decl) işle.
                        for inner_decl in body {
                            match inner_decl {
                                Decl::StmtDecl(stmt) => { // This now matches correctly
                                    if let Stmt::VarDecl { name: var_name, ty, is_const, is_public, .. } = &*stmt {
                                        if *is_public && *is_const {
                                            let info = VarInfo { ty: ty.clone(), is_const: true, _is_mutable: false };
                                            content.constants.insert(var_name.clone(), info);
                                        }
                                    }
                                }
                                Decl::ExternFn { name: fn_name, params, return_type, is_public } => {
                                    if is_public {
                                        let param_info: Vec<(String, Type, bool)> = params.iter().map(|(n, t, d)| (n.clone(), t.clone(), d.is_some())).collect();
                                        content.functions.insert(fn_name.clone(), (param_info, return_type.clone(), false, is_public));
                                    }
                                },
                                _ => {} // Diğer bildirim türleri (örn: iç içe group) şimdilik yoksayılıyor.
                            }
                        }
                        self.group_definitions.insert(name.clone(), content);
                    }
                }
            }
        }
        Ok(())
    }

    //  Belirli bir öğeyi modülden içe aktarır.
    fn import_item_from_module(&mut self, module_path: &str, original_name: &str, final_name_opt: Option<&String>) -> Result<(), String> {
        let ast = self.load_module(module_path)?;
        let final_name = final_name_opt.unwrap_or(&original_name.to_string()).clone();

        if let Decl::Program(declarations) = ast {
            let mut found = false;
            for decl in declarations {
                match decl {
                    Decl::Function { name, params, return_type, is_async, is_inline, is_public, .. } if name == *original_name => {
                        if is_public {
                            let param_info = params.iter().map(|(n, t, d)| (n.clone(), t.clone(), d.is_some())).collect();
                            let final_return_type = if is_async { Type::Future(Box::new(return_type.clone())) } else { return_type.clone() };
                            self.function_signatures.insert(final_name, (param_info, final_return_type, is_inline, is_public));
                            found = true;
                            break;
                        }
                    }
                    Decl::Struct { name, fields, is_public } if name == *original_name => {
                        if is_public {
                            if self.struct_definitions.contains_key(&final_name) { continue; }
                            let mut field_map = HashMap::new();
                            for (field_name, field_type) in fields {
                                field_map.insert(field_name.clone(), field_type.clone());
                            }
                            self.struct_definitions.insert(final_name.clone(), field_map);
                            found = true;
                            break;
                        }
                    }
                    Decl::StmtDecl(stmt) => {
                        if let Stmt::VarDecl { name, ty, is_const, is_public, .. } = &*stmt {
                            if *is_public && *is_const && *name == *original_name {
                                let info = VarInfo { ty: ty.clone(), is_const: true, _is_mutable: false };
                                self.define_variable(final_name.clone(), info)?;
                                found = true;
                                break;
                            }
                        }
                    }
                    _ => {}
                }
            }
            if !found {
                return Err(format!("Hata: '{}' modülünde '{}' isminde dışa aktarılmış (public) bir öğe bulunamadı.", module_path, original_name));
            }
        }

        Ok(())
    }

    //  `use` ve `export use` bildirimlerini işleyen merkezi fonksiyon.
    fn handle_use_declaration(&mut self, path: &Vec<String>, spec: &crate::ast::UseSpec, is_reexport: bool) -> Result<(), String> {
        //  Platforma özel modül yükleme mantığı
        let mut processed_path = path.clone();
        if let Some(last_part) = processed_path.last_mut() {
            if *last_part == "platform" {
                *last_part = match self.target_platform {
                    TargetPlatform::Windows => "windows".to_string(),
                    TargetPlatform::Linux => "linux".to_string(),
                    TargetPlatform::Macos => "macos".to_string(),
                    TargetPlatform::Unknown => return Err("Hata: Platforma özel modül yüklemek için bir hedef platform (--target) belirtilmelidir.".to_string()),
                };
            }
        }
        let module_path_str = processed_path.join("/");

        match spec {
            crate::ast::UseSpec::All(alias_opt) => {
                if let Some(alias) = alias_opt {
                    if !is_reexport { // `export use ... as ...` anlamsız, sadece normal `use` için geçerli.
                        self.module_aliases.insert(alias.clone(), module_path_str.clone());
                    }
                    return Ok(());
                }
                self.import_all_from_module(&module_path_str)?;
            }
            crate::ast::UseSpec::Wildcard => {
                self.import_all_from_module(&module_path_str)?;
            }
            crate::ast::UseSpec::Specific(item_list) => {
                for item in item_list.iter() {
                    let (original_name, alias) = match item {
                        crate::ast::UseSpecItem::Item(name) => (name, Some(name)),
                        crate::ast::UseSpecItem::RenamedItem(orig, als) => (orig, Some(als)),
                    };
                    self.import_item_from_module(&module_path_str, original_name, alias)?;
                }
            }
        }
        Ok(())
    }

    fn body_has_return(&self, body: &Stmt) -> bool {
        if let Stmt::Block(stmts) = body {
            if let Some(last_stmt) = stmts.last() {
                if let Stmt::Return(_) = last_stmt {
                    return true;
                }
            }
        } else if let Stmt::Return(_) = body {
            return true;
        }
        false
    }


    fn check_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Block(stmts) => {
				self.push_scope();
				for stmt in stmts {
					self.check_stmt(stmt)?;
				}
				self.pop_scope()?;
				Ok(())
			},
			Stmt::Empty => Ok(()),
			Stmt::VarDecl { name, ty, init, is_const, is_let: _is_let, is_mutable, .. } => {
				if *is_const && init.is_none() {
					return Err(format!("Hata: Sabit (const) '{}' tanımlanırken bir başlangıç değeri atanmalıdır.", name));
				}
				if *is_const && *is_mutable {
					return Err(format!("Hata: '{}' hem sabit (const) hem de değiştirilebilir (mut) olarak tanımlanamaz.", name));
				}
                //  Değişkenin tipi bir struct, enum veya takma ad ise, geçerli olup olmadığını kontrol et.
                if let Type::Custom(type_name) = self.resolve_type(ty)? {
                    // Eğer bu bir enum ise, onu özel Enum tipine dönüştür.
                    if self.enum_definitions.contains_key(&type_name) {
                        // Bu satırın çalışması için `ty`'nin `mut` olması gerekir, bu yüzden bu mantığı aşağıya taşıyoruz.
                    } else if !self.struct_definitions.contains_key(&type_name) 
                           && !self.function_signatures.contains_key(&type_name)
                           && !self.type_aliases.contains_key(&type_name) {
                                return Err(format!("Hata: Bilinmeyen tip '{}' kullanıldı.", type_name));
                            }
                }
                if let Some(init_expr) = init {
                    let init_type = self.type_of_expr(init_expr)?;
                    //  Karşılaştırma yapmadan önce deklare edilen tipi çözümle.
                    // Bu, typedef'lerin (örn: UserID) temel tipleriyle (örn: u64) doğru şekilde karşılaştırılmasını sağlar.
                    let resolved_ty = self.resolve_type(ty)?;

                    // Decimal tiplere float atamasını kontrol et
                    let allow_decimal_float_assignment_var_decl = match (ty, &init_type) {
                        (Type::D32 | Type::D64 | Type::D128, Type::F32 | Type::F64 | Type::F80 | Type::F128) => true,
                        _ => false,
                    };

                    // Bit tipine integer literal atamasını kontrol et (0 veya 1)
                    let allow_bit_int_assignment_var_decl = match (ty, init_expr) {
                        (Type::Bit, Expr::Literal(LiteralValue::Int(val))) if *val == 0 || *val == 1 => true,
                        _ => false,
                    };

                    // Bit dizisine integer literal atamasını kontrol et
                    let allow_int_to_bit_array_assignment = match (ty, init_expr) {
                        (Type::Array(inner, _), Expr::Literal(LiteralValue::Int(_))) if **inner == Type::Bit => true,
                        _ => false,
                    };
                    
                    // I32 literalden Byte atamasına izin ver
                    let allow_byte_i32_assignment_var_decl = match (ty, init_expr) {
                        (Type::Byte, Expr::Literal(LiteralValue::Int(val))) if *val >= 0 && *val <= 255 => true,
                        _ => false,
                    };
                    
                    // I32 literalden Hex atamasına izin ver
                    let allow_hex_i32_assignment_var_decl = match (ty, init_expr) {
                        (Type::Hex, Expr::Literal(LiteralValue::Int(val))) if *val >= 0 && *val <= 255 => true,
                        _ => false,
                    };

                    if let (Type::Array(expected_inner_type, _), Type::ArrayLiteral(element_types)) = (&resolved_ty, &init_type) {
                        if !element_types.is_empty() {
                            let first_element_type = &element_types[0];
                            for element_type in element_types.iter().skip(1) {
                                if element_type != first_element_type {
                                    return Err(format!("Hata: Dizi başlatıcısındaki tüm elemanlar aynı tipte olmalıdır. Bulunan tipler: {:?}.", element_types));
                                }
                            }
                            if expected_inner_type.as_ref() != &Type::Unknown && expected_inner_type.as_ref() != first_element_type {
                                return Err(format!("Hata: '{}' dizisine atanmaya çalışılan eleman tipi ({:?}), beklenen tip ({:?}) ile uyuşmuyor.", name, first_element_type, expected_inner_type.as_ref()));
                            }
                        }
                    } else if let Type::Fn(param_types, ret_type) = &resolved_ty {
                        if let Expr::Variable(fn_name) = init_expr {
                            if let Some((expected_params, expected_ret, _, _)) = self.function_signatures.get(fn_name) {
                                let expected_param_types: Vec<_> = expected_params.iter().map(|(_, ty, _)| ty.clone()).collect();
                                if &expected_param_types != param_types || expected_ret != ret_type.as_ref() {
                                    return Err(format!("Hata: '{}' fonksiyonuna atanan fonksiyon imzası uyumsuz.", name));
                                }
                            } else {
                                return Err(format!("Hata: Atanmaya çalışılan '{}' fonksiyonu bulunamadı.", fn_name));
                            }
                        } else if let Expr::Lambda { params, return_type, .. } = init_expr {
                            let lam_param_types: Vec<_> = params.iter().map(|(_, ty, _)| ty.clone()).collect();
                            if &lam_param_types != param_types || return_type != ret_type.as_ref() {
                                return Err(format!("Hata: '{}' değişkenine atanan lambda imzası uyumsuz.", name));
                            }
                        } else if init_type != resolved_ty {
                            return Err(format!("Hata: '{}' değişkenine atanmaya çalışılan tip ({:?}), deklare edilen tip ({:?}) ile uyuşmuyor.", name, init_type, &resolved_ty));
                        }
                    } else if init_type != resolved_ty && resolved_ty != Type::Any && init_type != Type::Any && init_type != Type::Null {
                        
                        //  Enum tipine temel tamsayı tipinin atanmasına izin ver.
                        if let (Type::Enum(_, base_type), other) = (&resolved_ty, &init_type) {
                            if other.is_integer() && base_type.is_integer() {
                                // Tipler uyumlu, devam et.
                            } else {
                                return Err(format!("Hata: '{}' enum değişkenine atanmaya çalışılan tip ({:?}), beklenen tamsayı tabanlı tip ile uyuşmuyor.", name, init_type));
                            }
                        } else {
                            let allow_float_literal_narrowing = ty.is_float() && init_type.is_float() && matches!(init_expr, Expr::Literal(LiteralValue::Float(_)));
                            
                            // Pozitif I32 literal'den U* tiplerine (ve çözümlenmiş tipe) atamaya izin ver
                            let allow_i32_to_unsigned_literal = resolved_ty.is_unsigned_integer() && init_type == Type::I32 && matches!(init_expr, Expr::Literal(LiteralValue::Int(val)) if *val >= 0);
                            
                            //  'arr' tipine bir dizi literali atanmasına izin ver.
                            let allow_arr_assignment = match (&resolved_ty, &init_type) {
                                (Type::Arr, Type::ArrayLiteral(_)) => true,
                                _ => false,
                            };

                            if !resolved_ty.can_be_assigned_from(&init_type) && !allow_decimal_float_assignment_var_decl && !allow_bit_int_assignment_var_decl && !allow_int_to_bit_array_assignment && !allow_byte_i32_assignment_var_decl && !allow_hex_i32_assignment_var_decl && !allow_float_literal_narrowing && !allow_i32_to_unsigned_literal && !allow_arr_assignment {
                                return Err(format!("Hata: '{}' değişkenine atanmaya çalışılan tip ({:?}), deklare edilen tip ({:?}) ile uyuşmuyor.", name, init_type, &resolved_ty));
                            }
                        }
                    }
				
				}
				let mut resolved_ty = self.resolve_type(ty)?;
				if resolved_ty == Type::Any {
					if let Some(init_expr) = init {
						resolved_ty = self.type_of_expr(init_expr)?;
					} else {
						return Err(format!("Hata: '{}' değişkeni için tip belirtilmedi ve bir başlangıç değeri atanmadı. Tip çıkarımı yapılamıyor.", name));
					}
				}

				let mut info = VarInfo {
					ty: resolved_ty,
					is_const: *is_const,
					_is_mutable: *is_mutable,
				};
                //  Eğer tip bir enum ise, onu Custom'dan Enum(name, base_type)'a dönüştür.
                if let Type::Custom(name) = &info.ty {
                    if let Some(variants) = self.enum_definitions.get(name) {
                        if let Some(first_variant) = variants.values().next() {
                             if let Type::Enum(_, base_type) = first_variant {
                                info.ty = Type::Enum(name.clone(), base_type.clone());
                             }
                        }
                    }
                }

				if let (Type::Array(inner, _), Some(init_expr)) = (&mut info.ty, init) {
					if **inner == Type::Unknown {
						// 'var x: arr = [1, 2, 3]' gibi bir durumda tip çıkarımı yap.
						// init_expr'in tipi Type::ArrayLiteral([I32, I32, I32]) olabilir.
						// Buradan I32'yi çıkarıp 'inner'a atamalıyız.
						if let Ok(init_type) = self.type_of_expr(init_expr) {
							if let Type::ArrayLiteral(elements) = init_type {
								*inner = Box::new(elements.get(0).cloned().unwrap_or(Type::Unknown));
							}
						}
					}
				}
				self.define_variable(name.clone(), info)?;
				Ok(())
			}
            Stmt::Assign { left, value } => {
                let left_type = self.type_of_expr(left)?;
                let right_type = self.type_of_expr(value)?;
                let name = format!("{:?}", left); // Hata mesajları için geçici bir isim

                match self.get_variable_info(&name) {
                    Ok(var_info) => {
                        // Decimal tiplere float atamasını kontrol et
                        let allow_decimal_float_assignment = match (&var_info.ty, &right_type) {
                            (Type::D32 | Type::D64 | Type::D128, Type::F32 | Type::F64 | Type::F80 | Type::F128) => true,
                            _ => false,
                        };
                        
                        // Bit tipine integer literal atamasını kontrol et (0 veya 1)
                        let allow_bit_int_assignment = match (&var_info.ty, value) {
                            (Type::Bit, Expr::Literal(LiteralValue::Int(val))) if *val == 0 || *val == 1 => true,
                            _ => false,
                        };

						// Bit dizisine integer literal atamasını kontrol et
						let allow_int_to_bit_array_assignment = match (&var_info.ty, value) {
							(Type::Array(inner, _), Expr::Literal(LiteralValue::Int(_))) if **inner == Type::Bit => true,
							_ => false,
						};

                        if var_info.is_const {
                            return Err(format!("Hata: Sabit (const) değişken '{}' yeniden atanamaz.", name));
                        }
                        //  'let' ile tanımlanmış ama 'mut' olmayan değişkenlere atamayı engelle.
                        if !var_info._is_mutable && !var_info.is_const {
                            return Err(format!("Hata: Değiştirilemeyen (immutable) değişken '{}' yeniden atanamaz. Değiştirmek için 'mut let' veya 'var' kullanın.", name));
                        }
                        if let Type::Fn(param_types, ret_type) = &var_info.ty {
                            if let Expr::Variable(fn_name) = value {
                                if let Some((expected_params, expected_ret, _, _)) = self.function_signatures.get(fn_name) {
                                    let expected_param_types: Vec<_> = expected_params.iter().map(|(_, ty, _)| ty.clone()).collect();
                                    if &expected_param_types != param_types || expected_ret != ret_type.as_ref() {
                                        return Err(format!("Hata: '{}' fonksiyonuna atanan fonksiyon imzası uyumsuz.", name));
                                    }
                                } else {
                                    return Err(format!("Hata: Atanmaya çalışılan '{}' fonksiyonu bulunamadı.", fn_name));
                                }
                            } else if let Expr::Lambda { params, return_type, .. } = value {
                                let lam_param_types: Vec<_> = params.iter().map(|(_, ty, _)| ty.clone()).collect();
                                if &lam_param_types != param_types || return_type != ret_type.as_ref() {
                                    return Err(format!("Hata: '{}' değişkenine atanan lambda imzası uyumsuz.", name));
                                }
                            } else if right_type != var_info.ty {
                                return Err(format!("Hata: '{}' değişkenine atanmaya çalışılan tip ({:?}), beklenen tip ({:?}) ile uyuşmuyor.", name, right_type, var_info.ty));
                            }
                        } else if right_type != left_type && left_type != Type::Any && right_type != Type::Any && right_type != Type::Null {
                            // I32 literalden Byte atamasına izin ver
                            let allow_byte_i32_assignment = match (&var_info.ty, value) {
                                (Type::Byte, Expr::Literal(LiteralValue::Int(val))) if *val >= 0 && *val <= 255 => true,
                                _ => false,
                            };
                            // I32 literalden Hex atamasına izin ver
                            let allow_hex_i32_assignment = match (&var_info.ty, value) {
                                (Type::Hex, Expr::Literal(LiteralValue::Int(val))) if *val >= 0 && *val <= 255 => true,
                                _ => false,
                            };
                            // Float literallerinin daha dar float tiplerine atanmasına izin ver
                            let allow_float_literal_narrowing = var_info.ty.is_float() && right_type.is_float() && matches!(value, Expr::Literal(LiteralValue::Float(_)));
                            // Pozitif I32 literal'den U* tiplerine atamaya izin ver
							let allow_i32_to_unsigned_literal = var_info.ty.is_unsigned_integer() && right_type == Type::I32 && matches!(value, Expr::Literal(LiteralValue::Int(val)) if *val >= 0);

                            if !left_type.can_be_assigned_from(&right_type) && !allow_decimal_float_assignment && !allow_bit_int_assignment && !allow_int_to_bit_array_assignment && !allow_byte_i32_assignment && !allow_hex_i32_assignment && !allow_float_literal_narrowing && !allow_i32_to_unsigned_literal {
                                return Err(format!("Hata: Atanmaya çalışılan tip ({:?}), beklenen tip ({:?}) ile uyuşmuyor.", right_type, left_type));
                            }
                        }
                    }
                    Err(_) => {
                        let info = VarInfo {
                            ty: right_type,
                            is_const: false,
                            _is_mutable: true,
                        };
                        self.define_variable(name.clone(), info)?;
                    }
                }
                Ok(())
            }
            Stmt::If { cond, then_branch, else_branch } => {
                let cond_type = self.type_of_expr(cond)?;
                if cond_type != Type::Bool {
                    return Err(format!("Hata: 'if' koşulu Boolean tipinde olmalıdır, bulundu: {:?}.", cond_type));
                }
                self.push_scope();
                self.check_stmt(then_branch)?;
                self.pop_scope()?;
                if let Some(else_stmt) = else_branch {
                    self.push_scope();
                    self.check_stmt(else_stmt)?;
                    self.pop_scope()?;
                }
                Ok(())
            }
            Stmt::While { condition, body } => {
                let cond_type = self.type_of_expr(condition)?;
                if cond_type != Type::Bool {
                    return Err(format!("Hata: While koşulu bool tipinde olmalıdır, bulundu: {:?}", cond_type));
                }
                self.check_block_stmt(body)?;
                Ok(())
            }
            Stmt::Loop { body } => {
                self.check_block_stmt(body)?;
                Ok(())
            }
            Stmt::For { initializer, condition, increment, variable, iterable, body } => {
                self.push_scope();
                if let (Some(var_name), Some(iter_expr)) = (variable, iterable) {
                    let iterable_type = self.type_of_expr(iter_expr)?;
                    let inner_type = match iterable_type {
                        // Durum 1: `for i in my_array`
                        Type::Array(inner, _) => *inner,
                        Type::Arr => Type::Any, // 'arr' tipiyle döngü kuruluyorsa, eleman tipini 'Any' kabul et.
                        //  Durum 2: `for i in 0..10`
                        Type::Custom(s) if s.starts_with("Range<") => {
                            // "Range<Type::I32>" gibi bir string'den I32 tipini çıkar.
                            // Bu basit bir çözüm, daha sonra daha sağlam bir hale getirilebilir.
                            if s.contains("I32") { Type::I32 }
                            else if s.contains("F64") { Type::F64 }
                            else { Type::Unknown } // Diğer aralık tipleri eklenebilir.
                        },
                        _ => {
                            return Err(format!(
                                "Hata: 'for-in' döngüsü sadece diziler veya aralıklar (range) üzerinde çalışır, bulundu: {:?}.",
                                iterable_type
                            ));
                        }
                    };

                        let info = VarInfo {
                        ty: inner_type,
                        is_const: false, // Döngü değişkeni her iterasyonda yeniden atanır.
                            _is_mutable: false,
                        };
                        self.define_variable(var_name.clone(), info)?;
                } else {
                    if let Some(init_stmt) = initializer {
                        if let Stmt::VarDecl { name, ty, init, is_const, is_let: _, is_mutable, .. } = &**init_stmt {
                            // Durum 1: `for (var i: i32 = 0; ...)` gibi açık bir tanım varsa.
                            self.check_and_define_variable(name, ty, init, is_const, is_mutable)?; // check_and_define_variable is_let'i kullanmıyor, bu yüzden burada görmezden gelebiliriz.
                        } else if let Stmt::ExprStmt(expr) = &**init_stmt {
                            // Durum 2: `for (i = 0; ...)` gibi örtük bir tanım varsa.
                            if let Expr::Assign { left, value } = expr { // `expr` bir `&Expr`
                                if let Expr::Variable(name) = &**left { // `left` bir `&Expr`
                                    // Sol taraf bir değişken. Sağ tarafın tipini çıkar.
                                    let var_type = self.type_of_expr(value)?;
                                    // Değişkeni döngü kapsamında 'mut' olarak tanımla.
                                    let info = VarInfo {
                                        ty: var_type,
                                        is_const: false,
                                        _is_mutable: true,
                                    };
                                    self.define_variable(name.clone(), info)?;
                                }
                            }
                        } else {
                            self.check_stmt(init_stmt)?;
                        }
                    }
                    if let Some(cond_expr) = condition {
                        let cond_type = self.type_of_expr(cond_expr)?;
                        if cond_type != Type::Bool {
                            return Err(format!("Hata: 'for' döngüsü koşulu bool tipinde olmalıdır, bulundu: {:?}", cond_type));
                        }
                    }
                    if let Some(inc_expr) = increment {
                        self.type_of_expr(inc_expr)?;
                    }
                }
                self.check_block_stmt_no_scope(body)?;
                self.pop_scope()?;
                Ok(())
            }
            Stmt::Echo(expr) => {
                self.type_of_expr(expr)?;
                Ok(())
            }
            Stmt::Return(expr) => {
                let actual_type = match expr {
                    Some(e) => self.type_of_expr(e)?,
                    None => Type::Void,
                };

                //  Eğer mevcut fonksiyon 'async' ise, beklenen dönüş tipi Future<T> değil, T'dir.
                // Bu yüzden 'actual_type' ile 'self.expected_return_type'ı doğrudan karşılaştırabiliriz.
                // Fonksiyonun genel imzasının Future<T> olduğu kontrolü zaten ilk taramada yapıldı.

                if self.expected_return_type == Type::Any {
                    return Ok(());
                }

                if actual_type != self.expected_return_type {
                    // Hata mesajını daha anlaşılır hale getirelim.
                    //  Mevcut fonksiyon adını kullanarak doğru imzayı bul.
                    let signature_return_type = self.current_function_name.as_ref()
                        .and_then(|name| self.function_signatures.get(name))
                        .map(|(_, ret, _, _)| ret.clone());
                    return Err(format!("Hata: Fonksiyondan dönülen tip ({:?}), beklenen tip ({:?}) ile uyuşmuyor. Fonksiyon imzası dönüş tipi: {:?}.", actual_type, self.expected_return_type, signature_return_type.unwrap_or(Type::Unknown)));
                }
                Ok(())
            }
            Stmt::Break => Ok(()),
            Stmt::Continue => Ok(()),
            Stmt::ExprStmt(expr) => {
                // if let Expr::Assign { name, value } = expr {
                //     return self.check_stmt(&Stmt::Assign { name: name.clone(), value: *value.clone() });
                // }
                self.type_of_expr(expr)?;
                Ok(())
            }
            Stmt::Routine(expr) => {
                // 'routine' sadece bir fonksiyon çağrısı ile kullanılabilir.
                if let Expr::Call { .. } = expr.as_ref() {
                    self.type_of_expr(expr)?;
                } else {
                    return Err(format!("Hata: 'routine' anahtar kelimesi sadece bir fonksiyon çağrısı ile kullanılabilir, bulundu: {:?}.", expr));
                }
                Ok(())
            }
            Stmt::LabeledStmt { label, stmt, .. } => { //  'is_public' alanını .. ile yoksay
                self.define_label(label.clone())?;
                self.push_scope();
                let info = VarInfo {
                    ty: Type::I32,
                    is_const: false,
                    _is_mutable: true,
                };
                self.define_variable("$rolling".to_string(), info)?;
                self.check_stmt(stmt)?;
                self.pop_scope()?;
                Ok(())
            }
            Stmt::Rolling(tag) => {
                if !self.get_label(tag) {
                    return Err(format!("Hata: Tanımlanmamış etiket '{}'", tag));
                }
                Ok(())
            }
            Stmt::LabeledExpr { label: _, expr } => {
                self.type_of_expr(expr)?;
                Ok(())
            }
            Stmt::Tag { .. } => Ok(()),
            Stmt::Unsafe(block) => {
                // `unsafe` bloğu, içindeki kodun tip kontrolünü etkilemez.
                // Sadece derleyiciye "buradaki işlemlerin güvensiz olabileceğini biliyorum" mesajı verir.
                self.check_stmt(block)
            },
            Stmt::FastExec(block) => {
                // `fastexec` bloğu da `unsafe` gibi, içindeki kodun tip kontrolünü etkilemez.
                // Sadece kod üretimi aşamasında derleyiciye optimizasyon ipuçları verir.
                let was_in_fastexec = self.in_fastexec_block;
                self.in_fastexec_block = true;
                let result = self.check_stmt(block);
                self.in_fastexec_block = was_in_fastexec;
                result
            },
            Stmt::Asm { .. } => {
                //  `asm` blokları sadece `fastexec` içinde kullanılabilir.
                if !self.in_fastexec_block {
                    return Err("Hata: 'asm' blokları yalnızca bir 'fastexec' bloğu içinde kullanılabilir.".to_string());
                }
                // İçeriği dilin tip sisteminin dışındadır, bu yüzden başka kontrol gerekmez.
                Ok(())
            }
        }
    }

    // check_stmt içindeki VarDecl mantığını dışarı taşıyan yeni yardımcı fonksiyon
    fn check_and_define_variable(&mut self, name: &String, ty: &Type, init: &Option<Expr>, is_const: &bool, is_mutable: &bool) -> Result<(), String> {
        if *is_const && init.is_none() {
            return Err(format!("Hata: Sabit (const) '{}' tanımlanırken bir başlangıç değeri atanmalıdır.", name));
        }
        if *is_const && *is_mutable {
            return Err(format!("Hata: '{}' hem sabit (const) hem de değiştirilebilir (mut) olarak tanımlanamaz.", name));
        }
        if let Type::Custom(type_name) = ty {
            if !self.struct_definitions.contains_key(type_name) && !self.function_signatures.contains_key(type_name) {
                return Err(format!("Hata: Bilinmeyen tip '{}' kullanıldı.", type_name));
            }
        }
        if let Some(init_expr) = init {
            let init_type = self.type_of_expr(init_expr)?;

            let allow_decimal_float_assignment_var_decl = match (ty, &init_type) {
                (Type::D32 | Type::D64 | Type::D128, Type::F32 | Type::F64 | Type::F80 | Type::F128) => true,
                _ => false,
            };
            let allow_bit_int_assignment_var_decl = match (ty, init_expr) {
                (Type::Bit, Expr::Literal(LiteralValue::Int(val))) if *val == 0 || *val == 1 => true,
                _ => false,
            };
            let allow_int_to_bit_array_assignment = match (ty, init_expr) {
                (Type::Array(inner, _), Expr::Literal(LiteralValue::Int(_))) if **inner == Type::Bit => true,
                _ => false,
            };
            let allow_byte_i32_assignment_var_decl = match (ty, init_expr) {
                (Type::Byte, Expr::Literal(LiteralValue::Int(val))) if *val >= 0 && *val <= 255 => true,
                _ => false,
            };
            let allow_hex_i32_assignment_var_decl = match (ty, init_expr) {
                (Type::Hex, Expr::Literal(LiteralValue::Int(val))) if *val >= 0 && *val <= 255 => true,
                _ => false,
            };

            if let (Type::Array(expected_inner_type, _), Type::ArrayLiteral(element_types)) = (ty, &init_type) {
                if !element_types.is_empty() {
                    let first_element_type = &element_types[0];
                    // Tüm elemanların aynı tipte olduğunu kontrol et (ArrayLiteral kontrolünde zaten yapılıyor ama burada da zararı olmaz)
                    for element_type in element_types.iter().skip(1) {
                        if element_type != first_element_type {
                            return Err(format!("Hata: Dizi başlatıcısındaki tüm elemanlar aynı tipte olmalıdır. Bulunan tipler: {:?}.", element_types));
                        }
                    }
                    // Atanan dizinin tipi, değişkenin beklenen iç tipiyle uyuşuyor mu?
                    if **expected_inner_type != Type::Unknown && **expected_inner_type != *first_element_type {
                        return Err(format!("Hata: '{}' dizisine atanmaya çalışılan eleman tipi ({:?}), beklenen tip ({:?}) ile uyuşmuyor.", name, first_element_type, expected_inner_type));
                    }
                }
            } else if init_type != *ty && *ty != Type::Any && init_type != Type::Any && init_type != Type::Null {
                let allow_float_literal_narrowing = ty.is_float() && init_type.is_float() && matches!(init_expr, Expr::Literal(LiteralValue::Float(_)));
                let allow_i32_to_unsigned_literal = ty.is_unsigned_integer() && init_type == Type::I32 && matches!(init_expr, Expr::Literal(LiteralValue::Int(val)) if *val >= 0);

                if !ty.can_be_assigned_from(&init_type) && !allow_decimal_float_assignment_var_decl && !allow_bit_int_assignment_var_decl && !allow_int_to_bit_array_assignment && !allow_byte_i32_assignment_var_decl && !allow_hex_i32_assignment_var_decl && !allow_float_literal_narrowing && !allow_i32_to_unsigned_literal {
                    return Err(format!("Hata: '{}' değişkenine atanmaya çalışılan tip ({:?}), deklare edilen tip ({:?}) ile uyuşmuyor.", name, init_type, ty));
                }
            }
        }
        let mut info = VarInfo {
            ty: ty.clone(),
            is_const: *is_const,
            _is_mutable: *is_mutable,
        };
        
        // Type::Arr için özel işlem: boyut çıkarımı yap ama tipi değiştirme
        if info.ty == Type::Arr {
            if let Some(init_expr) = init {
                if let Ok(init_type) = self.type_of_expr(init_expr) {
                    if let Type::ArrayLiteral(_elements) = init_type {
                        // Type::Arr olarak kalsın, sadece boyut bilgisini not et
                        // Boyut bilgisi codegen'de ArrayLiteral'den alınacak
                    }
                }
            }
        }
        
        // Type::Array için tip çıkarımı (homojen arrayler)
        if let (Type::Array(inner, _), Some(init_expr)) = (&mut info.ty, init) {
            if **inner == Type::Unknown {
                if let Ok(init_type) = self.type_of_expr(init_expr) {
                    if let Type::ArrayLiteral(elements) = init_type {
                        // İlk elemanın tipini kullan (homojen array için)
                        *inner = Box::new(elements.get(0).cloned().unwrap_or(Type::Unknown));
                        // Dizinin boyutunu da çıkar (Inference)
                        if let Type::Array(_, len_opt) = &mut info.ty {
                            if len_opt.is_none() {
                                *len_opt = Some(elements.len());
                            }
                        }
                    }
                }
            }
        }
        self.define_variable(name.clone(), info)
    }

    pub fn type_of_expr(&mut self, expr: &Expr) -> Result<Type, String> {
        match expr {
            Expr::Block { statements } => {
                self.push_scope();
                let mut return_type = Type::Void; // Varsayılan dönüş tipi
                let mut has_return = false;
    
                for stmt in statements { // `statements` bir `&Vec<Stmt>`
                    self.check_stmt(stmt)?;
                    if let Stmt::Return(Some(expr)) = stmt {
                        let current_return_type = self.type_of_expr(expr)?;
                        if has_return && return_type != current_return_type {
                            let _ = self.pop_scope();
                            return Err("Hata: Bir blok içindeki tüm 'return' ifadeleri aynı tipi döndürmelidir.".to_string());
                        }
                        return_type = current_return_type;
                        has_return = true;
                    }
                }
    
                self.pop_scope()?;
                Ok(return_type)
            },
            Expr::Literal(lit) => Ok(match lit {
                LiteralValue::Int(_) => Type::I32,
                LiteralValue::Float(_) => Type::F64,
                LiteralValue::Hex(_) => Type::Hex,
                LiteralValue::Str(_) => Type::Str(None),
                LiteralValue::Bool(_) => Type::Bool,
                LiteralValue::Char(_) => Type::Char,
                LiteralValue::Null => Type::Null,
            }),
            Expr::Try(expr) => {
                let expr_type = self.type_of_expr(expr)?;
                match expr_type {
                    Type::Result(ok_type, err_type) => {
                        // Fonksiyonun dönüş tipi de uyumlu bir Result olmalı.
                        if let Type::Result(_, expected_err_type) = &self.expected_return_type {
                            if *err_type != **expected_err_type {
                                return Err(format!("Hata: '?' operatörü, fonksiyonun dönüş hatası tipiyle ({:?}) uyumsuz bir hata tipi ({:?}) döndürebilir.", expected_err_type, err_type));
                            }
                            // Her şey yolundaysa, ifade 'Ok' içindeki değeri döndürür.
                            Ok(*ok_type)
                        } else {
                            Err(format!("Hata: '?' operatörü yalnızca dönüş tipi 'Result<T, E>' olan fonksiyonlar içinde kullanılabilir. Bulunan dönüş tipi: {:?}.", self.expected_return_type))
                        }
                    }
                    _ => Err(format!("Hata: '?' operatörü yalnızca 'Result<T, E>' tipindeki ifadelere uygulanabilir, bulundu: {:?}.", expr_type)),
                }
            },
            Expr::Tuple(elements) => {
                let mut element_types = Vec::new(); // `elements` bir `&Vec<Expr>`
                for elem in elements {
                    element_types.push(self.type_of_expr(elem)?);
                }
                Ok(Type::Tuple(element_types))
            }
            Expr::ArrayLiteral(elements) => {
                let mut element_types = Vec::new(); // `elements` bir `&Vec<Expr>`
                for elem in elements {
                    element_types.push(self.type_of_expr(elem)?);
                }
                // Dizi homojen olmalı, tüm elemanlar aynı tipte olmalı
                if let Some(first_type) = element_types.first() {
                    if !element_types.iter().all(|t| t == first_type) {
                        return Err(format!("Hata: Dizi literali içindeki tüm elemanlar aynı tipte olmalıdır. Bulunan tipler: {:?}.", element_types));
                    }
                }
                Ok(Type::ArrayLiteral(element_types))
            }
            Expr::Conditional { cond, then_branch, else_branch } => {
                let cond_type = self.type_of_expr(cond)?;
                if cond_type != Type::Bool {
                    return Err(format!("Hata: Ternary operatör koşulu Boolean tipinde olmalıdır, bulundu: {:?}.", cond_type));
                }

                let then_type = self.type_of_expr(then_branch)?;
                let else_type = self.type_of_expr(else_branch)?;

                if then_type != else_type && then_type != Type::Any && else_type != Type::Any && then_type != Type::Null && else_type != Type::Null {
                    return Err(format!("Hata: Ternary operatörünün her iki kolu da aynı tipi döndürmelidir. Bulunan tipler: {:?} ve {:?}.", then_type, else_type));
                }
                Ok(then_type)
            }
            //  'await' ifadesinin tip kontrolü.
            Expr::Await(expr) => {
                let expr_type = self.type_of_expr(expr)?;
                if let Type::Future(inner_type) = expr_type {
                    // 'await' bir Future<T> alır ve T döndürür.
                    Ok(*inner_type)
                } else {
                    Err(format!("Hata: 'await' sadece Future tipindeki ifadelere uygulanabilir, bulundu: {:?}", expr_type))
                }
            },
            Expr::Assign { left, value } => {
                let left_type = self.type_of_expr(left)?;
                let right_type = self.type_of_expr(value)?;

                // Atama yapılabilir mi kontrolü (l-value kontrolü)
                if !matches!(**left, Expr::Variable(_) | Expr::MemberAccess {..} | Expr::ArrayAccess {..}) {
                    return Err(format!("Hata: Atama ifadesinin sol tarafı bir değişkene, struct alanına veya dizi elemanına atanabilir olmalıdır."));
                }

                if left_type != right_type && left_type != Type::Any && right_type != Type::Any {
                    return Err(format!("Hata: Atama işleminde tipler uyuşmuyor. Beklenen: {:?}, Bulunan: {:?}", left_type, right_type));
                }
                // Atama ifadesi, atanan değeri döndürür.
                Ok(right_type)
            },
            Expr::ArrayAccess { name, index } => {
                let array_info = self.get_variable_info(name)?; // `name` bir `&String`
                let array_type = array_info.ty;
                if let Type::Array(inner_type, _) = array_type {
                    let index_type = self.type_of_expr(index)?;
                    if index_type != Type::I32 {
                        return Err(format!("Hata: Dizi indeksi tam sayı (I32) olmalıdır, bulundu: {:?}.", index_type));
                    }
                    Ok(*inner_type)
                } else {
                    Err(format!("Hata: Array erişimi, dizi olmayan tipe ('{}' : {:?}) uygulanamaz.", name, array_type))
                }
            }
            Expr::MemberAccess { object, member } => {
                let object_type = self.type_of_expr(object)?;
                match &object_type {
                    Type::Custom(name) => {
                        //  Tek bir `Type::Custom` dalında hem struct hem de group kontrolü yap.
                        
                        // 1. Struct alanı mı?
                        if let Some(struct_def) = self.struct_definitions.get(name) {
                            if let Some(field_type) = struct_def.get(member) {
                                return Ok(field_type.clone());
                            }
                        }

                        // 2. Struct metodu mu?
                        if let Some(method_map) = self.method_signatures.get(name) {
                            if let Some((params, ret_type, _is_public)) = method_map.get(member) {
                                let param_types = params.iter().skip(1).map(|(_, ty, _)| ty.clone()).collect();
                                return Ok(Type::Fn(param_types, Box::new(ret_type.clone())));
                            }
                        }

                        // 3. Normal grup üyesi mi?
                        if let Some(group_content) = self.group_definitions.get(name) {
                            // Önce sabit (const) olup olmadığını kontrol et
                            if let Some(const_info) = group_content.constants.get(member) {
                                return Ok(const_info.ty.clone());
                            }
                            // Sonra fonksiyon olup olmadığını kontrol et
                            if let Some((params, ret_type, _, _)) = group_content.functions.get(member) {
                                let param_types = params.iter().map(|(_, ty, _)| ty.clone()).collect();
                                return Ok(Type::Fn(param_types, Box::new(ret_type.clone())));
                            }
                        }

                        // 4. Hiçbiri değilse hata ver.
                        Err(format!("Hata: '{}' tipinin '{}' isminde bir alanı veya üyesi yok.", name, member))
                    }
                    Type::Channel(inner_type) => {
                        if member == "new" {
                            Ok(Type::Channel(inner_type.clone()))
                        } else {
                            Err(format!("Hata: Kanal tipinin '{}' isminde bir metodu yok.", member))
                        }
                    }
                    _ => Err(format!("Hata: Üye erişimi ('.') yalnızca struct, grup veya kanal tiplerine uygulanabilir, bulundu: {:?}.", object_type)),
                }
            }
            Expr::Variable(name) => {
                //  Merkezi isim çözümleme mantığını kullan.
                match self.get_variable_info(name) {
                    Ok(info) => Ok(info.ty),
                    Err(_) => {
                        // Eğer `get_variable_info` bulamazsa, bu bir grup adı olabilir.
                        if self.group_definitions.contains_key(name) { // `name` bir `&str`
                            Ok(Type::Custom(name.clone()))
                        } else {
                            Err(format!("Hata: Tanımlanmamış değişken veya grup adı: '{}'.", name))
                        }
                    }
                }
            },
            Expr::Input(prompt_opt) => {
                if let Some(prompt_expr) = prompt_opt {
                    let ty = self.type_of_expr(prompt_expr)?;
                    
                    // ty.is_str() yerine bunu yaz:
                    if !matches!(ty, Type::Str(_)) {
                        return Err("input() fonksiyonuna verilen prompt bir metin (string) olmalıdır.".to_string());
                    }
                }
                Ok(Type::Str(None)) // input her zaman string döner
            },
            Expr::Range { start, end } => {
                let start_type = self.type_of_expr(start)?;
                let end_type = self.type_of_expr(end)?;
                
                if !start_type.is_integer() || !end_type.is_integer() {
                    return Err(format!("Hata: Aralık (range) operatörü '..' sadece tamsayılar arasında çalışır, bulundu: {:?} .. {:?}.", start_type, end_type));
                }
                
                // Şimdilik Range'i Custom olarak işaretliyoruz, for döngüsü bunu tanıyacak.
                Ok(Type::Custom(format!("Range<{:?}>", start_type)))
            }
            Expr::Binary { left, op, right } => {
                let left_type = self.type_of_expr(left)?;
                let right_type = self.type_of_expr(right)?;
                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                        // Farklı sayısal tipler arasında işlemlere izin ver (örn: f64 / i32)
                        if left_type.is_float() && right_type.is_integer() {
                            Ok(left_type) // Sonuç float olur
                        } else if left_type.is_integer() && right_type.is_float() {
                            Ok(right_type) // Sonuç float olur
                        } else if left_type == right_type {
                            // Tipler aynıysa, işlemin bu tip için geçerli olup olmadığını kontrol et
                            match left_type {
                                Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::I128 |
                                Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::U128 |
                                Type::F32 | Type::F64 | Type::F80 | Type::F128 |
                                Type::D32 | Type::D64 | Type::D128 => Ok(left_type),
                                _ => Err(format!("Hata: Aritmetik işlem sayısal olmayan tipe ({:?}) uygulanamaz.", left_type)),
                            }
                        } else if left_type == Type::Any || right_type == Type::Any {
                            // 'any' tipiyle esneklik sağla
                            if left_type == Type::Any { Ok(right_type) } else { Ok(left_type) }
                        } else {
                            // Diğer tüm uyumsuz tipler için hata ver
                            Err(format!("Hata: Aritmetik işlemde tipler uyuşmuyor: {:?} {:?} {:?}.", left_type, op, right_type))
                        }
                    }
                    BinOp::Equal | BinOp::NotEqual | BinOp::Greater | BinOp::Less | BinOp::GreaterEqual | BinOp::LessEqual | BinOp::Identical | BinOp::NotIdentical => {
                        if left_type != right_type && left_type != Type::Any && right_type != Type::Any && left_type != Type::Null && right_type != Type::Null {
                            // Allow comparing special types with integers
                            let is_compatible = match (&left_type, &right_type) {
                                (Type::Char, Type::I32) | (Type::I32, Type::Char) => true,
                                (Type::Bit, Type::I32) | (Type::I32, Type::Bit) => true,
                                (Type::Byte, Type::I32) | (Type::I32, Type::Byte) => true,
                                (Type::Hex, Type::I32) | (Type::I32, Type::Hex) => true,
                                (Type::Enum(_, _), other) if other.is_integer() => true,
                                (other, Type::Enum(_, _)) if other.is_integer() => true,
                                _ => false,
                            };
                            if !is_compatible {
                                return Err(format!("Hata: Karşılaştırma işleminde tipler uyuşmuyor: {:?} {:?} {:?}.", left_type, op, right_type));
                            }
                        }
                        Ok(Type::Bool)
                    }
                    BinOp::And | BinOp::Or => {
                        if left_type != Type::Bool || right_type != Type::Bool {
                            return Err(format!("Hata: Mantıksal işlemde tipler Bool olmalıdır, bulundu: {:?} ve {:?}.", left_type, right_type));
                        }
                        Ok(Type::Bool)
                    }
                    BinOp::BitwiseAnd | BinOp::BitwiseOr | BinOp::BitwiseXor | BinOp::LShift | BinOp::RShift => {
                        if !left_type.is_integer() || !right_type.is_integer() {
                            return Err(format!("Hata: Bitsel işlem yalnızca tamsayı tiplerine uygulanabilir, bulundu: {:?} ve {:?}.", left_type, right_type));
                        }
                        if left_type != right_type {
                            // Şimdilik farklı tamsayı tipleri arasında işleme izin vermiyoruz.
                            return Err(format!("Hata: Bitsel işlemde tipler uyuşmuyor: {:?} {:?} {:?}.", left_type, op, right_type));
                        }
                        Ok(left_type) // Sonuç tipi, işlenenlerin tipiyle aynıdır.
                    }
                    _ => Err(format!("Hata: Bilinmeyen veya desteklenmeyen ikili operatör: {:?}.", op)),
                }
            }
            Expr::Unary { op, right } => {
                let right_type = self.type_of_expr(right)?;
                match op {
                    UnOp::Neg => match right_type {
                        Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::I128 |
                        Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::U128 |
                        Type::F32 | Type::F64 | Type::F128 |
                        Type::D32 | Type::D64 | Type::D128 => Ok(right_type),
                        _ => Err(format!("Hata: Negatifleştirme ({:?}) sayısal olmayan tipe ({:?}) uygulanamaz.", op, right_type)),
                    },
                    UnOp::Not => {
                        if right_type != Type::Bool {
                            return Err(format!("Hata: Mantıksal DEĞİL (!{:?}) Bool olmayan tipe ({:?}) uygulanamaz.", op, right_type));
                        }
                        Ok(Type::Bool)
                    }
                    UnOp::AddressOf => {
                        // &x ifadesi, x bir l-value (atanabilir bir yer) olmalıdır.
                        if !matches!(right.as_ref(), Expr::Variable(_)) {
                             return Err(format!("Hata: Adres alma operatörü '&' sadece değişkenlere uygulanabilir, bulundu: {:?}.", right));
                        }
                        // Sonuç, ifadenin tipine bir pointer'dır. T -> *T
                        Ok(Type::Ptr(Box::new(right_type)))
                    }
                    UnOp::Deref => {
                        // *p ifadesi, p bir pointer olmalıdır.
                        if let Type::Ptr(inner_type) = right_type {
                            // Sonuç, pointer'ın işaret ettiği tiptir. *T -> T
                            Ok(*inner_type)
                        } else {
                            Err(format!("Hata: Dereferans operatörü '*' sadece pointer tiplerine uygulanabilir, bulundu: {:?}.", right_type))
                        }
                    }
                    _ => {
                        let right_type = self.type_of_expr(right)?;
                        match right_type {
                            Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::I128 |
                            Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::U128 |
                            Type::F32 | Type::F64 | Type::F128 |
                            Type::D32 | Type::D64 | Type::D128 => {
                                if let Expr::Variable(name) = right.as_ref() {
                                    let var_info = self.get_variable_info(name)?;
                                    if !var_info._is_mutable {
                                        return Err(format!("Hata: Artırma/azaltma operatörü değiştirilemeyen (immutable) değişkene uygulanamaz: '{}'", name));
                                    }
                                } else {
                                    return Err(format!("Hata: Artırma/azaltma operatörü sadece değişkenlere uygulanabilir."));
                                }
                                Ok(right_type)
                            },
                            _ => Err(format!("Hata: Artırma/azaltma operatörü sayısal olmayan tipe ({:?}) uygulanamaz.", right_type)),
                        }
                    }
                }
            }
            Expr::Call { callee, args } => {
                // echo ve print gibi özel, esnek (variadic) fonksiyonlar için öncelikli kontrol.

                //  clone() fonksiyonu için özel kontrol
                if let Expr::Variable(callee_name) = &**callee {
                    if callee_name == "clone" {
                        if args.len() != 1 {
                            return Err("Hata: 'clone' fonksiyonu tam olarak bir argüman bekler.".to_string());
                        }
                        if let Some(arg_name) = &args[0].0 {
                            return Err(format!("Hata: 'clone' fonksiyonu isimlendirilmiş argüman ('{}') almaz.", arg_name));
                        }

                        let arg_type = self.type_of_expr(&args[0].1)?; // `args[0].1` bir `Expr`

                        // Klonlanabilir tipleri kontrol et
                        match arg_type {
                            Type::Custom(_) | Type::Enum(_, _) | Type::Array(_, _) | Type::Tuple(_) => {
                                return Ok(arg_type); // Klonlanmış ifadenin tipi, orijinaliyle aynıdır.
                            },
                            _ => return Err(format!("Hata: Sadece struct, enum, dizi ve tuple tipleri klonlanabilir, bulundu: {:?}.", arg_type)),
                        }
                    }
                }

                let callee_type = self.type_of_expr(callee)?; // `callee` bir `&Expr`

                let (params_def, return_type) = match callee_type {
                    Type::Fn(ref param_types, ref ret_type) => {
                        let params: Vec<(String, Type, bool)> = param_types.iter().map(|t| ("".to_string(), t.clone(), false)).collect();
                        (params, *ret_type.clone())
                    }
                    _ => return Err(format!("Hata: Çağrılabilir olmayan bir ifade çağrılamaz: {:?}", callee)),
                };

                // Argüman kontrolü için hazırlık
                let mut provided_args = HashSet::new();
                let mut positional_arg_index = 0;

                // 1. Adım: Argümanları doğrula ve parametrelerle eşleştir
                for (arg_name_opt, arg_expr) in args {
                    let arg_type = self.type_of_expr(arg_expr)?; // `arg_expr` bir `&Expr`
                    // Fonksiyon pointer'ı/lambda çağrılarında sadece pozisyonel argümanlara izin ver.
                    // Çünkü parametre isimleri bu noktada bilinmiyor.
                    if let Type::Fn(..) = callee_type {
                        if arg_name_opt.is_some() {
                            return Err("Hata: Fonksiyon pointer'ları veya lambdalar aracılığıyla yapılan çağrılarda isimlendirilmiş argümanlar kullanılamaz.".to_string());
                        }
                    }
                    
                    if let Some(name) = arg_name_opt {
                        // İsimli Argüman (named argument)
                        let param_index = params_def.iter().position(|(p_name, _, _)| p_name == name)
                            .ok_or_else(|| format!("Hata: Fonksiyonun '{}' isminde bir parametresi yok.", name))?;
                        
                        if !provided_args.insert(name.clone()) {
                            return Err(format!("Hata: '{}' parametresi birden fazla kez sağlandı.", name));
                        }
                        
                        let (_, expected_type, _) = &params_def[param_index];
                        //  Beklenen tipi, typedef ise gerçek tipine çözümle.
                        let resolved_expected_type = self.resolve_type(expected_type)?;

                        
                        // arrlen için özel kontrol: Herhangi bir dizi tipini kabul et
                        if let Expr::Variable(callee_name) = &**callee {
                            if callee_name == "arrlen" {
                                if arg_type.is_array() || arg_type == Type::Arr {
                                    // Tip doğru, devam et
                                } else {
                                    return Err(format!("Hata: 'arrlen' fonksiyonu bir dizi bekler, bulundu: {:?}.", arg_type));
                                }
                            } else if arg_type != resolved_expected_type && resolved_expected_type != Type::Any && arg_type != Type::Any && arg_type != Type::Null {
                                return Err(format!("Hata: '{}' parametresi için tip uyuşmazlığı: beklenen {:?}, bulunan {:?}.", name, &resolved_expected_type, arg_type));
                            }
                        } else if resolved_expected_type.is_array() && arg_type == Type::Arr {
                            // Genel durum: Eğer bir fonksiyon Array bekliyorsa ve Arr gönderildiyse, kabul et.
                            // Bu, arrlen dışındaki fonksiyonlar için de çalışır.
                            // Tip doğru, devam et.
                        } else if arg_type != resolved_expected_type && resolved_expected_type != Type::Any && arg_type != Type::Any && arg_type != Type::Null {
                             return Err(format!("Hata: '{}' parametresi için tip uyuşmazlığı: beklenen {:?}, bulunan {:?}.", name, &resolved_expected_type, arg_type));
                        }
                    } else {
                        // Pozisyonel Argüman (positional argument)
                        if positional_arg_index >= params_def.len() {
                            return Err(format!("Hata: Fonksiyona çok fazla argüman verildi. Beklenen: {}, Sağlanan: {}", params_def.len(), args.len()));
                        }
                        
                        let (param_name, expected_type, _) = &params_def[positional_arg_index];
                        //  Beklenen tipi, typedef ise gerçek tipine çözümle.
                        let resolved_expected_type = self.resolve_type(expected_type)?;

                        // Eğer parametre adı boşsa (bu bir fonksiyon pointer'ı çağrısıdır),
                        // bu kontrolü atla çünkü tüm parametreler aynı boş isme sahip olacaktır.
                        if !param_name.is_empty() {
                            if provided_args.contains(param_name) {
                                return Err(format!("Hata: '{}' parametresi hem pozisyonel hem de isimlendirilmiş olarak sağlandı.", param_name.clone()));
                            }
                        }
                        
                        // arrlen için özel kontrol: Herhangi bir dizi tipini kabul et
                        if let Expr::Variable(callee_name) = &**callee {
                            if callee_name == "arrlen" {
                                if arg_type.is_array() || arg_type == Type::Arr {
                                    // Tip doğru, devam et
                                } else {
                                    return Err(format!("Hata: 'arrlen' fonksiyonu bir dizi bekler, bulundu: {:?}.", arg_type));
                                }
                            } else {
                                //  Pozitif i32 literallerinin işaretsiz tamsayı parametrelerine atanmasına izin ver.
                                let allow_i32_to_unsigned_literal = resolved_expected_type.is_unsigned_integer() &&
                                    arg_type == Type::I32 &&
                                    matches!(arg_expr, Expr::Literal(LiteralValue::Int(val)) if *val >= 0);

                                if arg_type != resolved_expected_type && resolved_expected_type != Type::Any && arg_type != Type::Any && arg_type != Type::Null && !allow_i32_to_unsigned_literal {
                                    return Err(format!("Hata: {}. parametre tipi uyumsuz: beklenen {:?}, bulundu {:?}.", positional_arg_index + 1, &resolved_expected_type, arg_type));
                                }
                            }
                        } else if resolved_expected_type.is_array() && arg_type == Type::Arr {
                            // Genel durum: Eğer bir fonksiyon Array bekliyorsa ve Arr gönderildiyse, kabul et.
                            // Bu, arrlen dışındaki fonksiyonlar için de çalışır.
                            // Tip doğru, devam et.
                        } else {
                            let allow_i32_to_unsigned_literal = resolved_expected_type.is_unsigned_integer() &&
                                arg_type == Type::I32 &&
                                matches!(arg_expr, Expr::Literal(LiteralValue::Int(val)) if *val >= 0);

                            if arg_type != resolved_expected_type && resolved_expected_type != Type::Any && arg_type != Type::Any && arg_type != Type::Null && !allow_i32_to_unsigned_literal {
                                return Err(format!("Hata: {}. parametre tipi uyumsuz: beklenen {:?}, bulundu {:?}.", positional_arg_index + 1, &resolved_expected_type, arg_type));
                            }
                        }

                        provided_args.insert(param_name.clone());
                        positional_arg_index += 1;
                    }
                }

                // 2. Adım: Eksik argümanları kontrol et
                for (param_name, _, has_default) in &params_def {
                    // Eğer parametrenin varsayılan değeri yoksa ve çağrıda sağlanmadıysa, bu bir hatadır.
                    if !has_default && !provided_args.contains(param_name) {
                        return Err(format!("Hata: Gerekli olan '{}' parametresi sağlanmadı.", param_name));
                    }
                }

                Ok(return_type)
            }
            Expr::Match { discriminant, cases } => {
                let discriminant_type = self.type_of_expr(discriminant)?;
                let mut case_types = Vec::new();

                for (pattern, result) in cases {
                    let pattern_type = self.type_of_expr(pattern)?; // `pattern` bir `&Expr`
                    if pattern_type != discriminant_type && pattern_type != Type::Any {
                        return Err(format!("Hata: Match ifadesindeki desen tipi ({:?}), kontrol edilen ifadenin tipiyle ({:?}) uyuşmuyor.", pattern_type, discriminant_type));
                    }
                    case_types.push(self.type_of_expr(result)?);
                }

                if case_types.is_empty() {
                    return Ok(Type::Void);
                }

                let first_case_type = case_types[0].clone();
                for case_type in case_types.iter().skip(1) {
                    if *case_type != first_case_type {
                        return Err(format!("Hata: Match ifadesindeki tüm kollar aynı tipi döndürmelidir. Bulunan tipler: {:?}.", case_types));
                    }
                }

                Ok(first_case_type)
            }
            Expr::DefaultCase => {
                Ok(Type::Any) // 'def' durumu her tiple eşleşebilir.
            }
            Expr::InterpolatedString(parts) => {
                // İnterpolasyonlu string içindeki her bir ifadenin tipini kontrol et.
                for part in parts {
                    self.type_of_expr(part)?; // `part` bir `&Expr`
                }
                Ok(Type::Str(None))
            },
            Expr::Lambda { params, return_type, body } => {
                self.push_scope();
                for (param_name, param_type, default_value) in params {
                    if let Some(val) = default_value {
                        let val_type = self.type_of_expr(val)?; // `val` bir `&Expr`
                        if val_type != *param_type {
                            return Err(format!("Hata: Lambda parametresi '{}' için varsayılan değer tipi ({:?}) uyumsuz, beklenen {:?}.", param_name, val_type, param_type));
                        }
                    }
                    let info = VarInfo { ty: param_type.clone(), is_const: false, _is_mutable: false };
                    self.define_variable(param_name.clone(), info)?;
                }
                
                //  Lambda gövdesini kontrol etmeden önce beklenen dönüş tipini geçici olarak ayarla.
                let old_expected_return_type = self.expected_return_type.clone();
                self.expected_return_type = return_type.clone();
                
                let body_type = self.type_of_expr(body)?; // `body` bir `&Expr`

                // Kontrol bittikten sonra eski beklenen dönüş tipini geri yükle.
                self.expected_return_type = old_expected_return_type;

                if body_type != *return_type && *return_type != Type::Any && body_type != Type::Any {
                    return Err(format!("Hata: Lambda gövdesinin tipi ({:?}), beklenen dönüş tipiyle ({:?}) uyuşmuyor.", body_type, return_type));
                }

                self.pop_scope()?;
                let param_types = params.iter().map(|(_, ty, _)| ty.clone()).collect();
                Ok(Type::Fn(param_types, Box::new(return_type.clone())))
            },
            Expr::SizeOf(_) => {
                // sizeof her zaman bir tamsayı boyutu döndürür.
                Ok(Type::U64)
            },
            Expr::EnumAccess { enum_name, variant_name } => {
                //  `::` operatörü bir modül takma adına mı erişiyor?
                if let Some(real_module_name) = self.module_aliases.get(enum_name).cloned() {
                    // Evet, bu bir modül erişimi. Modülü yükle ve öğeyi bul.
                    //  `real_module_name`'i klonladığımız için artık `self`'i mutable olarak ödünç alabiliriz.
                    let module_ast = self.load_module(&real_module_name)?; 
                    if let Decl::Program(declarations) = module_ast {
                        for decl in declarations {
                            if let Decl::Function { name, params, return_type, is_async, is_public, .. } = decl {
                                if name == *variant_name && is_public {
                                    let param_types = params.iter().map(|(_, ty, _)| ty.clone()).collect();
                                    let final_return_type = if is_async { Type::Future(Box::new(return_type)) } else { return_type };
                                    return Ok(Type::Fn(param_types, Box::new(final_return_type)));
                                }
                            }
                        }
                    }
                    return Err(format!("Hata: '{}' modülünde '{}' isminde dışa aktarılmış bir fonksiyon bulunamadı.", real_module_name, variant_name));
                }

                if let Some(variants) = self.enum_definitions.get(enum_name) {
                    if let Some(variant_type) = variants.get(variant_name) {
                        // Enum üyesine erişildiğinde, onun tam Enum(name, base) tipini döndür.
                        // Bu, `var x = Color::Red;` gibi atamalarda x'in tipinin doğru olmasını sağlar.
                        // Karşılaştırmalarda ise (if x == Color::Red), `check_expr` `Color::Red`'in
                        // temel tipini (i32) döndürecektir. Bu mantığı düzeltiyoruz:
                        Ok(variant_type.clone())
                    } else {
                        Err(format!("Hata: '{}' enum'unun '{}' isminde bir üyesi yok.", enum_name, variant_name))
                    }
                } else {
                    Err(format!("Hata: Tanımlanmamış enum tipi: '{}'.", enum_name))
                }
            },
            Expr::StructLiteral { name, fields } => {
                // 1. Struct'ın tanımlı olup olmadığını kontrol et.
                let struct_def = self.struct_definitions.get(name)
                    .ok_or_else(|| format!("Hata: Tanımlanmamış struct tipi: '{}'.", name))?
                    .clone(); //  Borrow checker hatasını çözmek için struct tanımını klonla.

                let mut provided_fields = HashSet::new();

                // 2. Sağlanan her alan için tip kontrolü yap.
                for (field_name, field_expr) in fields {
                    let expected_field_type = struct_def.get(field_name)
                        .ok_or_else(|| format!("Hata: '{}' struct'ının '{}' isminde bir alanı yok.", name, field_name))?;

                    let provided_type = self.type_of_expr(field_expr)?; // `field_expr` bir `&Expr`

                    //  Enum tipi karşılaştırması için özel mantık.
                    let types_match = if let (Type::Custom(expected_name), Type::Enum(provided_name, _)) = (expected_field_type, &provided_type) {
                        // Eğer beklenen tip bir Custom("Status") ise ve sağlanan tip bir Enum("Status", ...) ise,
                        // isimleri eşleşiyorsa bunu geçerli kabul et.
                        expected_name == provided_name
                    } else {
                        // Diğer tüm durumlar için normal karşılaştırma yap.
                        provided_type == *expected_field_type
                    };

                    if !types_match {
                        return Err(format!("Hata: '{}' struct'ının '{}' alanı için tip uyuşmazlığı. Beklenen: {:?}, bulunan: {:?}.", name, field_name, expected_field_type, provided_type));
                    }
                    provided_fields.insert(field_name.clone());
                }

                // 3. Struct'ın tüm alanlarının sağlanıp sağlanmadığını kontrol et.
                for required_field_name in struct_def.keys() {
                    if !provided_fields.contains(required_field_name) {
                        return Err(format!("Hata: '{}' struct'ı oluşturulurken zorunlu olan '{}' alanı eksik.", name, required_field_name));
                    }
                }

                // Her şey yolundaysa, ifadenin tipi bu struct'tır.
                Ok(Type::Custom(name.clone()))
            },
            //  Kanal gönderme ve alma işlemlerinin tip kontrolü
            Expr::Send { channel, value } => {
                let channel_type = self.type_of_expr(channel)?; // `channel` bir `&Expr`
                let value_type = self.type_of_expr(value)?; // `value` bir `&Expr`
                if let Type::Channel(inner_type) = channel_type {
                    if *inner_type != value_type && *inner_type != Type::Any {
                        return Err(format!("Hata: Kanala gönderilen değerin tipi ({:?}), kanalın beklediği tiple ({:?}) uyuşmuyor.", value_type, inner_type));
                    }
                    // Gönderme işlemi bir değer döndürmez.
                    Ok(Type::Void)
                } else {
                    Err(format!("Hata: Gönderme işlemi (<-) sadece kanal tiplerine uygulanabilir, bulundu: {:?}.", channel_type))
                }
            },
            Expr::Recv(channel) => {
                let channel_type = self.type_of_expr(channel)?; // `channel` bir `&Expr`
                if let Type::Channel(inner_type) = channel_type {
                    // Alma işlemi, kanalın içindeki değeri döndürür.
                    Ok(*inner_type)
                } else {
                    Err(format!("Hata: Alma işlemi (<-) sadece kanal tiplerine uygulanabilir, bulundu: {:?}.", channel_type))
                }
            },
        }
    }

    //  Bir tipi, takma ad ise gerçek tipine dönüştürür.
    fn resolve_type(&self, ty: &Type) -> Result<Type, String> {
        if let Type::Custom(name) = ty {
            if let Some(resolved_type) = self.type_aliases.get(name) {
                return self.resolve_type(resolved_type); // Zincirleme takma adları için özyinelemeli çözümle
            }
        }
        Ok(ty.clone())
    }

    fn check_block_stmt(&mut self, block: &Stmt) -> Result<(), String> {
        self.push_scope();
        let check_result = self.check_stmt(block);
        let pop_result = self.pop_scope();
        match check_result {
            Ok(_) => pop_result,
            Err(e) => Err(e),
        }
    }

    //  Kapsam açmadan bir blok deyimini kontrol eden yardımcı fonksiyon.
    // 'for' döngüsü gibi zaten kendi kapsamını yöneten yapılar için kullanılır.
    fn check_block_stmt_no_scope(&mut self, block: &Stmt) -> Result<(), String> {
        if let Stmt::Block(stmts) = block {
            for stmt in stmts {
                self.check_stmt(stmt)?;
            }
        }
        Ok(())
    }
}