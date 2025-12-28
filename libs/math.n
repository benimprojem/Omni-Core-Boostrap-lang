// libs/math.nim
// Bu bir kütüphane modülüdür.

pub const PI: f64 = 3.1415926535;

pub struct Vec2 {
    x: f64;
    y: f64;
}

// Bu fonksiyon dışarıya açıktır.
pub fn topla(a: i32, b: i32): i32 {
    return a + b;
}

// Bu fonksiyon sadece bu modül içinde kullanılabilir.
fn private_helper(): void {
    echo("Bu özel bir yardımcı fonksiyondur.");
}

// Bu fonksiyon dışarıya açıktır ve özel fonksiyonu kullanır.
pub fn cikar(a: i32, b: i32): i32 {
    private_helper();
    return a - b;
}