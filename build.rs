fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=assets/icon.png");

    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "Marshall CODE Editor");
        res.set("FileDescription", "Marshall CODE amp preset editor");
        res.set("CompanyName", "CODE Editor");
        res.set("LegalCopyright", "MIT");
        if let Err(e) = res.compile() {
            // Allow builds without the Windows SDK resource compiler in odd CI setups.
            println!("cargo:warning=winresource failed: {e}");
        }
    }
}
