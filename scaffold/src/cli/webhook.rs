use base64;
use serde_json;
use reqwest;
fn webhook_port_init() -> i64 {
    todo!("package-level var init")
}
fn output_file_name_init() -> String {
    todo!("package-level var init")
}
fn quiet_mode_init() -> bool {
    todo!("package-level var init")
}
fn pretty_mode_init() -> bool {
    todo!("package-level var init")
}
/// CreateWebhookCommand creates the webhook command and its subcommands.
pub fn create_webhook_command(auth_instance: Box<Auth>) -> Box<Command> {
    let mut webhook_cmd = Box::new(Command {
        r#use: "webhook".to_string(),
        short: "Manage webhooks for the X API".to_string(),
        long: r#"Manages X API webhooks. Currently supports starting a local server with an ngrok tunnel to handle CRC checks."#
            .to_string(),
        ..Default::default()
    });
    let mut webhook_start_cmd = Box::new(Command {
        r#use: "start".to_string(),
        short: "Start a local webhook server with an ngrok tunnel".to_string(),
        long: r#"Starts a local HTTP server and an ngrok tunnel to listen for X API webhook events, including CRC checks. POST request bodies can be saved to a file using the -o flag. Use -q for quieter console logging of POST events. Use -p to pretty-print JSON POST bodies in the console."#
            .to_string(),
        run: |cmd: Box<Command>, args: Vec<String>| {
            color.cyan("Starting webhook server with ngrok...");
            if auth_instance.is_none() || auth_instance.token_store.is_none() {
                color.red("Error: Authentication module not initialized properly.");
                std::process::exit(1);
            }
            let mut oauth1_token = auth_instance.token_store.get_o_auth1_tokens();
            if oauth1_token.is_none() || oauth1_token.o_auth1.is_none()
                || oauth1_token.o_auth1.consumer_secret == ""
            {
                color
                    .red(
                        "Error: OAuth 1.0a consumer secret not found. Please configure OAuth 1.0a credentials using 'xurl auth oauth1'.",
                    );
                std::process::exit(1);
            }
            let mut consumer_secret = oauth1_token.o_auth1.consumer_secret;
            let mut output_file: Box<std::fs::File> = Default::default();
            let mut err_open_file: anyhow::Error = Default::default();
            if output_file_name != "" {
                (output_file, err_open_file) = os
                    .open_file(
                        output_file_name,
                        os.o_append | os.o_create | os.o_wronly,
                        0644,
                    );
                if err_open_file.is_some() {
                    color
                        .red(
                            "Error opening output file %s: %v",
                            output_file_name,
                            err_open_file,
                        );
                    std::process::exit(1);
                }
                let _defer = scopeguard::guard(
                    (),
                    |_| {
                        output_file.close();
                    },
                );
                color.green("Logging POST request bodies to: %s", output_file_name);
            }
            color
                .yellow(
                    "Enter your ngrok authtoken (leave empty to try NGROK_AUTHTOKEN env var): ",
                );
            let mut reader = bufio.new_reader(os.stdin);
            let (ngrok_auth_token, _) = reader.read_string('\n');
            ngrok_auth_token = ngrok_auth_token.trim();
            let mut ctx = context.background();
            let mut tunnel_opts: Vec<ConnectOption> = Default::default();
            if ngrok_auth_token != "" {
                tunnel_opts = {
                    tunnel_opts.push(ngrok.with_authtoken(ngrok_auth_token));
                    tunnel_opts.clone()
                };
            } else {
                color
                    .cyan(
                        "Attempting to use NGROK_AUTHTOKEN environment variable for ngrok authentication.",
                    );
                tunnel_opts = {
                    tunnel_opts.push(ngrok.with_authtoken_from_env());
                    tunnel_opts.clone()
                };
            }
            let mut forward_to_addr = format!("localhost:{}", webhook_port);
            color
                .cyan(
                    "Configuring ngrok to forward to local port: %s",
                    color.magenta_string("%d", webhook_port),
                );
            let mut ngrok_listener = ngrok
                .listen(
                    ctx,
                    config.http_endpoint(config.with_forwards_to(forward_to_addr)),
                    tunnel_opts,
                )
                .unwrap();
            let _defer = scopeguard::guard(
                (),
                |_| {
                    ngrok_listener.close();
                },
            );
            color.green("Ngrok tunnel established!");
            print!(
                "  Forwarding URL: {} -> {}\n", color.hi_green_string(ngrok_listener
                .url()), color.magenta_string(forward_to_addr)
            );
            color
                .yellow(
                    "Use this URL for your X API webhook registration: %s/webhook",
                    color.hi_green_string(ngrok_listener.url()),
                );
            http.handle_func(
                "/webhook",
                |w: ResponseWriter, r: Box<reqwest::Request>| {
                    if r.method == http.method_get {
                        let mut crc_token = r.url.query().get("crc_token");
                        if crc_token == "" {
                            http.error(
                                w,
                                "Error: crc_token missing from request",
                                http.status_bad_request,
                            );
                            eprintln!(
                                "{}", "[WARN] Received GET /webhook without crc_token"
                            );
                            return;
                        }
                        eprintln!(
                            "{}", "[INFO] Received GET %s%s with crc_token: %s", color
                            .blue_string(r.host), color.blue_string(r.url.path), color
                            .yellow_string(crc_token)
                        );
                        let mut mac = hmac
                            .new(sha256.new, consumer_secret.as_bytes().to_vec());
                        mac.write(crc_token.as_bytes().to_vec());
                        let mut hashed_token = mac.sum(None);
                        let mut encoded_token = base64
                            .std_encoding
                            .encode_to_string(hashed_token);
                        let mut response = std::collections::HashMap::from([
                            ("response_token", "sha256=" + encoded_token),
                        ]);
                        w.header().set("Content-Type", "application/json");
                        json.new_encoder(w).encode(response);
                        eprintln!(
                            "{}", "[INFO] Responded to CRC check with token: %s", color
                            .green_string(response["response_token"])
                        );
                    } else if r.method == http.method_post {
                        let mut body_bytes = {
                            let mut buf = String::new();
                            r.body.read_to_string(&mut buf)?;
                            buf
                        }
                            .unwrap();
                        let _defer = scopeguard::guard(
                            (),
                            |_| {
                                r.body.close();
                            },
                        );
                        if quiet_mode {
                            eprintln!(
                                "{}", "[INFO] Received POST %s%s event (quiet mode).", color
                                .blue_string(r.host), color.blue_string(r.url.path)
                            );
                        } else {
                            eprintln!(
                                "{}", "[INFO] Received POST %s%s event:", color
                                .blue_string(r.host), color.blue_string(r.url.path)
                            );
                            if pretty_mode {
                                let mut json_data: Box<dyn std::any::Any> = Default::default();
                                if serde_json::from_str(&body_bytes).is_none() {
                                    let mut pretty_colored = pretty
                                        .color(pretty.pretty(body_bytes), pretty.terminal_style);
                                    eprintln!(
                                        "{}", "[DATA] Body:\n%s", String::from(pretty_colored)
                                    );
                                } else {
                                    eprintln!(
                                        "{}",
                                        "[DATA] Body (raw, not valid JSON for pretty print):\n%s",
                                        String::from(body_bytes)
                                    );
                                }
                            } else {
                                eprintln!(
                                    "{}", "[DATA] Body: %s", String::from(body_bytes)
                                );
                            }
                        }
                        if output_file.is_some() {
                            output_file.write(body_bytes).unwrap();
                        }
                        w.write_header(http.status_ok);
                    } else {
                        http.error(
                            w,
                            "Method not allowed",
                            http.status_method_not_allowed,
                        );
                    }
                },
            );
            color
                .cyan(
                    "Starting local HTTP server to handle requests from ngrok tunnel (forwarded from %s)...",
                    color.hi_green_string(ngrok_listener.url()),
                );
            let mut err = http.serve(ngrok_listener, None);
            color.yellow("Webhook server and ngrok tunnel shut down.");
        },
        ..Default::default()
    });
    webhook_start_cmd
        .flags()
        .int_var_p(
            &webhook_port,
            "port",
            "p",
            8080,
            "Local port for the webhook server to listen on (ngrok will forward to this port)",
        );
    webhook_start_cmd
        .flags()
        .string_var_p(
            &output_file_name,
            "output",
            "o",
            "",
            "File to write incoming POST request bodies to",
        );
    webhook_start_cmd
        .flags()
        .bool_var_p(
            &quiet_mode,
            "quiet",
            "q",
            false,
            "Enable quiet mode (logs only that a POST event was received, not the full body to console)",
        );
    webhook_start_cmd
        .flags()
        .bool_var_p(
            &pretty_mode,
            "pretty",
            "P",
            false,
            "Pretty-print JSON POST bodies in console output (ignored if -q is used)",
        );
    webhook_cmd.add_command(webhook_start_cmd);
    webhook_cmd
}
