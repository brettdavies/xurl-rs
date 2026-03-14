/// CreateAuthCommand creates the auth command and its subcommands
pub fn create_auth_command(a: Box<Auth>) -> Box<Command> {
    let mut auth_cmd = Box::new(Command {
        r#use: "auth".to_string(),
        short: "Authentication management".to_string(),
        ..Default::default()
    });
    auth_cmd.add_command(create_auth_bearer_cmd(a));
    auth_cmd.add_command(create_auth_o_auth2_cmd(a));
    auth_cmd.add_command(create_auth_o_auth1_cmd(a));
    auth_cmd.add_command(create_auth_status_cmd());
    auth_cmd.add_command(create_auth_clear_cmd(a));
    auth_cmd.add_command(create_app_cmd(a));
    auth_cmd.add_command(create_default_cmd(a));
    auth_cmd
}
fn create_auth_bearer_cmd(a: Box<Auth>) -> Box<Command> {
    let mut bearer_token: String = Default::default();
    let mut cmd = Box::new(Command {
        r#use: "app".to_string(),
        short: "Configure app-auth (bearer token)".to_string(),
        run: |cmd: Box<Command>, args: Vec<String>| {
            let mut err = a.token_store.save_bearer_token(bearer_token);
            print!("\033[32mApp authentication successful!\033[0m\n");
        },
        ..Default::default()
    });
    cmd.flags()
        .string_var(
            &bearer_token,
            "bearer-token",
            "",
            "Bearer token for app authentication",
        );
    cmd.mark_flag_required("bearer-token");
    cmd
}
fn create_auth_o_auth2_cmd(a: Box<Auth>) -> Box<Command> {
    let mut cmd = Box::new(Command {
        r#use: "oauth2".to_string(),
        short: "Configure OAuth2 authentication".to_string(),
        run: |cmd: Box<Command>, args: Vec<String>| {
            a.o_auth2_flow("").unwrap();
            print!("\033[32mOAuth2 authentication successful!\033[0m\n");
        },
        ..Default::default()
    });
    cmd
}
fn create_auth_o_auth1_cmd(a: Box<Auth>) -> Box<Command> {
    let mut consumer_key: String = Default::default();
    let mut consumer_secret: String = Default::default();
    let mut access_token: String = Default::default();
    let mut token_secret: String = Default::default();
    let mut cmd = Box::new(Command {
        r#use: "oauth1".to_string(),
        short: "Configure OAuth1 authentication".to_string(),
        run: |cmd: Box<Command>, args: Vec<String>| {
            let mut err = a
                .token_store
                .save_o_auth1_tokens(
                    access_token,
                    token_secret,
                    consumer_key,
                    consumer_secret,
                );
            print!("\033[32mOAuth1 credentials saved successfully!\033[0m\n");
        },
        ..Default::default()
    });
    cmd.flags().string_var(&consumer_key, "consumer-key", "", "Consumer key for OAuth1");
    cmd.flags()
        .string_var(
            &consumer_secret,
            "consumer-secret",
            "",
            "Consumer secret for OAuth1",
        );
    cmd.flags().string_var(&access_token, "access-token", "", "Access token for OAuth1");
    cmd.flags().string_var(&token_secret, "token-secret", "", "Token secret for OAuth1");
    cmd.mark_flag_required("consumer-key");
    cmd.mark_flag_required("consumer-secret");
    cmd.mark_flag_required("access-token");
    cmd.mark_flag_required("token-secret");
    cmd
}
fn create_auth_status_cmd() -> Box<Command> {
    let mut cmd = Box::new(Command {
        r#use: "status".to_string(),
        short: "Show authentication status".to_string(),
        run: |cmd: Box<Command>, args: Vec<String>| {
            let mut ts = store.new_token_store();
            let mut apps = ts.list_apps();
            let mut default_app = ts.get_default_app();
            if apps.len() == 0 {
                println!(
                    "{:?}",
                    "No apps registered. Use 'xurl auth apps add' to register one."
                );
                return;
            }
            for (i, name) in apps.iter().enumerate() {
                let mut app = ts.get_app(name);
                let mut marker = " ";
                if name == default_app {
                    marker = "▸";
                }
                let mut client_hint = "(no credentials)";
                if app.client_id != "" {
                    client_hint = format!(
                        "client_id: {}…", truncate(app.client_id, 8)
                    );
                }
                print!("{} {}  [{}]\n", marker, name, client_hint);
                let mut usernames = ts.get_o_auth2_usernames_for_app(name);
                if usernames.len() > 0 {
                    for u in usernames.iter() {
                        if u == app.default_user {
                            print!("    ▸ oauth2: {}\n", u);
                        } else {
                            print!("      oauth2: {}\n", u);
                        }
                    }
                } else {
                    println!("{:?}", "      oauth2: (none)");
                }
                if app.o_auth1_token.is_some() {
                    println!("{:?}", "      oauth1: ✓");
                } else {
                    println!("{:?}", "      oauth1: –");
                }
                if app.bearer_token.is_some() {
                    println!("{:?}", "      bearer: ✓");
                } else {
                    println!("{:?}", "      bearer: –");
                }
                if i < apps.len() - 1 {
                    println!();
                }
            }
        },
        ..Default::default()
    });
    cmd
}
fn create_auth_clear_cmd(a: Box<Auth>) -> Box<Command> {
    let mut all: bool = Default::default();
    let mut oauth1: bool = Default::default();
    let mut bearer: bool = Default::default();
    let mut oauth2_username: String = Default::default();
    let mut cmd = Box::new(Command {
        r#use: "clear".to_string(),
        short: "Clear authentication tokens".to_string(),
        run: |cmd: Box<Command>, args: Vec<String>| {
            if all {
                let mut err = a.token_store.clear_all();
                println!("{:?}", "All authentication cleared!");
            } else if oauth1 {
                let mut err = a.token_store.clear_o_auth1_tokens();
                println!("{:?}", "OAuth1 tokens cleared!");
            } else if oauth2_username != "" {
                let mut err = a.token_store.clear_o_auth2_token(oauth2_username);
                println!("{:?} {:?}", "OAuth2 token cleared for", oauth2_username + "!");
            } else if bearer {
                let mut err = a.token_store.clear_bearer_token();
                println!("{:?}", "Bearer token cleared!");
            } else {
                println!(
                    "{:?}",
                    "No authentication cleared! Use --all to clear all authentication."
                );
                std::process::exit(1);
            }
        },
        ..Default::default()
    });
    cmd.flags().bool_var(&all, "all", false, "Clear all authentication");
    cmd.flags().bool_var(&oauth1, "oauth1", false, "Clear OAuth1 tokens");
    cmd.flags()
        .string_var(
            &oauth2_username,
            "oauth2-username",
            "",
            "Clear OAuth2 token for username",
        );
    cmd.flags().bool_var(&bearer, "bearer", false, "Clear bearer token");
    cmd
}
fn create_app_cmd(a: Box<Auth>) -> Box<Command> {
    let mut app_cmd = Box::new(Command {
        r#use: "apps".to_string(),
        short: "Manage registered X API apps".to_string(),
        ..Default::default()
    });
    app_cmd.add_command(create_app_add_cmd(a));
    app_cmd.add_command(create_app_update_cmd(a));
    app_cmd.add_command(create_app_remove_cmd(a));
    app_cmd.add_command(create_app_list_cmd());
    app_cmd
}
fn create_app_add_cmd(a: Box<Auth>) -> Box<Command> {
    let mut client_id: String = Default::default();
    let mut client_secret: String = Default::default();
    let mut cmd = Box::new(Command {
        r#use: "add NAME".to_string(),
        short: "Register a new X API app".to_string(),
        long: r#"Register a new X API app with a client ID and secret.

Examples:
  xurl auth apps add my-app --client-id abc --client-secret xyz"#
            .to_string(),
        args: cobra.exact_args(1),
        run: |cmd: Box<Command>, args: Vec<String>| {
            let mut name = args[0];
            let mut err = a.token_store.add_app(name, client_id, client_secret);
            print!("\033[32mApp {:?} registered!\033[0m\n", name);
            if a.token_store.list_apps().len() == 1 {
                print!("  (set as default app)\n");
            }
        },
        ..Default::default()
    });
    cmd.flags().string_var(&client_id, "client-id", "", "OAuth2 client ID");
    cmd.flags().string_var(&client_secret, "client-secret", "", "OAuth2 client secret");
    cmd.mark_flag_required("client-id");
    cmd.mark_flag_required("client-secret");
    cmd
}
fn create_app_update_cmd(a: Box<Auth>) -> Box<Command> {
    let mut client_id: String = Default::default();
    let mut client_secret: String = Default::default();
    let mut cmd = Box::new(Command {
        r#use: "update NAME".to_string(),
        short: "Update credentials for an existing app".to_string(),
        long: r#"Update the client ID and/or secret for an existing registered app.

Examples:
  xurl auth apps update default --client-id abc --client-secret xyz
  xurl auth apps update my-app --client-id newid"#
            .to_string(),
        args: cobra.exact_args(1),
        run: |cmd: Box<Command>, args: Vec<String>| {
            let mut name = args[0];
            if client_id == "" && client_secret == "" {
                println!(
                    "{:?}",
                    "Nothing to update. Provide --client-id and/or --client-secret."
                );
                std::process::exit(1);
            }
            let mut err = a.token_store.update_app(name, client_id, client_secret);
            print!("\033[32mApp {:?} updated.\033[0m\n", name);
        },
        ..Default::default()
    });
    cmd.flags().string_var(&client_id, "client-id", "", "OAuth2 client ID");
    cmd.flags().string_var(&client_secret, "client-secret", "", "OAuth2 client secret");
    cmd
}
fn create_app_remove_cmd(a: Box<Auth>) -> Box<Command> {
    let mut cmd = Box::new(Command {
        r#use: "remove NAME".to_string(),
        short: "Remove a registered app and all its tokens".to_string(),
        args: cobra.exact_args(1),
        run: |cmd: Box<Command>, args: Vec<String>| {
            let mut name = args[0];
            let mut err = a.token_store.remove_app(name);
            print!("\033[32mApp {:?} removed.\033[0m\n", name);
        },
        ..Default::default()
    });
    cmd
}
fn create_app_list_cmd() -> Box<Command> {
    let mut cmd = Box::new(Command {
        r#use: "list".to_string(),
        short: "List registered apps".to_string(),
        run: |cmd: Box<Command>, args: Vec<String>| {
            let mut ts = store.new_token_store();
            let mut apps = ts.list_apps();
            let mut default_app = ts.get_default_app();
            if apps.len() == 0 {
                println!(
                    "{:?}",
                    "No apps registered. Use 'xurl auth apps add' to register one."
                );
                return;
            }
            for name in apps.iter() {
                let mut app = ts.get_app(name);
                let mut marker = "  ";
                if name == default_app {
                    marker = "▸ ";
                }
                let mut client_hint = "";
                if app.client_id != "" {
                    client_hint = format!(
                        " (client_id: {}…)", truncate(app.client_id, 8)
                    );
                }
                print!("{}{}{}\n", marker, name, client_hint);
            }
        },
        ..Default::default()
    });
    cmd
}
fn create_default_cmd(a: Box<Auth>) -> Box<Command> {
    let mut cmd = Box::new(Command {
        r#use: "default [APP_NAME [USERNAME]]".to_string(),
        short: "Set default app and/or user (interactive or by argument)".to_string(),
        long: r#"Set the default app and/or OAuth2 user.

Without arguments: launches an interactive picker (Bubble Tea).
With one argument:  sets the default app.
With two arguments: sets the default app and default OAuth2 user.

Examples:
  xurl auth default                     # interactive picker
  xurl auth default my-app              # set default app
  xurl auth default my-app alice        # set default app + user"#
            .to_string(),
        args: cobra.maximum_n_args(2),
        run: |cmd: Box<Command>, args: Vec<String>| {
            let mut ts = a.token_store;
            if args.len() >= 1 {
                let mut app_name = args[0];
                let mut err = ts.set_default_app(app_name);
                print!("\033[32mDefault app set to {:?}\033[0m\n", app_name);
                if args.len() == 2 {
                    let mut user_name = args[1];
                    let mut err = ts.set_default_user(app_name, user_name);
                    print!("\033[32mDefault user set to {:?}\033[0m\n", user_name);
                }
                return;
            }
            let mut apps = ts.list_apps();
            if apps.len() == 0 {
                println!(
                    "{:?}",
                    "No apps registered. Use 'xurl auth apps add' to register one."
                );
                return;
            }
            let mut app_choice = run_picker("Select default app", apps).unwrap();
            if app_choice == "" {
                return;
            }
            let mut err = ts.set_default_app(app_choice);
            print!("\033[32mDefault app set to {:?}\033[0m\n", app_choice);
            let mut users = ts.get_o_auth2_usernames_for_app(app_choice);
            if users.len() > 0 {
                let mut user_choice = run_picker("Select default OAuth2 user", users)
                    .unwrap();
                if user_choice != "" {
                    let mut err = ts.set_default_user(app_choice, user_choice);
                    print!("\033[32mDefault user set to {:?}\033[0m\n", user_choice);
                }
            }
        },
        ..Default::default()
    });
    cmd
}
fn truncate(s: &str, max_len: i64) -> String {
    if s.len() <= max_len {
        s
    }
    s[..max_len]
}
