use std::io;

pub fn get_ssh_config_path_string() -> String {
    let home = std::env::var("HOME").expect("HOME environment variable not set");
    format!("{}/.ssh/config", home)
}

pub fn handle_error<T>(result: io::Result<T>, error_msg: &str) -> T {
    match result {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Error: {} - {}", error_msg, e);
            std::process::exit(1);
        }
    }
}
