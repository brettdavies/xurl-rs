/// CreateVersionCommand creates the version command
pub fn create_version_command() -> Box<Command> {
    let mut version_cmd = Box::new(Command {
        r#use: "version".to_string(),
        short: "Show xurl version information".to_string(),
        run: |cmd: Box<Command>, args: Vec<String>| {
            print!("xurl {}\n", version.version);
        },
        ..Default::default()
    });
    version_cmd
}
