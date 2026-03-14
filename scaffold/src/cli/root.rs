/// CreateRootCommand creates the root command for the xurl CLI
pub fn create_root_command(cfg: Box<Config>, a: Box<Auth>) -> Box<Command> {
    let mut root_cmd = Box::new(Command {
        r#use: "xurl [flags] URL".to_string(),
        short: "Auth enabled curl-like interface for the X API".to_string(),
        long: r#"A command-line tool for making authenticated requests to the X API.

Shortcut commands (agent‑friendly):
  xurl post "Hello world!"                        Post to X
  xurl reply 1234567890 "Nice!"                   Reply to a post
  xurl read 1234567890                             Read a post
  xurl search "golang" -n 20                       Search posts
  xurl whoami                                      Show your profile
  xurl like 1234567890                             Like a post
  xurl repost 1234567890                           Repost
  xurl follow @user                                Follow a user
  xurl dm @user "Hey!"                             Send a DM
  xurl timeline                                    Home timeline
  xurl mentions                                    Your mentions

Raw API access (curl‑style):
  basic requests        xurl /2/users/me
                        xurl -X POST /2/tweets -d '{"text":"Hello world!"}'
                        xurl -H "Content-Type: application/json" /2/tweets
  authentication        xurl --auth oauth2 /2/users/me
                        xurl --auth oauth1 /2/users/me
                        xurl --auth app /2/users/me
  media and streaming   xurl media upload path/to/video.mp4
                        xurl /2/tweets/search/stream --auth app
                        xurl -s /2/users/me

Multi-app management:
  xurl auth apps add my-app --client-id ... --client-secret ...
  xurl auth apps list
  xurl auth default                                # interactive picker
  xurl auth default my-app                         # set by name
  xurl --app my-app /2/users/me                    # per-request override

Run 'xurl --help' to see all available commands."#
            .to_string(),
        persistent_pre_run: |cmd: Box<Command>, args: Vec<String>| {
            let (app_override, _) = cmd.flags().get_string("app");
            if app_override != "" {
                a.with_app_name(app_override);
            }
        },
        args: |cmd: Box<Command>, args: Vec<String>| -> anyhow::Error { Ok(()) },
        run: |cmd: Box<Command>, args: Vec<String>| {
            let (method, _) = cmd.flags().get_string("method");
            if method == "" {
                method = "GET";
            }
            let (headers, _) = cmd.flags().get_string_array("header");
            let (data, _) = cmd.flags().get_string("data");
            let (auth_type, _) = cmd.flags().get_string("auth");
            let (username, _) = cmd.flags().get_string("username");
            let (verbose, _) = cmd.flags().get_bool("verbose");
            let (trace, _) = cmd.flags().get_bool("trace");
            let (force_stream, _) = cmd.flags().get_bool("stream");
            let (media_file, _) = cmd.flags().get_string("file");
            if args.len() == 0 {
                println!("{:?}", "No URL provided");
                println!("{:?}", "Usage: xurl [OPTIONS] [URL] [COMMAND]");
                println!("{:?}", "Try 'xurl --help' for more information.");
                std::process::exit(1);
            }
            let mut url = args[0];
            let mut client = api.new_api_client(cfg, a);
            let mut request_options = RequestOptions {
                method: method,
                endpoint: url,
                headers: headers,
                data: data,
                auth_type: auth_type,
                username: username,
                verbose: verbose,
                trace: trace,
                ..Default::default()
            };
            let mut err = api
                .handle_request(request_options, force_stream, media_file, client);
        },
        ..Default::default()
    });
    root_cmd
        .persistent_flags()
        .string("app", "", "Use a specific registered app (overrides default)");
    root_cmd.flags().string_p("method", "X", "", "HTTP method (GET by default)");
    root_cmd.flags().string_array_p("header", "H", vec![], "Request headers");
    root_cmd.flags().string_p("data", "d", "", "Request body data");
    root_cmd.flags().string("auth", "", "Authentication type (oauth1 or oauth2)");
    root_cmd.flags().string_p("username", "u", "", "Username for OAuth2 authentication");
    root_cmd.flags().bool_p("verbose", "v", false, "Print verbose information");
    root_cmd.flags().bool_p("trace", "t", false, "Add trace header to request");
    root_cmd
        .flags()
        .bool_p(
            "stream",
            "s",
            false,
            "Force streaming mode for non-streaming endpoints",
        );
    root_cmd
        .flags()
        .string_p("file", "F", "", "File to upload (for multipart requests)");
    root_cmd.add_command(create_auth_command(a));
    root_cmd.add_command(create_media_command(a));
    root_cmd.add_command(create_version_command());
    root_cmd.add_command(create_webhook_command(a));
    create_shortcut_commands(root_cmd, a);
    root_cmd
}
