// Intègre assets/logo.ico comme icône et les informations de version (Propriétés > Détails) du fichier shirokap.exe (si l'outil Windows `rc` est dispo ; sinon on continue sans).
fn main() {
    println!("cargo:rerun-if-changed=assets/logo.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").map(|o| o == "windows").unwrap_or(false) {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/logo.ico")
            .set("ProductName", "Shirokap")
            .set("FileDescription", "Shirokap - lightweight screenshot tool")
            .set("CompanyName", "Saishiiro")
            .set("LegalCopyright", "Copyright (c) 2026 Saishiiro")
            .set("OriginalFilename", "shirokap.exe");
        if let Err(e) = res.compile() {
            println!("cargo:warning=icone de l'exe ignoree : {e}");
        }
    }
}
