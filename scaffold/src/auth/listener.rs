// WARNING: Generated code could not be parsed by syn for formatting.
// Run `cargo fmt` manually after fixing any syntax issues.

use reqwest;

pub fn start_listener(port: i64, callback: fn(code string, state string) -> anyhow::Error) -> anyhow::Result<()> {
    let mut server = Box::new(Server /* todo: http.Server */ { addr: format!("127.0.0.1:{}", port), handler: http.default_serve_mux, ..Default::default() });
    let mut done = tokio::sync::mpsc::channel(1);
    http.handle_func("/callback", |w: ResponseWriter /* todo: http.ResponseWriter */, r: Box<reqwest::Request>| {
        let mut code = r.url.query().get("code");
        let mut state = r.url.query().get("state");
        let mut err = callback(code, state);
        // if err != nil { ... } — handled by ? above
        w.write_header(http.status_ok);
        write!(w, "Authentication successful! You can close this window.");
        done.send(None).await;
        std::thread::spawn(move || {
        server.shutdown(context.background());
        });
    });
    std::thread::spawn(move || {
        let mut err = server.listen_and_serve();
        if err.is_some() && err != http.err_server_closed {
            done.send(xurl_errors.new_auth_error("ServerError", err)).await;
        }
    });
    // select { } — requires tokio::select!
    todo!("select");
}

