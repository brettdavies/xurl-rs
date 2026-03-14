/// CreateMediaCommand creates the media command and its subcommands
pub fn create_media_command(auth: Box<Auth>) -> Box<Command> {
    let mut media_cmd = Box::new(Command {
        r#use: "media".to_string(),
        short: "Media upload operations".to_string(),
        ..Default::default()
    });
    media_cmd.add_command(create_media_upload_cmd(auth));
    media_cmd.add_command(create_media_status_cmd(auth));
    media_cmd
}
/// Create media upload subcommand
fn create_media_upload_cmd(auth: Box<Auth>) -> Box<Command> {
    let mut media_type: String = Default::default();
    let mut media_category: String = Default::default();
    let mut wait_for_processing: bool = Default::default();
    let mut cmd = Box::new(Command {
        r#use: "upload [flags] FILE".to_string(),
        short: "Upload media file".to_string(),
        long: r#"Upload a media file to X API. Supports images, GIFs, and videos."#
            .to_string(),
        args: cobra.exact_args(1),
        run: |cmd: Box<Command>, args: Vec<String>| {
            let mut file_path = args[0];
            let (auth_type, _) = cmd.flags().get_string("auth");
            let (username, _) = cmd.flags().get_string("username");
            let (verbose, _) = cmd.flags().get_bool("verbose");
            let (headers, _) = cmd.flags().get_string_array("header");
            let (trace, _) = cmd.flags().get_bool("trace");
            let mut config = config.new_config();
            let mut client = api.new_api_client(config, auth);
            let mut err = api
                .execute_media_upload(
                    file_path,
                    media_type,
                    media_category,
                    auth_type,
                    username,
                    verbose,
                    trace,
                    wait_for_processing,
                    headers,
                    client,
                );
        },
        ..Default::default()
    });
    cmd.flags()
        .string_var(
            &media_type,
            "media-type",
            "video/mp4",
            "Media type (e.g., image/jpeg, image/png, video/mp4)",
        );
    cmd.flags()
        .string_var(
            &media_category,
            "category",
            "amplify_video",
            "Media category (e.g., tweet_image, tweet_video, amplify_video)",
        );
    cmd.flags()
        .bool_var(
            &wait_for_processing,
            "wait",
            true,
            "Wait for media processing to complete",
        );
    cmd.flags().string("auth", "", "Authentication type (oauth1 or oauth2)");
    cmd.flags().string_p("username", "u", "", "Username for OAuth2 authentication");
    cmd.flags().bool_p("verbose", "v", false, "Print verbose information");
    cmd.flags().bool_p("trace", "t", false, "Add trace header to request");
    cmd.flags().string_array_p("header", "H", vec![], "Request headers");
    cmd
}
/// Create media status subcommand
fn create_media_status_cmd(auth: Box<Auth>) -> Box<Command> {
    let mut cmd = Box::new(Command {
        r#use: "status [flags] MEDIA_ID".to_string(),
        short: "Check media upload status".to_string(),
        long: r#"Check the status of a media upload by media ID."#.to_string(),
        args: cobra.exact_args(1),
        run: |cmd: Box<Command>, args: Vec<String>| {
            let mut media_id = args[0];
            let (auth_type, _) = cmd.flags().get_string("auth");
            let (username, _) = cmd.flags().get_string("username");
            let (verbose, _) = cmd.flags().get_bool("verbose");
            let (wait, _) = cmd.flags().get_bool("wait");
            let (trace, _) = cmd.flags().get_bool("trace");
            let (headers, _) = cmd.flags().get_string_array("header");
            let mut config = config.new_config();
            let mut client = api.new_api_client(config, auth);
            let mut err = api
                .execute_media_status(
                    media_id,
                    auth_type,
                    username,
                    verbose,
                    wait,
                    trace,
                    headers,
                    client,
                );
        },
        ..Default::default()
    });
    cmd.flags().string("auth", "", "Authentication type (oauth1 or oauth2)");
    cmd.flags().string_p("username", "u", "", "Username for OAuth2 authentication");
    cmd.flags().bool_p("verbose", "v", false, "Print verbose information");
    cmd.flags().bool_p("wait", "w", false, "Wait for media processing to complete");
    cmd.flags().bool_p("trace", "t", false, "Add trace header to request");
    cmd.flags().string_array_p("header", "H", vec![], "Request headers");
    cmd
}
