// src/main.rs

// Tüm modüller
mod token;
mod lexer;
mod ast;
mod parser;
mod type_checker;
mod codegen; // YENİ: Codegen modülünü ekle

// doğrudan use ifadeleri
use lexer::Lexer;
use parser::Parser;
use token::TokenType;
use std::env;
use std::fs;
use std::process;
use crate::type_checker::TypeChecker;
use crate::ast::{Decl, TargetPlatform}; // YENİ: TargetPlatform'u ast'den al.
use crate::codegen::Codegen; // YENİ: Codegen'i içeri aktar.
use std::process::Command; // YENİ: Dış komutları çalıştırmak için.

// YENİ: Derleme modunu belirten enum.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuildMode {
    Debug,
    Release,
}

// YENİ: Çıktı tipini belirten enum.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputType {
    Executable,
    SharedLibrary,
}

pub struct Config {
    pub include_paths: Vec<String>,
    pub input_file: String,
    pub target_platform: TargetPlatform,
    pub show_help: bool,
    pub build_mode: BuildMode, // YENİ: Derleme modu
    pub output_type: OutputType, // YENİ: Çıktı tipi
}

fn parse_config(args: Vec<String>) -> Result<Config, String> {
    // YENİ: Varsayılan arama yollarına `./libs` eklendi.
    let mut include_paths = vec![".".to_string(), "./libs".to_string()];
    let mut input_file = String::new();

    // 1. Adım: `nim.conf` dosyasını oku (varsa)
    if let Ok(config_content) = fs::read_to_string("nim.conf") {
        for line in config_content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                if key.trim() == "include" {
                    include_paths.push(value.trim().to_string());
                }
            }
        }
    }

    // 2. Adım: Komut satırı argümanlarını ayrıştır (config dosyasını geçersiz kılabilir)
    let mut iter = args.into_iter().skip(1);
    let mut target_platform = TargetPlatform::Unknown;
    let mut show_help = false;
    let mut build_mode = BuildMode::Release;
    let mut output_type = OutputType::Executable; // YENİ: Varsayılan olarak çalıştırılabilir dosya

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-h" | "-help" | "--help" => {
                show_help = true;
                break; // Yardım bayrağı her şeyi geçersiz kılar.
            }
            "--target" => {
                if let Some(target_str) = iter.next() {
                    target_platform = match target_str.to_lowercase().as_str() {
                        "windows" => TargetPlatform::Windows,
                        "linux" => TargetPlatform::Linux,
                        "macos" => TargetPlatform::Macos,
                        _ => return Err(format!("Bilinmeyen hedef platform: '{}'. Geçerli olanlar: windows, linux, macos.", target_str)),
                    };
                } else {
                    return Err("'--target' bayrağı bir platform (windows, linux, macos) bekliyor.".to_string());
                }
            }
            "--mode" => { // YENİ: Derleme modu bayrağı
                if let Some(mode_str) = iter.next() {
                    build_mode = match mode_str.to_lowercase().as_str() {
                        "debug" => BuildMode::Debug,
                        "release" => BuildMode::Release,
                        _ => return Err(format!("Bilinmeyen derleme modu: '{}'. Geçerli olanlar: debug, release.", mode_str)),
                    };
                } else {
                    return Err("'--mode' bayrağı bir mod (debug, release) bekliyor.".to_string());
                }
            }
            "--output-type" => { // YENİ: Çıktı tipi bayrağı
                if let Some(type_str) = iter.next() {
                    output_type = match type_str.to_lowercase().as_str() {
                        "exe" | "executable" => OutputType::Executable,
                        "dll" | "so" | "dylib" | "shared" | "shared-library" => OutputType::SharedLibrary,
                        _ => return Err(format!("Bilinmeyen çıktı tipi: '{}'. Geçerli olanlar: exe, shared.", type_str)),
                    };
                } else {
                    return Err("'--output-type' bayrağı bir tip (exe, shared) bekliyor.".to_string());
                }
            }
            _ if arg.starts_with("-I") => {
                // Hem -I/path hem de -I /path formatlarını destekle
                if arg.len() > 2 {
                    include_paths.push(arg[2..].to_string());
                } else if let Some(path) = iter.next() {
                    include_paths.push(path);
                } else {
                    return Err("'-I' bayrağı bir yol (path) bekliyor.".to_string());
                }
            }
            _ if arg.ends_with(".nim") || arg.ends_with(".n") => {
                if input_file.is_empty() {
                    input_file = arg;
                } else {
                    return Err("Şimdilik sadece tek bir kaynak dosya derlenebilir.".to_string());
                }
            }
            _ => {
                return Err(format!("Bilinmeyen argüman veya bayrak: '{}'", arg));
            }
        }
    }

    // YENİ: Eğer hedef platform belirtilmemişse, derleyicinin çalıştığı platformu varsay.
    if target_platform == TargetPlatform::Unknown {
        target_platform = match env::consts::OS {
            "windows" => TargetPlatform::Windows,
            "linux" => TargetPlatform::Linux,
            "macos" => TargetPlatform::Macos,
            unsupported_os => {
                println!("Uyarı: Bilinmeyen veya desteklenmeyen bir platformda ('{}') çalışılıyor. Platforma özel modüller yüklenirken hata oluşabilir.", unsupported_os);
                TargetPlatform::Unknown
            }
        };
    }

    // Eğer hiç kaynak dosya belirtilmemişse veya yardım istenmişse, yardım göster.
    if input_file.is_empty() {
        show_help = true;
    }

    Ok(Config { include_paths, input_file, target_platform, show_help, build_mode, output_type })
}

