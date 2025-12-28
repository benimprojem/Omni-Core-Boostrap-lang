// libs/os.nim
//
// Bu modül, işletim sistemi etkileşimleri için ana giriş noktasıdır.
// Derleyicinin hedef platformuna göre ilgili alt modülü (örn: os/windows)
// otomatik olarak yükler ve dışa aktarır.

// Derleyici, hedef platforma göre bu 'use' ifadesini akıllıca çözümleyecek
// ve 'os/windows.nim' veya 'os/linux.nim' gibi doğru dosyayı yükleyecektir.
export use os/platform;

// Buraya, tüm platformlarda ortak olan ve NIM ile yazılmış
// os fonksiyonları eklenebilir.
// Örn: pub fn is_windows(): bool { return true; } // (Bu, #ifdef ile daha iyi olurdu)