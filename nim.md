# NIM (NIMBLE) Programlama Dili Referans Kılavuzu

**Sürüm:** 1.0 (Nihai Referans) 
**Tarih:** 11.12.2025

Bu belge, NIM (NIMBLE) programlama dilinin **eksiksiz** başvuru kaynağıdır. Dilin sunduğu tüm veri tipleri, sözdizimi varyantları, derleyici kuralları ve standart kütüphane fonksiyonları en ince ayrıntısına kadar açıklanmıştır.

---

## İçindekiler

1.  [Giriş ve Felsefe](#1-giriş-ve-felsefe)
2.  [Temel Sözdizimi](#2-temel-sözdizimi)
3.  [Değişkenler ve Veri Tipleri](#3-değişkenler-ve-veri-tipleri)
4.  [Veri Yapıları ve Koleksiyonlar](#4-veri-yapıları-ve-koleksiyonlar)
5.  [Operatörler](#5-operatörler)
6.  [Kontrol Akışı](#6-kontrol-akışı)
7.  [Fonksiyonlar ve Fonksiyonel Programlama](#7-fonksiyonlar-ve-fonksiyonel-programlama)
8.  [Modüler Programlama ve Kütüphaneler](#8-modüler-programlama-ve-kütüphaneler)
9.  [Nesne Yönelimli Programlama (Struct & Group)](#9-nesne-yönelimli-programlama-struct--group)
10. [Bellek ve Sistem Erişimi](#10-bellek-ve-sistem-erişimi)
11. [Standart Kütüphane Referansı](#11-standart-kütüphane-referansı)
12. [Modüllerden Bağımsız Yardımcı Fonksiyonlar](#12-modüllerden-bağımsız-yardımcı-fonksiyonlar-built-ins)
13. [Derleyici Hedefleri ve Optimizasyon](#13-derleyici-hedefleri-ve-optimizasyon-targets)

---

## 1. Giriş ve Felsefe

NIMBLE, kontrolü geliştiriciye veren ancak modern kolaylıkları da sunan hibrit bir dildir.
*   **Tam Kontrol:** Bellek (`ptr`), CPU registerları (`cpu`) ve GPU çekirdeklerine (`gpu`) doğrudan erişim.
*   **Esneklik:** Fonksiyonel (zincirleme, lambda, UFCS) ve OOP (struct-group) paradigması.
*   **Netlik:** Açık modül önek tanımları (`:onek;`) ve okunabilir C-tarzı sözdizimi.

---

## 2. Temel Sözdizimi

### 2.1 Bloklar ve İfadeler
*   **Sonlandırma:** Noktalı virgül (`;`) her ifadenin sonunda zorunludur.
*   **Bloklar:** `{ ... }` süslü parantezleri ile kapsam (scope) oluşturulur.

### 2.2 Yorumlar

```nim
// Bu bir tek satır yorumdur.

/* 
   Bu bir
   çok satırlı (blok) yorumdur.
*/
```

---

## 3. Değişkenler ve Veri Tipleri

NIMBLE, statik tipli bir dildir ancak `any` tipi ile dinamikliği destekler.

### 3.1 Tanımlayıcılar (`var`, `let`, `must`, `const`)

*   **`var`**: Değiştirilebilir (mutable) değişken.
    ```nim
    var sayac: i32 = 0;
    sayac = 1; // Geçerli
    ```
*   **`let`**: Değiştirilemez (immutable) sabit. (Varsayılan olarak).
    ```nim
    let pi: f32 = 3.14;
    // pi = 3.15; // HATA!
    ```
*   **`mut let`**: Değiştirilebilir `let` ( `var` ile benzerdir, stil tercihidir).
*   **`must`**: Derleme zamanında (compile-time) kesinlikle hesaplanması veya mevcut olması gereken değerler için (Assertion gibi).
    ```nim
    must var config = load_config(); // Yüklenemezse derleme durur.
    ```
*   **`const`**: Derleme zamanı sabiti (Compile-time constant).
    ```nim
    const SURUM = "1.0.0";
    ```

### 3.2 Temel Tamsayı Tipleri (Integers)

| Tip   | Bit Genişliği | İşaretli mi? | Değer Aralığı | Yaygın Kullanım |
|---    |---            |---           |---            |---             |
| `i8`  | 8-bit         | Evet         | -128 .. 127   | ASCII Karakter, İşaretli Byte |
| `u8`  | 8-bit         | Hayır        | 0 .. 255      | Ham Veri, Piksel, Byte |
| `i16` | 16-bit        | Evet         | -32,768 .. 32,767 | Ses Örnekleri, Sensör Verisi |
| `u16` | 16-bit        | Hayır        | 0 .. 65,535   | Port Numaraları, Sayaçlar |
| `i32` | 32-bit        | Evet         | -2.14 Milyar .. +2.14 Milyar | Varsayılan Tamsayı |
| `u32` | 32-bit        | Hayır        | 0 .. 4.29 Milyar | Boyutlar, İndeksler |
| `i64` | 64-bit        | Evet         | Çok Geniş | Zaman Damgası (Timestamp), Dosya Boyutu |
| `u64` | 64-bit        | Hayır        | 0 .. 18 Kentilyon | Bellek Adresleri (Pointer) |
| `i128`| 128-bit       | Evet         | Astronomik | Kriptografi, Bilimsel Hesaplama |
| `u128`| 128-bit       | Hayır        | Maksimum | Hash Değerleri (UUID, GUID) |

### 3.3 Ondalıklı Sayılar (Floating Point)

*   `f32`: Tek hassasiyetli (Single precision, 7 basamak). Grafik ve oyun fiziği için.
*   `f64`: Çift hassasiyetli (Double precision, 15 basamak). Varsayılan ondalık tip.
*   `f128`: Dört hassasiyetli (Quad precision). Bilimsel simülasyonlar.

### 3.4 Finansal/Sabit Noktalı Sayılar (Fixed Point)

Para birimi hesaplamalarında yuvarlama hatası olmaması için kullanılır.

*   `d32`: Düşük hassasiyetli finansal değer.
*   `d64`: Standart finansal değer.
*   `d128`: Yüksek hassasiyetli finansal değer.

### 3.5 Özel Donanım ve Sistem Tipleri
*   **`bit`**: Sadece 1-bit alan kaplar (`0` veya `1`). Boolean yerine donanım bayrakları için.
*   **`byte`**: `u8` için bir takma addır (alias).
*   **`hex`**: Onaltılık (Hexadecimal) veri literali. `var adres: hex = 0xFF1A;`
*   **`ptr`**: Tipi belirsiz ham bellek işaretçisi (`void*` karşılığı).
*   **`any`**: Dinamik tip. Çalışma zamanında her türlü veriyi tutabilir.

---

## 4. Veri Yapıları ve Koleksiyonlar

### 4.1 Diziler (Arrays)

NIMBLE'da diziler köşeli parantez `[]` ile tanımlanır ve başlatılır.

**Statik (Sabit Boyutlu) Diziler:**
Stack belleğinde tutulur. Boyut derleme zamanında bilinmelidir.

```nim
var sayilar[5]: i32 = [1, 2, 3, 4, 5];
var matris[3][3]: i32 = [
    [1, 0, 0],
    [0, 1, 0],
    [0, 0, 1]
];
```

**Dinamik (Büyüyebilir) Diziler:**
Heap belleğinde tutulur. `T[]` sözdizimi kullanılır.

```nim
var liste: i32[] = [10, 20, 30];
liste.push(40); // Sona ekle
```

**Fonksiyonlar:** (Bkz: `array` Standart Modülü)
*   `push`, `pop`, `count`, `clear`, `find`, `sort`, `reverse`.

### 4.2 Haritalar (Maps / Dictionaries)

Anahtar-Değer (Key-Value) çiftlerini tutar. `map<AnahtarTipi, DegerTipi>`

```nim
var notlar: map<str, i32>;
notlar["Ali"] = 90;
notlar["Veli"] = 85;

var notu: i32 = notlar["Ali"]; // 90
```

### 4.3 Matematiksel Vektörler
Oyun ve grafik programlama için SIMD destekli tipler.

*   `Vec2`: `{x, y}`
*   `Vec3`: `{x, y, z}`
*   `Vec4`: `{x, y, z, w}`

```nim
var v1 = Vec3{1.0, 2.0, 3.0};
var v2 = Vec3{4.0, 5.0, 6.0};
var sonuc = v1 + v2; // {5.0, 7.0, 9.0} otomatik toplanır.
```

---

## 5. Operatörler

*   **Aritmetik:** `+`, `-`, `*`, `/` (bölme), `%` (mod), `++` (artırma), `--` (azaltma).
*   **Mantıksal:** `&&` (VE), `||` (VEYA), `!` (DEĞİL).
*   **Bitwise (Bit Düzeyi):** `&` (VE), `|` (VEYA), `^` (XOR), `~` (TÜMLEYEN), `<<` (SOLA KAYDIR), `>>` (SAĞA KAYDIR).
*   **Karşılaştırma:**
    *   `==`: Değer eşitliği.
    *   `===`: Kimlik (Referans/Adres) eşitliği.
    *   `!=`: Eşit değil.
    *   `!==`: Aynı nesne değil.
    *   `<`, `>`, `<=`, `>=`.
*   **Erişim:**
    *   `.`: Üye erişimi (`insan.ad`).
    *   `.`: Statik/Modül erişimi (`network.http.get`). //group fonksiyonun yapısı gereği üye erişimi . ile yapılır.

---

## 6. Kontrol Akışı

### 6.1 `if - elseif - else`

```nim
var x = 10;
if (x > 100) {
    print("Büyük");
} elseif (x == 10) {
    print("Eşit");
} else {
    print("Küçük");
}
```

### 6.2 `match` (Desen Eşleştirme)

NIMBLE'ın en güçlü özelliklerinden biridir. 4 farklı kullanım varyantı vardır.

**1. Basit Eşleşme:**
```nim
match (durum) {
    1 => {print("Aktif")},
    0 => {print("Pasif")},
    def => {print("Bilinmiyor")} // Varsayılan (default)
}
```

**2. Çoklu Seçim:**
Birden fazla değeri tek satırda eşleştirme.
```nim
match (sayi) {
    1, 3, 5, 7, 9 => {print("Tek Sayı")},
    2, 4, 6, 8, 0 => {print("Çift Sayı")}
}
```

**3. İfade Olarak (Expression Match):**
Sonucu bir değişkene atama.
```nim
var mesaj = match (kod) {
    200 => "Başarılı",
    404 => "Bulunamadı",
    500 => "Sunucu Hatası",
    def => "Bilinmeyen Hata"
};
```

**4. `Result` ve `Option` Tipleri ile:**
Hata yönetiminde kullanılır.
```nim
var dosya = file.open("test.txt", file.READ);
match (dosya) {
    Ok(h) => h.read_all(), // h: FileHandle
    Err(e) => print("Hata: " + e)
}
```

### 6.3 Döngüler

**1. C-Tarzı For Döngüsü:**
Geleneksel, virgül ile ayrılmış yapı.
```nim
for (i=0, i<10, i++) {
    print(i);
}
```

**2. Aralık (Range) Döngüsü:**
`baslangic..bitis` (bitiş dahil).
```nim
// i burada otomatik "i:i32=0" olarak tanımlanır. 
// range (A..Z), (a..z),(0..999) gibi tanımlana bilir.
// hex lerdekullanılabilir (0x0..0xFF) gibi yada bellek taraması yapabilir.
for (i in 0..10) {
    print(i); // 0, 1, ..., 10
}
```

**3. Koleksiyon Döngüsü (Foreach):**
```nim
var isimler = ["Ali", "Veli"];
for (isim in isimler) {
    print(isim);
}
```

**4. While Döngüsü:**
```nim
while (kosul) {
    // ...
}
```

**5. Sonsuz Döngü (`loop`):**
```nim
loop {
    if (bitir) break;
}
```

### 6.4 `rolling` (Hata Toleranslı Blok)

Bir kod bloğunun hata vermesi durumunda, belirli bir sayıda tekrar denenmesini sağlayan mekanizmadır. `$rolling` özel değişkeni mevcut deneme sayısını tutar.

```nim
// Blok adı: BAGLANTI
rolling:BAGLANTI => {
    // $rolling değişkeni otomatik tanımlanır ve 0'dan başlar.
    
    print("Bağlanmaya çalışılıyor... Deneme: {$rolling}");
    
    var conn = network.connect("sunucu.com");
    
    if (conn.hata && $rolling < 5) {
        time.sleep(1000); // 1 saniye bekle
        rolling:BAGLANTI; // Bloğu başa sar (goto gibi ama güvenli)
    }
}
```

---

## 7. Fonksiyonlar ve Fonksiyonel Programlama

### 7.1 Tanımlama Biçimleri

**Standart Tanım:**
```nim
fn topla(a: i32, b: i32): i32 {
    return a + b;
}
```

**Void Fonksiyon (Prosedür):**
Dönüş tipi belirtilmezse `void` kabul edilir.
```nim
fn selamla(isim: str) {
    print("Merhaba " + isim);
}
```

**Expression Body (Tek Satır):**
Süslü parantez yerine `->` kullanılır.
```nim
fn kare(x: i32) -> x * x;
```

**GPU Kernel Fonksiyonu (`gpu`):**
Ekran kartı üzerinde paralel çalışacak fonksiyonlar.
```nim
gpu fn piksel_is(veri: u8[]) {
    // GPU kodu...
}
```

### 7.2 Parametre Özellikleri

*   **Varsayılan Değer:** `fn topla(a: i32, b: i32 = 0)`
*   **İsimli Çağrı:** `topla(a: 5, b: 10)` veya `topla(b: 10, a: 5)`
*   **Değişken Sayıda Parametre (Varargs):** `...`
    ```nim
    fn log(mesajlar: str...) { ... }
    log("Hata", "Dosya", "Yok");
    ```

### 7.3 UFCS (Uniform Function Call Syntax)
Fonksiyonların, ilk parametrelerinin bir metoduymuş gibi çağrılabilmesini sağlar. Bu, zincirleme (chaining) yapmayı kolaylaştırır.

```nim
// Standart çağrı:
string.to_upper(string.trim("  nim  "));

// UFCS ile çağrı (Okunabilir):
"  nim  ".trim().to_upper();
```

### 7.4 Higher-Order Functions (HOF)
Fonksiyonları parametre olarak alabilen veya döndürebilen fonksiyonlar.

```nim
fn islem_yap(dizi: i32[], operasyon: fn(i32): i32) {
    for (i in dizi) {
        operasyon(i);
    }
}
```

---

## 8. Modüler Programlama ve Kütüphaneler

### 8.1 Modül Tanımlama (`:onek;`)
Her modül dosyasının (`.nim`) en başında, o modülün diğer dosyalarda hangi önek (prefix) ile çağrılacağını belirleyen özel bir tanımlama bulunur.

**Örnek Dosya: `network.nim`**
```nim
:net; /* ön ek olarak kullanır.
:;  veya hiçbir önek almadan kullanılır. bu şekilde sadece group (http) adını kullanır. */
export group http { ... }
```

### 8.2 Erişim Belirteçleri (`export`)
Varsayılan olarak her şey **gizlidir (private)**. Dışarı açmak için `export` veya `pub` kullanılır.

```nim
var gizli: i32 = 0;
export var acik: i32 = 1;

export fn genel_fonk() { ... }
```

### 8.3 Kullanım (`use`)
```nim
use network;       // network.nim dosyasını yükle.
// Dosya başında :net; tanımlı olduğu için:
net.http.get(...); 

use math as m;     // math modülünü 'm' olarak takma adla yükle.
m.sin(30);
```

---

## 9. Nesne Yönelimli Programlama (Struct & Group)

NIMBLE, veriyi (`struct`) ve davranışı (`group`) birbirinden kesin olarak ayırır.

### 9.1 Struct (Veri Yapısı)
Sadece alanları (field) tutar. Metot içermez.

```nim
struct Oyuncu {
    ad: str,
    puan: i32,
    aktif: bool
}
```

### 9.2 Group (Davranış Grubu)
Fonksiyonları (metotları) gruplar. 

**KURAL:** Bir `group`, bir `struct`'a metot ekleyecekse, isimleri **BİREBİR AYNI** olmalıdır.

```nim
group Oyuncu {
    /* struct group içerisinde tanımlanır ise dışarıdan direkt olarak erişilemez 
        sadece group metotları kullanılarak ulaşılabilir */
    struct Oyuncu {  
        ad: str,
        puan: i32,
        aktif: bool
    }

    // Yapıcı (Constructor) - Statik Fonksiyon
    yeni => fn(isim: str): Oyuncu {
        return Oyuncu { 
            ad: isim, 
            puan: 0, 
            aktif: true 
        };
    }

    // Nesne Metodu
    // İlk parametre 'self' olmak zorundadır. Tipi (Oyuncu) belirtilmez (infer edilir).
    puan_arttir => fn(self, miktar: i32) {
        self.puan = self.puan + miktar;
    }
    
    // Tek satır metot
    isim_getir => fn(self) -> self.ad;
}

// Kullanım:
var o1 = Oyuncu.yeni("Ahmet");
o1.puan_arttir(10);
print(o1.isim_getir()); // Ahmet
```

---

## 10. Bellek ve Sistem Erişimi

### 10.1 `memory` Modülü: Manuel Yönetim
NIMBLE normalde otomatik bellek yönetimi kullanır. Ancak `memory` modülü ile C-tarzı manuel yönetim mümkündür. İleri düzey kullanıcılar içindir.

> **Önemli Not:** Derleyici, `memory` modülü kullanıldığında bile herhangi bir hatayı önlemek için ayrılan hafızayı ve veri tiplerini arka planda kontrol eder. Aşırı uyumsuzluk veya taşma durumunda derleme zamanında veya çalışma zamanında hata üretir. Bu sayede manuel yönetimde bile bellek sızıntılarının ve kritik hataların mümkün olduğunca önüne geçilir.

```nim
use memory;

// 1024 byte yer ayır
var ptr = memory.alloc(1024);

// Belleği kullan (unsafe)
memory.set(ptr, 0, 1024); // Sıfırla

// Serbest bırak
memory.free(ptr);

// Veri Okuma/Yazma (Dereferencing)
memory.write<i32>(ptr, 0, 100);      // ptr[0] = 100
var val = memory.read<i32>(ptr, 0);  // val = ptr[0]

// Blok İşlemleri
memory.copy(src_ptr, dest_ptr, 1024); // memcpy
memory.move(src_ptr, dest_ptr, 1024); // memmove (güvenli)
```

**Temel Fonksiyonlar:**
*   `alloc(size) -> ptr`
*   `realloc(ptr, new_size) -> ptr`
*   `free(ptr)`
*   `set(ptr, val_u8, size)`: Belirtilen alanı bayt ile doldurur (memset).
*   `read<T>(ptr, offset)`: Adresten değer okur.
*   `write<T>(ptr, offset, val)`: Adrese değer yazar.
*   `copy(src_ptr, dest_ptr, 1024)`: memcpy
*   `move(src_ptr, dest_ptr, 1024)`: memmove (güvenli)

### 10.2 `asm` ve `fastexec`:
fastexec, asm kodları için kapsayıcı, değişken geçişlerinin otomatik olarak yapıldığı ve optimize edici bir bloktur.
Inline Assembly kodları için CPU register kontrolü.

```nim

fn Calc():void {
	// agressif optimizasyon uygulanır.
	fastexec {
		var a:i32 = 10;
		var b:i32 = 20;
		var total:i32;
		// asm blockları fastexec kapsamında değil ise hata üretilir.
        // asm blokları başka asm bloklara, asm nin normal etiket çağrıları gibi çağrı yapabilir.
		asm: CRITICAL_ADD { 
			mov rax, %a 
			add rax, %b 
			mov %total, rax 
		} 
		echo(total)
	}
}

```

### 10.3 `cpu` Modülü: İşlemci Kontrolü
**Gereksinim:** `cpu` modülü fonksiyonları **sadece `fastexec` bloğu içerisinde** çalışır. Bu kapsam dışında çağrılırlarsa derleyici hata üretir.

Bu modül, çekirdekler, registerlar (kaydediciler) ve düşük seviyeli işlemci komutları ile doğrudan etkileşim kurmayı sağlar.

*   `core_count() -> i32`: Sistemdeki mantıksal çekirdek sayısını döndürür.
*   `reg_get(reg_id) -> u64`: Belirtilen register'ın (örn: `cpu.RAX`) değerini okur.
*   `reg_set(reg_id, val)`: Register'a değer yazar. **Dikkatli kullanılmalıdır.**
*   `pause()`: İşlemciye "duraklama" (spin-loop hint) sinyali gönderir. Enerji tasarrufu ve HT performansı için döngülerde kullanılır.
*   `clflush(ptr)`: Belirtilen adresi CPU önbelleğinden (cache) temizler.
*   `rdtsc() -> u64`: Time-Stamp Counter değerini okur (Çok hassas zamanlama için).
*   `get_freq() -> i32`: Anlık CPU frekansını (MHz) döndürür.

**Kullanım Örneği:**

```nim
fn LowLevelOp() {
    fastexec {
        // Çekirdek sayısını al
        var cores = cpu.core_count();
        
        // Hassas zaman ölçümü (Cycle cinsinden)
        var start = cpu.rdtsc();
        
        asm: ISLEM {
            mov rax, 100
            // ...
        }
        
        var end = cpu.rdtsc();
        var cycles = end - start;
        
        // Spin-wait döngüsü örneği
        while (mesgul_mu) {
            cpu.pause(); // CPU'yu rahatlat
        }
    }
    // cpu.rdtsc(); // HATA! fastexec dışında kullanılamaz.
}
```

---

## 11. Standart Kütüphane Referansı

Bu bölüm, NIMBLE standart kütüphanesindeki modüllerin tam listesidir.

### 11.1 `io` Modülü
**Açıklama:** Standart Giriş/Çıkış işlemleri.

*   `fn print(val: any): void` - Ekrana yazar (yeni satır yok).
*   `fn println(val: any): void` - Ekrana yazar ve satır atlar.
*   `fn eprint(val: any): void` - Hata akışına (stderr) yazar.
*   `fn prompt(msg: str): str` - Mesaj gösterip girdi ister.
*   `fn input(): str` - Klavyeden veri okur.
*   `fn flush(): void` - Çıktı tamponunu temizler.

**Örnek:**
```nim
var ad = io.prompt("Adınız: ");
io.println("Merhaba {ad}");
```

### 11.2 `string` Modülü
**Açıklama:** Metin işleme fonksiyonları.
// :str; 

*   `len(s)`: Karakter sayısı. /str.len() / modülden bağımsız strlen(string) ile aynı işi yapar.
*   `is_empty(s)`: Boş kontrolü.
*   `to_upper(s)`, `to_lower(s)`: Dönüşüm.
*   `trim(s)`: Boşlukları temizle.
*   `split(s, ayirici)`: Böl ve dizi döndür.
*   `join(dizi, birlestirici)`: Birleştir.
*   `contains(s, aranan)`, `starts_with`, `ends_with`.
*   `replace(s, eski, yeni)`.
*   `substring(s, bas, uzunluk)`.

**Örnek:**
```nim
var s = "  merhaba dünya  ";
var temiz = s.trim().to_upper(); // "MERHABA DÜNYA"
if temiz.contains("DÜNYA") { ... }
```

### 11.3 `array` Modülü
**Açıklama:** Dinamik dizi araçları.

*   `count(arr)`: Eleman sayısı.
*   `capacity(arr)`: Ayrılan bellek kapasitesi.
*   `push(arr, val)`: Sona ekle.
*   `pop(arr)`: Sondan çıkar (`Result` döner).
*   `insert(arr, index, val)`: Araya ekle.
*   `remove(arr, index)`: Sil.
*   `find(arr, val)`: Ara (`Option<index>` döner).
*   `sort(arr)`: Sırala.
*   `reverse(arr)`: Ters çevir.
*   `clear(arr)`: Temizle.
*   `new_capacity(size)`: Optimize dizi oluştur.

**Örnek:**
```nim
var sayilar: i32[] = [3, 1, 4];
sayilar.sort(); // [1, 3, 4]
sayilar.push(2);
sayilar.reverse(); // [2, 4, 3, 1]
sayilar.sort(); // [1, 2, 3, 4]
sayilar.clear(); // []
sayilar.new_capacity(10); // [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
```

### 11.4 `math` Modülü
**Açıklama:** Matematiksel fonksiyonlar ve sabitler (`f64` üzerinde çalışır).

*   **Sabitler:** `PI`, `E`, `INF`, `NAN`.
*   **Temel:** `abs`, `pow(x, y)`, `sqrt`.
*   **Logaritmik:** `exp`, `log` (ln), `log10`.
*   **Trigonometrik:** `sin`, `cos`, `tan`, `atan2`.
*   **Dönüşüm:** `deg_to_rad`, `rad_to_deg`.
*   **Yuvarlama:** `round`, `floor`, `ceil`, `trunc`.

**Örnek:**
```nim
var yaricap = 5.0;
var alan = math.PI * math.pow(yaricap, 2.0);
var aci_rad = math.deg_to_rad(45.0);
```

### 11.5 `file` Modülü
**Açıklama:** Dosya sistemi işlemleri.

*   `open(path, mode) -> Result<Handle, Error>`
*   Modlar: `READ`, `WRITE`, `APPEND`, `READ_WRITE`.
*   `read_all(handle) -> str`
*   `write(handle, data)`
*   `close(handle)`
*   `exists(path) -> bool`
*   `delete(path)`
*   `copy(src, dest)`

**Örnek:**
```nim
match (file.open("not.txt", file.WRITE)) {
    Ok(h) => {
        file.write(h, "Test verisi");
        file.close(h);
    },
    Err(e) => io.eprint("Hata: {e}")
}
```

### 11.6 `network` Modülü
**Önek:** `:net;` (Kullanımda `net` olarak çağrılır.)

Bu modül, tüm ağ operasyonlarını üç ana grup altında toplar.

#### ** Grup: `net.http` (Web İstemci ve Sunucu)**

*   `get(url) -> Result<Response, Error>`
*   `post(url, body) -> Result<Response, Error>`
*   `server(port, handler_fn)`: Basit HTTP sunucusu başlatır.

**Örnek (İstemci):**
```nim
var cevap = net.http.get("https://api.ornek.com/veri");
if cevap.is_ok() {
    print(cevap.unwrap().body);
}
```

**Örnek (Sunucu):**
```nim
// 8080 portunda sunucu başlat
net.http.server(8080, fn(req) {
    if (req.path == "/selam") return "Merhaba Dünya";
    return 404; // Not Found
});
```

#### ** Grup: `net.socket` (Düşük Seviye Soket)**

*   `create(type)`: `TCP` veya `UDP` soketi oluşturur.
*   `bind(sock, addr, port)`
*   `listen(sock)`
*   `accept(sock)`
*   `connect(sock, addr, port)`
*   `send(sock, data)`
*   `recv(sock, size)`
*   `close(sock)`

**Örnek (Raw Socket):**
```nim
var s = net.socket.create(net.TCP);
net.socket.connect(s, "127.0.0.1", 9000);
net.socket.send(s, "Ping");
```

#### ** Grup: `net.tcp` (TCP Yardımcıları)**

*   `connect(host, port) -> Stream`: Hızlı bağlantı.
*   `listen(port, handler)`: Hızlı sunucu.

```nim
var stream = net.tcp.connect("google.com", 80);
stream.write("GET / HTTP/1.1\r\n\r\n");
```

### 11.7 `database` Modülü
**Önek:** `:db;`

Veritabanı işlemleri için `sqlite` ve `nosql` gruplarını içerir.

#### ** Grup: `db.sqlite` (İlişkisel Veritabanı)**

*   `connect(path) -> Connection`
*   `exec(conn, sql)`: SQL komutu çalıştır (CREATE, INSERT).
*   `query(conn, sql) -> Rows`: Veri sorgula (SELECT).
*   `close(conn)`

**Örnek:**
```nim
var con = db.sqlite.connect("kullanicilar.db");

// Tablo oluştur
db.sqlite.exec(con, "CREATE TABLE IF NOT EXISTS user (id INT, name TEXT)");

// Veri ekle
db.sqlite.exec(con, "INSERT INTO user VALUES (1, 'Ahmet')");

// Sorgula
var satirlar = db.sqlite.query(con, "SELECT * FROM user");
for (satir in satirlar) {
    print("Kullanıcı: {satir["name"]}");
}

db.sqlite.close(con);
```

#### ** Grup: `db.nosql` (Anahtar-Değer Deposu)**

Basit, gömülü (embedded) bir KV (Key-Value) veritabanı.

*   `open(path) -> Store`
*   `set(store, key, value)`
*   `get(store, key) -> Option<val>`
*   `del(store, key)`

**Örnek:**
```nim
var store = db.nosql.open("cache.dat");

db.nosql.set(store, "session_123", "{user_id: 5}");

var val = db.nosql.get(store, "session_123");
if val.is_some() {
    print("Oturum Verisi: {val.unwrap()}");
}
```

### 11.8 `os` Modülü
**Açıklama:** İşletim sistemine özgü işlemler.

*   `args()`: Komut satırı argümanlarını dizi olarak verir.
*   `get_env(key)`, `set_env(key, val)`.
*   `cwd()`: Mevcut çalışma dizini.
*   `chdir(path)`, `mkdir(path)`, `rmdir(path)`.
*   `execute(cmd)`: Sistem komutu çalıştır.
*   `exit(code)`: Programdan çık.

**Örnek:**
```nim
var dosyalar = os.execute("ls -la");
var kullanici = os.get_env("USER").unwrap_or("Misafir");
```

### 11.9 `crypto` Modülü
**Açıklama:** Kriptografik işlemler.

*   `hash(algo, data)`: SHA256, MD5 vb.
*   `hmac(key, data)`.
*   `generate_key()`.
*   `encrypt(algo, key, data)`.
*   `decrypt(algo, key, data)`.

**Örnek:**
```nim
var sifre_hash = crypto.hash(crypto.SHA256, "gizlisifre123");
print("Hash: {sifre_hash}");
```

### 11.10 `time` Modülü
**Açıklama:** Zaman ve tarih.

*   `now()`: Unix timestamp.
*   `sleep(ms)`: Beklet.
*   `utc_now()`, `local_now()`: DateTime yapısı döner.
*   `format(dt, fmt_str)`.
*   `measure_start()`, `measure_end()`: Performans ölçümü.

**Örnek:**
```nim
var basla = time.measure_start();
time.sleep(500); // 500ms uyu
var sure = time.measure_end(basla);
print("Geçen süre: {sure}");
```

### 11.11 `rand` Modülü
**Açıklama:** Rastgele sayı üretimi.

*   `new_rng()`: Standart üreteç.
*   `new_secure()`: Kriptografik güvenli üreteç.
*   `range_i32(rng, min, max)`.
*   `f64(rng)`: 0.0 - 1.0 arası.
*   `choice(rng, dizi)`.

**Örnek:**
```nim
var rng = rand.new_rng();
var zar = rand.range_i32(rng, 1, 7); // 1-6
```

### 11.12 `regex` Modülü
**Açıklama:** Düzenli ifadeler.

*   `compile(pattern)`.
*   `match(regex, text)`, `find(regex, text)`.
*   `replace(regex, text, new)`.
*   `Match` yapısı: `grp(i)`, `start()`, `end()`.

**Örnek:**
```nim
var re = regex.compile("\\d+").unwrap(); // Sayıları bul
var sonuc = re.find("Sipariş No: 12345");
if sonuc.is_some() {
    print("Numara: {sonuc.unwrap().text()}"); // 12345
}
```

### 11.13 `json` Modülü
**Açıklama:** JSON ayrıştırma ve oluşturma.

*   `parse(str) -> JsonValue`.
*   `stringify(val) -> str`.
*   `JsonValue` metotları: `get(key)`, `at(index)`, `as_str()`, `as_i32()`.

**Örnek:**
```nim
var veri = json.parse("{\"id\": 1, \"aktif\": true}").unwrap();
var id = veri.get("id").as_i32().unwrap();
```

### 11.14 `log` Modülü
**Açıklama:** Loglama sistemi.

*   Seviyeler: `DEBUG`, `INFO`, `WARN`, `ERROR`, `FATAL`.
*   Fonksiyonlar: `log.info(msg)`, `log.error(msg)` vb.
*   `set_level(lvl)`, `add_target(file)`.

**Örnek:**
```nim
log.set_level(log.WARN);
log.info("Bu görünmez");
log.error("Bu görünür");
```

### 11.15 `gpu` Modülü
**Açıklama:** GPU programlama arayüzü.

*   `select_device(id)`.
*   `compile_kernel(fn)`.
*   `alloc_array(dev, size)`, `to_gpu(arr)`, `from_gpu(arr)`.
*   `launch(kernel, grid, block, args...)`.
*   `sync(dev)`.

**GPU Kernel Fonksiyonu (`gpu`):**
Ekran kartı üzerinde paralel çalışacak fonksiyonlar.
```nim
gpu fn piksel_is(veri: u8[]) {
    // GPU kodu...
}
```

### 11.16 `thread` Modülü
**Açıklama:** İş parçacığı yönetimi.

*   `spawn(fn)`: Yeni thread.
*   `join(handle)`: Bekle.
*   `Mutex`, `Semaphore`, `Channel` (Mesajlaşma kanalları).

**Örnek:**
```nim
thread.spawn(fn() {
    print("Arka planda çalışıyor...");
});
```

### 11.17 `ffi` Modülü (Foreign Function Interface)
**Açıklama:** C/C++ kütüphaneleriyle bağlantı.

*   `extern fn`: Dış fonksiyon tanımlama.
*   `#[link(name="lib")]`: Kütüphane bağlama.

**Örnek:**
```nim
#[link(name="m")]
extern fn sqrt(x: f64): f64;
// Kullanım: sqrt(16.0);
```

### 11.18 `error` Modülü
**Açıklama:** Hata türleri ve yönetimi. `Result` ve `Option` temel yapıtaşlarıdır.

*   `panic(msg)`: Programı durdur.
*   `unwrap()`, `unwrap_or(default)`.

### 11.19 `types` Modülü
**Açıklama:** İleri seviye tip dönüşümü, tip kontrolü ve güvenli/güvensiz dönüşüm araçları. Yerleşik `int()`/`float()` fonksiyonlarından daha detaylı kontrol sağlar.

*   **Güvenli Dönüşümler (Checked Casts):**
    *   `to_i32(val) -> Result<i32, Error>`: Hata kontrollü dönüşüm.
    *   `to_f64(val) -> Result<f64, Error>`
    *   `to_str(val) -> str`
    *   `parse<T>(str) -> Result<T, Error>`: Genel ayrıştırma.
*   **Tip Kontrolü (Runtime Type Info):**
    *   `is_type<T>(val) -> bool`: Değişkenin tipi o mu?
    *   `name_of(val) -> str`: Tip adını döndürür.
    *   `size_of<T>() -> i32`: Tipin boyutu.
*   **Güvensiz Dönüşümler (Unsafe Casts):**
    *   `as_type<T>(val)`: Zorla dönüştür (Veri kaybı olabilir).
    *   `cast<T>(val)`: Bitwise Reinterpret Cast (Ham bellek yorumlaması).

**Örnek:**
```nim
var s = "123";
if types.is_type<str>(s) {
    var num = types.parse<i32>(s).unwrap();
}

// Bitwise dönüşüm (float -> int bitleri)
var f = 1.0;
var bits = types.cast<i32>(f);
```

### 11.20 `generics` (Jenerikler)
Dilin çekirdek özelliğidir. `<T>` sözdizimi ile kullanılır.

```nim
fn kutu_yap<T>(deger: T) { ... }
```

### 11.21 `linalg` Modülü (Lineer Cebir & Vektör)
**Açıklama:** Oyun, fizik ve grafik programlama için Vektör ve Matris aritmetiği. `Vec2`, `Vec3`, `Vec4` tipleri üzerinde çalışır.

*   `dot(v1, v2)`: Nokta çarpımı (Skaler sonuç).
*   `cross(v1, v2)`: Çapraz çarpım (`Vec3`).
*   `normalize(v)`: Birim vektör yapar (Uzunluk = 1).
*   `access(v)`: Uzunluk (magnitude) hesaplar.
*   `distance(v1, v2)`: İki nokta arası mesafe.
*   `lerp(v1, v2, t)`: Lineer enterpolasyon (t: 0.0 - 1.0).
*   `rotate(v, axis, angle)`: Vektörü döndürür.

**Örnek:**
```nim
var v = Vec3{0, 1, 0};
var yon = linalg.rotate(v, Vec3{0, 0, 1}, 90.0);
```

### 11.22 `collections` Modülü (Gelişmiş Veri Yapıları)
**Açıklama:** Standart `array` ve `map` dışındaki veri yapıları.

*   `Stack<T>`: LIFO (Son giren ilk çıkar) yığını. `push`, `pop`, `peek`.
*   `Queue<T>`: FIFO (İlk giren ilk çıkar) kuyruğu. `enqueue`, `dequeue`.
*   `Set<T>`: Eşsiz eleman kümesi. `add`, `contains`, `remove`, `intersect` (kesişim), `union` (birleşim).
*   `LinkedList<T>`: Çift yönlü bağlı liste.

**Örnek:**
```nim
var küme = collections.Set<i32>();
küme.add(1);
küme.add(1); // Eklenmez (Zaten var)
```

### 11.23 `path` Modülü (Dosya Yolu İşlemleri)
**Açıklama:** Dosya yollarını işletim sisteminden bağımsız (Windows/Linux) yönetir.

*   `join(parts...)`: Yolları birleştirir (`/` veya `\` otomatik).
*   `ext(path)`: Dosya uzantısını verir.
*   `base(path)`: Dosya adını verir.
*   `dir(path)`: Klasör yolunu verir.
*   `abs(path)`: Mutlak (absolute) yolu verir.

**Örnek:**
```nim
var tam_yol = path.join(os.cwd(), "veri", "resim.png");
```

### 11.24 `process` Modülü (Süreç Yönetimi)
**Açıklama:** Harici programları çalıştırma ve yönetme.

*   `spawn(cmd, args) -> Pid`: Program başlatır.
*   `exec(cmd) -> Output`: Çalıştır ve çıktısını al.
*   `kill(pid)`: Süreci sonlandırır.
*   `wait(pid)`: Bitmesini bekle.
*   `pipe()`: Veri akışı oluştur.

**Örnek:**
```nim
var cikti = process.exec("git status").stdout;
echo(cikti);
```

### 11.25 `encoding` Modülü
**Açıklama:** Veri kodlama ve sıkıştırma.

*   `base64_encode(data)`, `base64_decode(str)`.
*   `hex_encode(data)`, `hex_decode(str)`.
*   `compress(data, algo)`, `decompress(data, algo)`: (Gzip, Zlib).

### 11.26 `test` Modülü (Birim Test)
**Açıklama:** Yazılım kalitesi için test çerçevesi.

*   `assert_eq(a, b)`, `assert_ne(a, b)`.
*   `assert_true(cond)`.
*   `run_tests()`: Tüm testleri çalıştırır.

```nim
fn test_toplama() {
    test.assert_eq(2 + 2, 4);
}
```


### 11.27 `data` Modülü (Veri Manipülasyonu)
**Açıklama:** Python benzeri gelişmiş liste ve veri işleme fonksiyonları. Veri analizi ve manipülasyonu için kullanılır.

*   `slice(arr, start, end) -> Arr`: Dizinin bir kısmını kopyalar (`arr[start:end]` benzeri).
*   `map(arr, fn) -> Arr`: Dizideki her elemana fonksiyonu uygular ve yeni dizi döner.
*   `filter(arr, fn) -> Arr`: Koşulu sağlayan elemanları filtreler.
*   `reduce(arr, fn, init) -> val`: Diziyi tek bir değere indirger.
*   `zip(arr1, arr2) -> Arr`: İki diziyi fermuar gibi birleştirir `[[a1, b1], [a2, b2]...]`.
*   `enumerate(arr) -> Arr`: İndeks ve değeri çift olarak döner `[[0, val0], [1, val1]...]`.
*   `range(start, stop, step) -> Arr`: Sayı dizisi oluşturur.
*   `any(arr, fn) -> bool`: En az bir eleman koşulu sağlıyor mu?
*   `all(arr, fn) -> bool`: Tüm elemanlar koşulu sağlıyor mu?
*   `sum(arr) -> val`: Sayısal diziyi toplar.

**Örnek:**
```nim
var sayilar = [1, 2, 3, 4, 5];
var kareler = data.map(sayilar, fn(x) -> x * x); // [1, 4, 9, 16, 25]
var ciftler = data.filter(kareler, fn(x) -> x % 2 == 0); // [4, 16]
var toplam = data.sum(ciftler); // 20
```

### 11.28 `tpu` Modülü (Tensör İşlemciler)
**Açıklama:** Yapay Zeka ve Makine Öğrenimi (ML) için özelleşmiş Tensor Processing Unit (TPU) veya NPU (Neural Processing Unit) donanımlarını hedefler. Büyük matris çarpımları ve tensör operasyonları için optimize edilmiştir.

**1. TPU Kernel (`tpu fn`):**
TPU üzerinde çalışacak kod blokları. Genellikle matris tabanlı tekil talimatlar (SIMD/MIMD) içerir.
```nim
tpu fn matris_carp(a: tensor<f32>, b: tensor<f32>) {
    // TPU'ya özel yüksek hızlı matris çarpımı
    ...
}
```

**2. Fonksiyonlar:**
*   `load_tensor(data, shape) -> Tensor`: Veriyi TPU belleğine yükler.
*   `matmul(t1, t2) -> Tensor`: Matris çarpımı (Donanım hızlandırmalı).
*   `conv2d(input, kernel) -> Tensor`: 2D Konvolüsyon (CNN için).
*   `relu(t) -> Tensor`: Aktivasyon fonksiyonu.
*   `sync()`: TPU işleminin bitmesini bekler.

**Örnek:**
```nim
var t1 = tpu.load_tensor(veri1, [1024, 1024]);
var t2 = tpu.load_tensor(veri2, [1024, 1024]);
var sonuc = tpu.matmul(t1, t2); // Işık hızında işlem
```

### 12. Modüllerden Bağımsız Yardımcı Fonksiyonlar (Built-ins)

Bu fonksiyonlar herhangi bir `use` bildirimi gerektirmeden her yerden doğrudan çağrılabilir. Dilin çekirdeğine gömülüdürler.

#### 12.1 Temel Giriş/Çıkış
*   **`echo(args...)`**: Çok yönlü ekrana yazdırma fonksiyonu. Otomatik tip çıkarımı yapar ve süslü parantez `{}` ile string interpolation destekler. `print` ve `println` fonksiyonlarının daha yetenekli ve pratik halidir.
    ```nim
    echo("Merhaba Dünya\n");           // Düz metin
    echo(12345);                       // Sayısal değer
    echo("Sonuç: {10 + 20}");          // İfade interpolasyonu
    echo("Değer: {degisken}");         // Değişken interpolasyonu
    echo(fonksiyon_cagrisi());         // Fonksiyon sonucu
    ```
*   **`input(prompt)`**: Kullanıcıdan veri almak için kullanılır. Opsiyonel bir mesaj (prompt) görüntüleyebilir. Her zaman `str` (metin) döndürür.
    ```nim
    var ad = input("Adınız nedir?: ");
    ```

#### 12.2 Bellek ve Tip Bilgisi
*   **`sizeof(T)`**: Bir tipin veya değişkenin bellekte kapladığı alanı (byte cinsinden) döndürür.
*   **`typeof(val)`**: Değişkenin çalışma zamanındaki tipinin adını (`str` olarak) döndürür.
*   **`default(T)`**: Belirtilen tipin varsayılan değerini (sıfır, false, null vb.) döndürür.

#### 12.3 Güvenlik ve Hata Ayıklama
*   **`assert(kosul, mesaj)`**: Koşul `false` ise programı belirtilen mesajla ve hata koduyla durdurur. Sadece geliştirme (debug) modunda çalışır.
*   **`panic(mesaj)`**: Kurtarılamaz bir hata durumunda programı derhal sonlandırır ve yığın izini (stack trace) ekrana basar.
*   **`todo(mesaj)`**: Henüz implemente edilmemiş kod blokları için yer tutucu. Çalıştırılırsa "Not Implemented" hatası verir.

#### 12.4 Akış Kontrolü ve İşlem Yönetimi
*   **`exit(code)`**: Programı belirtilen çıkış koduyla (`0`: Başarılı, `1` ve üzeri: Hata) derhal sonlandırır. `os.exit` ile aynıdır ancak modül gerektirmez.
*   **`defer(blok)`**: İçinde bulunduğu blok (scope) tamamlandığında çalıştırılacak kodu sıraya ekler. Kaynak temizliği (dosya kapatma, bellek serbest bırakma) için idealdir.
    ```nim
    {
        var f = file.open("veri.txt", file.READ);
        defer { file.close(f); } // Blok bitince otomatik çalışır.
        // ... dosya işlemleri ...
    }
    ```

#### 12.5 Matematik ve Mantık Yardımcıları
*   **`min(a, b)`**: İki değerden küçük olanı döndürür.
*   **`max(a, b)`**: İki değerden büyük olanı döndürür.
*   **`clamp(val, min, max)`**: Değeri belirtilen aralığa sınırlar (`val < min` ise `min`, `val > max` ise `max` döner).
*   **`swap(a, b)`**: İki değişkenin değerini güvenli ve hızlı bir şekilde yer değiştirir.

#### 12.6 Evrensel Tip Dönüşümleri
Modül kullanmadan hızlıca tip dönüşümü (casting) yapmak için kısa yollar.

*   **`int(val)` / `stri32(val)`**: Verilen değeri (özellikle string'i) tamsayıya çevirir. `input`'tan gelen veriyi matematikte kullanmak için gereklidir.
    ```nim
    var yas = int(input("Yaşınız: ")); // veya stri32(giris)
    ```
*   **`float(val)`**: Değeri ondalıklı sayıya (`f64`) çevirir.
*   **`str(val)`**: Herhangi bir değeri metne çevirir.
*   **`bool(val)`**: Değeri mantıksal (`true/false`) tipe çevirir.

#### 12.7 Sonuç Yönetim Metotları (Result & Option)
NIMBLE'da hata veya boş değer dönebilen fonksiyonlar `Result` veya `Option` tipi döndürür. Bu kutuların içindeki gerçek veriyi (veya hatayı) almak için bu metotlar kullanılır.

*   **`val.unwrap()`**: Kutuyu açar ve **eğer başarılıysa** içindeki değeri döndürür. **Eğer hataysa (Err) veya boşsa (None)** programı `panic` ile patlatır (durdurur). Sadece %100 emin olduğunuzda kullanın.
*   **`val.unwrap_or(default)`**: Kutu boşsa veya hatalıysa, patlamak yerine parametre olarak verilen varsayılan değeri döndürür. Güvenli olan yöntemdir.
    ```nim
    var sayi = types.parse<i32>("abc").unwrap_or(0); // Hata olursa 0 döner.
    ```
*   **`val.expect(msg)`**: `unwrap` gibidir ama hata durumunda kendi özel mesajınızla patlar. Hata ayıklamayı kolaylaştırır.
*   **`val.is_ok()` / `val.is_err()`**: İşlemin başarılı mı başarısız mı olduğunu `bool` olarak söyler.
*   **`val.is_some()` / `val.is_none()` / `val.is_null()`**: Değer var mı yok mu (`Option` için) kontrol eder.

### 13. Derleyici Hedefleri ve Optimizasyon (Targets)

NIMBLE, kodun hangi donanım üzerinde en verimli çalışacağını belirlemek için `@target` notasyonunu ve gelişmiş derleyici bayraklarını kullanır.

#### 13.1 İşlemci Mimarileri (CPU)
Kodunuzun belirli bir işlemci ailesi için ("Native" hızda) derlenmesini sağlar.

*   **Genel Etiketler:** `x86_64`, `arm64` (Apple Silicon/Raspberry Pi), `riscv64`, `wasm32`.
*   **Kullanım (`@target`):** Fonksiyon veya modül başına optimizasyon yapabilirsiniz.

```nim
// Sadece ARM işlemcilerde (örn: M2 Chip) bu versiyon derlenir/çalışır
@target(arch="arm64", feature="neon")
fn matris_hizli() { ... }

// RISC-V vektör eklentisi varsa bunu kullan
@target(arch="riscv64", feature="v")
fn matris_hizli() { ... }
```

#### 13.2 GPU ve Hızlandırıcı Hedefleri
Farklı GPU markaları ve mimarileri için özel çekirdek (kernel) derlemeleri tanımlanabilir.

*   `nvidia`: CUDA çekirdekleri için optimize edilir.
*   `amd`: ROCm/HIP için optimize edilir.
*   `intel`: OneAPI/LevelZero için optimize edilir.
*   `apple`: Metal performansı için optimize edilir.

```nim
// NVIDIA kartlar için özel optimizasyon
@target(vendor="nvidia", arch="sm_90") // H100 GPU
gpu fn hesapla() { ... }
```

#### 13.3 Çapraz Derleme (Cross-Compilation)
Derleyiciye verilen bayraklarla hedef platform değiştirilebilir. Kod içinde `os.target_arch()` ile kontrol edilebilir.

*   **Desteklenen Hedefler:**
    *   `win-x64`: Windows Intel/AMD/ARM
    *   `win-x86`: Windows x86
    *   `linux-x64`: Linux Intel/AMD/ARM
    *   `linux-x86`: Linux x86
    *   `linux-arm64`: Linux ARM (Sunucu/Gömülü)
    *   `android-riscv`: RISC-V Android Cihazlar
    *   `mac-m` serisi: Apple Silicon

---
**NIMBLE Referans Dokümanı Sonu.**
