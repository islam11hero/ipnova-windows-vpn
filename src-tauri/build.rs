fn main() {
    // Tauri embeds Windows VERSION + resources; winres caused duplicate VERSION (LNK1123).
    tauri_build::build()
}
