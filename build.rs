fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set("FileDescription", "Kısayolluk");
        res.set("ProductName", "Kısayolluk");
        res.set("OriginalFilename", "kisayolluk.exe");
        res.set("InternalName", "kisayolluk.exe");
        res.set("CompanyName", "Kısayolluk");
        res.set("LegalCopyright", "Kısayolluk");
        // 0x0000 represents Language Neutral, which Windows Task Manager uses across all system languages
        res.set_language(0x0000);
        if let Err(e) = res.compile() {
            eprintln!("Windows kaynak derleme hatası: {}", e);
        }
    }
}

