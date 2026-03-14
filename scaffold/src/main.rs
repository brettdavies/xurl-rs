fn main() {
    let mut config = config.new_config();
    let mut auth = auth.new_auth(config);
    let mut root_cmd = cli.create_root_command(config, auth);
    let mut err = root_cmd.execute();
}