// YENİ: Yardım mesajını gösteren fonksiyon.
fn print_help() {
    println!("NIMBLE Derleyici v0.0.1 - Kullanım Kılavuzu");
    println!("----------------------------------------");
    println!("Kullanım: nim <kaynak_dosya.nim> [seçenekler]\n");
    println!("Seçenekler:");
    println!("  -h, -help, --help      Bu yardım mesajını gösterir.");
    println!("  --target <platform>    Derleme hedefini belirtir. Platformlar: windows, linux, macos.");
    println!("  --output-type <type>   Üretilecek çıktı tipini belirtir. Tipler: exe, shared (Varsayılan: exe).");
    println!("  --mode <mode>          Derleme modunu belirtir. Modlar: debug, release (Varsayılan: release).");
    println!("                         (Varsayılan: Çalıştırıldığı sistem)");
    println!("  -I <yol>               Modül arama yollarına ek bir dizin ekler.");
    println!("\nÖrnek:");
    println!("  nim programim.nim --target windows -I ./ek_kutuphaneler");
}

fn main() {
    let config = match parse_config(env::args().collect()) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Yapılandırma Hatası: {}", e);
            process::exit(1);
        }
    };

    // YENİ: Yardım gösterme kontrolü.
    if config.show_help {
        print_help();
        process::exit(0);
    }

    let source_code = fs::read_to_string(&config.input_file).unwrap_or_else(|_| {
        eprintln!("Hata: Dosya okunamadı: {}", &config.input_file);
        process::exit(1);
    });

    println!(">>> NIMBLE Derleyicisi v0.0.1");
    println!(">>> Aşama 1: Lexer (Sözcük Analizi)");
    
    // Lexer
    let mut lexer = Lexer::new(&source_code);
    let mut tokens = Vec::new();
    
    loop {
        let token = lexer.next_token();
        tokens.push(token.clone()); 
        if token.kind == TokenType::Eof { 
            break; 
        }
    }
    //println!("  {} token üretildi.", tokens.len());
    //println!("-------------------------------------\n");

    // Parser
    println!(">>> Aşama 2: Parser (Sözdizimi Analizi)");
    let mut parser = Parser::new(tokens); 
    let (program_root, errors) = parser.parse();

    if !errors.is_empty() {
        println!("\n--- Parser Hataları ---");
        for error in &errors {
            eprintln!("{}", error);
        }
        println!("-----------------------\n");
        eprintln!("Derleme, sözdizimi hataları nedeniyle durduruldu.");
        process::exit(1);
    }
  
    let program_decls: Vec<Decl> = match program_root {
        Decl::Program(decls) => decls, 
        _ => {
            eprintln!("Hata: Parser, Program kök yapısı yerine beklenmeyen bir Decl döndürdü.");
            process::exit(1);
        }
    };
	
    //println!("✅ Parser başarıyla tamamlandı.");
    
    // Testler bitene kadar AST çıktısını geçici olarak devre dışı bırakıyoruz.
    // println!("\n--- Parser Çıktısı (AST) ---\n{:#?}\n---------------------------", program_decls);
    // println!("-------------------------------------\n");
    // Type Checker
    println!(">>> Aşama 3: Semantik Analiz (Tip Kontrolü)");
    let mut type_checker = TypeChecker::new(&program_decls, config.include_paths, config.target_platform);

    match type_checker.check_program() {
        Ok(_) => println!(" "), //println!("✅ Tip Kontrolü Başarılı!"),
        Err(e) => {
            eprintln!("Tip Kontrolü Hatası: {}", e);
            process::exit(1);
        }
    }

    // YENİ: Kod Üretimi Aşaması
    println!("\n>>> Aşama 4: Kod Üretimi (Codegen)");
    let mut codegen = Codegen::new(&program_decls, &mut type_checker, config.target_platform);
    match codegen.generate() {
        Ok(asm_code) => {
            // YENİ: Çıktı dizinlerini oluştur
            let obj_dir = "build/obj";
            let exe_base_dir = match config.build_mode {
                BuildMode::Debug => "build/debug",
                BuildMode::Release => "build/release",
            };

            fs::create_dir_all(obj_dir).expect("Nesne dizini oluşturulamadı.");
            fs::create_dir_all(exe_base_dir).expect("Çalıştırılabilir dizini oluşturulamadı.");

            // Giriş dosyasından temel adı al (örn: "test1.nim" -> "test1")
            let base_name = std::path::Path::new(&config.input_file)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output")
                .to_string();

            // YENİ: Hedef platforma göre dosya uzantılarını ve isimlerini belirle
            let (_platform_suffix, _obj_ext, output_ext) = match config.target_platform {
                TargetPlatform::Windows => ("windows", "obj", if config.output_type == OutputType::Executable { ".exe" } else { ".dll" }),
                TargetPlatform::Linux => ("linux", "o", if config.output_type == OutputType::Executable { "" } else { ".so" }),
                TargetPlatform::Macos => ("macos", "o", if config.output_type == OutputType::Executable { "" } else { ".dylib" }),
                _ => ("unknown", "o", ""), // Bilinmeyen platformlar için varsayılan
            };

            let output_asm_file = format!("{}/{}.s", obj_dir, base_name); // GAS typically uses .s or .asm
            let output_obj_file = format!("{}/{}.o", obj_dir, base_name);
            let output_final_file = format!("{}/{}{}", exe_base_dir, base_name, output_ext);

            fs::write(&output_asm_file, asm_code).expect("Assembly dosyası yazılamadı.");
            println!("✅ GAS (Intel) kodu başarıyla '{}' dosyasına yazıldı.", output_asm_file);

            // 1. AŞAMA: GCC ile Assembly'den Nesne Dosyası (.o) Üretme
            println!("⚙️ GCC ile Assembly derleniyor...");
            let assemble_status = Command::new("gcc")
                .args(&["-x", "assembler", "-c", &output_asm_file, "-o", &output_obj_file])
                .status()
                .expect("GCC (assembler) çalıştırılamadı. GCC'nin sistem PATH'inde olduğundan emin olun.");

            if !assemble_status.success() {
                eprintln!("❌ Assembly derlemesi başarısız oldu.");
                process::exit(1);
            }

            // 2. AŞAMA: Linkleme
            let (linker_cmd, linker_args) = match config.target_platform {
                TargetPlatform::Windows => (
                    "gcc",
                    if config.output_type == OutputType::Executable {
                        vec![output_obj_file.to_string(), "libs/_print.obj".to_string(), "-o".to_string(), output_final_file.to_string()]
                    } else { // SharedLibrary (DLL)
                        vec!["-shared".to_string(), output_obj_file.to_string(), "libs/_print.obj".to_string(), "-o".to_string(), output_final_file.to_string()]
                    }
                ),
                TargetPlatform::Linux => (
                    "gcc",
                    if config.output_type == OutputType::Executable {
                        vec![output_obj_file.to_string(), "-o".to_string(), output_final_file.to_string(), "-no-pie".to_string()]
                    } else { // SharedLibrary (SO)
                        vec!["-shared".to_string(), output_obj_file.to_string(), "-o".to_string(), output_final_file.to_string()]
                    }
                ),
                TargetPlatform::Macos => (
                    "gcc",
                    if config.output_type == OutputType::Executable {
                        vec![output_obj_file.to_string(), "-o".to_string(), output_final_file.to_string()]
                    } else { 
                        vec!["-shared".to_string(), output_obj_file.to_string(), "-o".to_string(), output_final_file.to_string()]
                    }
                ),
                _ => {
                    println!("Uyarı: Bu platform için otomatik derleme ve linkleme desteklenmiyor.");
                    return;
                }
            };
            
            println!("🔗 Linker ile bağlanıyor...");
            let linker_status = Command::new(linker_cmd)
                .args(&linker_args)
                .status()
                .expect("Linker (gcc) çalıştırılamadı.");

            if !linker_status.success() { 
                eprintln!("❌ Linkleme başarısız oldu."); 
                process::exit(1); 
            }
            println!("✅ Başarıyla oluşturuldu: {}", output_final_file);
        }
        Err(e) => {
            eprintln!("Kod Üretimi Hatası: {}", e);
            process::exit(1);
        }
    }
}