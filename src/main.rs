mod config;
mod sshr;
mod sshct;
mod utils;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // 実行されたバイナリ名を取得
    let binary_name = args[0]
        .split('/')
        .last()
        .unwrap_or("unknown");
    
    match binary_name {
        "sshr" => {
            if args.len() < 2 {
                eprintln!("Usage: sshr <host>");
                std::process::exit(1);
            }
            
            let host = &args[1];
            if let Err(e) = sshr::run_sshr(host) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        "sshct" => {
            let sshct_args = if args.len() > 1 {
                &args[1..]
            } else {
                &[]
            };
            
            if let Err(e) = sshct::run_sshct(sshct_args) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("Unknown binary: {}", binary_name);
            eprintln!("This binary should be named either 'sshr' or 'sshct'");
            std::process::exit(1);
        }
    }
}
