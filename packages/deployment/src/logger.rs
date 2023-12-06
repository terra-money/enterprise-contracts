use pretty_env_logger::env_logger;

pub fn enable_info_logger() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init(); // Used to log contract and chain interactions
}
