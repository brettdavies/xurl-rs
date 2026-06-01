fn main() {
    // Restore default SIGPIPE handling (Rust masks it, causing panics on closed pipes)
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    std::process::exit(xurl::cli::run_argv());
}
