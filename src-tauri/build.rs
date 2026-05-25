fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_manifest("windows/app.manifest");
        if let Err(e) = res.compile() {
            eprintln!("winres compile warning: {e}");
        }
    }

    tauri_build::build()
}
