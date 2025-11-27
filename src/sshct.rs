use crate::config::{add_ssh_config, find_config_by_host, update_ssh_config, SshConfig};
use crate::sshr::run_sshr;
use dialoguer::{Input, Confirm};
use rpassword::read_password;
use std::io::{self, Write};

pub fn run_sshct(args: &[String]) -> io::Result<()> {
    if args.is_empty() {
        eprintln!("Usage: sshct <new|show|edit> [server_name]");
        std::process::exit(1);
    }
    
    match args[0].as_str() {
        "new" => {
            let server_name = if args.len() > 1 {
                args[1].clone()
            } else {
                Input::<String>::new()
                    .with_prompt("input client name(must)")
                    .interact_text()
                    .unwrap()
            };
            sshct_new(&server_name)
        }
        "show" => {
            if args.len() < 2 {
                eprintln!("Usage: sshct show <server_name>");
                std::process::exit(1);
            }
            sshct_show(&args[1])
        }
        "edit" => {
            if args.len() < 2 {
                eprintln!("Usage: sshct edit <server_name>");
                std::process::exit(1);
            }
            sshct_edit(&args[1])
        }
        _ => {
            eprintln!("Unknown command: {}", args[0]);
            eprintln!("Available commands: new, show, edit");
            std::process::exit(1);
        }
    }
}

fn sshct_new(server_name: &str) -> io::Result<()> {
    println!("Creating new SSH configuration for '{}'", server_name);
    
    let hostname: String = Input::new()
        .with_prompt("input host name(must)")
        .interact_text()
        .unwrap();
    
    let username: String = Input::new()
        .with_prompt("input user name(must)")
        .interact_text()
        .unwrap();
    
    let identity_file: String = Input::new()
        .with_prompt("input IdentityFile path")
        .allow_empty(true)
        .interact_text()
        .unwrap();
    
    print!("input password: ");
    io::stdout().flush()?;
    let password = read_password()?;
    
    let proxy_command: String = Input::new()
        .with_prompt("input ProxyCommand")
        .allow_empty(true)
        .interact_text()
        .unwrap();
    
    let mut config = SshConfig::new(server_name.to_string());
    config.hostname = Some(hostname);
    config.user = Some(username);
    
    if !identity_file.is_empty() {
        config.identity_file = Some(identity_file);
    }
    if !password.is_empty() {
        config.password = Some(password);
    }
    if !proxy_command.is_empty() {
        config.proxy_command = Some(proxy_command);
    }
    
    add_ssh_config(config)?;
    println!("\nSSH configuration for '{}' has been added successfully!", server_name);
    
    Ok(())
}

fn sshct_show(server_name: &str) -> io::Result<()> {
    let config = find_config_by_host(server_name)?;
    
    let config = match config {
        Some(c) => c,
        None => {
            eprintln!("Error: Host '{}' not found in SSH config", server_name);
            std::process::exit(1);
        }
    };
    
    // 設定情報を表示
    println!("{}", config.to_config_string());
    
    // 接続確認
    if Confirm::new()
        .with_prompt("Would you like to connect?")
        .interact()
        .unwrap()
    {
        run_sshr(server_name)?;
    }
    
    Ok(())
}

fn sshct_edit(server_name: &str) -> io::Result<()> {
    let config = find_config_by_host(server_name)?;
    
    let mut config = match config {
        Some(c) => c,
        None => {
            eprintln!("Error: Host '{}' not found in SSH config", server_name);
            std::process::exit(1);
        }
    };
    
    println!("Editing SSH configuration for '{}'", server_name);
    println!("(Press Enter to keep current value)\n");
    
    // HostName
    let current_hostname = config.hostname.as_deref().unwrap_or("");
    let hostname: String = Input::new()
        .with_prompt(&format!("input host name [{}]", current_hostname))
        .allow_empty(true)
        .interact_text()
        .unwrap();
    if !hostname.is_empty() {
        config.hostname = Some(hostname);
    }
    
    // User
    let current_user = config.user.as_deref().unwrap_or("");
    let username: String = Input::new()
        .with_prompt(&format!("input user name [{}]", current_user))
        .allow_empty(true)
        .interact_text()
        .unwrap();
    if !username.is_empty() {
        config.user = Some(username);
    }
    
    // IdentityFile
    let current_identity = config.identity_file.as_deref().unwrap_or("");
    let identity_file: String = Input::new()
        .with_prompt(&format!("input IdentityFile path [{}]", current_identity))
        .allow_empty(true)
        .interact_text()
        .unwrap();
    if !identity_file.is_empty() {
        config.identity_file = Some(identity_file);
    }
    
    // Password
    let current_pass_display = if config.password.is_some() { "****" } else { "" };
    print!("input password [{}]: ", current_pass_display);
    io::stdout().flush()?;
    let password = read_password()?;
    if !password.is_empty() {
        config.password = Some(password);
    }
    
    // ProxyCommand
    let current_proxy = config.proxy_command.as_deref().unwrap_or("");
    let proxy_command: String = Input::new()
        .with_prompt(&format!("input ProxyCommand [{}]", current_proxy))
        .allow_empty(true)
        .interact_text()
        .unwrap();
    if !proxy_command.is_empty() {
        config.proxy_command = Some(proxy_command);
    }
    
    update_ssh_config(server_name, config)?;
    println!("\nSSH configuration for '{}' has been updated successfully!", server_name);
    
    Ok(())
}
