/// These variables will be set during build time via ldflags.
/// When installed via `go install`, they fall back to Go's embedded module info.
pub fn version_init() -> String {
    todo!("package-level var init")
}
/// These variables will be set during build time via ldflags.
/// When installed via `go install`, they fall back to Go's embedded module info.
pub fn commit_init() -> String {
    todo!("package-level var init")
}
/// These variables will be set during build time via ldflags.
/// When installed via `go install`, they fall back to Go's embedded module info.
pub fn build_date_init() -> String {
    todo!("package-level var init")
}
fn init() {
    if version != "" {
        return;
    }
    let (info, ok) = debug.read_build_info();
    if ok && info.main.version != "" && info.main.version != "(devel)" {
        version = info.main.version;
    } else {
        version = "dev";
    }
}
