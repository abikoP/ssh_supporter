use crate::config::find_config_by_host;
use std::env;
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

pub fn run_sshr(host: &str) -> io::Result<()> {
    // SSH configからホスト情報を取得
    let config = find_config_by_host(host)?;
    
    let config = match config {
        Some(c) => c,
        None => {
            eprintln!("Error: Host '{}' not found in SSH config", host);
            std::process::exit(1);
        }
    };
    
    // パスワードがある場合は自動入力、ない場合は通常のSSH接続
    if let Some(password) = &config.password {
        run_ssh_with_password(host, password)
    } else {
        // パスワードがない場合は通常のsshコマンドを実行
        let status = Command::new("ssh")
            .arg(host)
            .status()?;
        
        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }
        Ok(())
    }
}

fn run_ssh_with_password(host: &str, password: &str) -> io::Result<()> {
    // expectが利用可能かチェック
    let expect_check = Command::new("which")
        .arg("expect")
        .output();
    
    match expect_check {
        Ok(output) if output.status.success() => {
            // expectスクリプトを使用
            run_with_expect_script(host, password)
        }
        _ => {
            // expectが利用できない場合はエラー
            eprintln!("Error: 'expect' command is not installed.");
            eprintln!("Please install expect to use password authentication:");
            eprintln!("  macOS: brew install expect");
            eprintln!("  Linux: sudo apt-get install expect (Debian/Ubuntu)");
            eprintln!("         sudo yum install expect (RHEL/CentOS)");
            std::process::exit(1);
        }
    }
}

fn run_with_expect_script(host: &str, password: &str) -> io::Result<()> {
    // expectスクリプトの内容
    let script_content = format!(
        r#"#!/usr/bin/env expect -f

set timeout 30

# SSH接続を開始
spawn ssh {}

# パスワードまたはパスフレーズのプロンプトを待つ
expect {{
    -re "(?i)(password|passphrase).*:" {{
        send "{}\r"
        exp_continue
    }}
    -re "(?i)yes/no" {{
        send "yes\r"
        exp_continue
    }}
    eof {{
        exit 0
    }}
    timeout {{
        puts "Connection timeout"
        exit 1
    }}
}}

# インタラクティブモードに移行
interact
"#,
        host, password
    );
    
    // 一時ファイルにスクリプトを書き込む
    let temp_dir = env::temp_dir();
    let script_path = temp_dir.join(format!("ssh_expect_{}.exp", std::process::id()));
    
    fs::write(&script_path, script_content)?;
    
    // スクリプトに実行権限を付与
    let mut perms = fs::metadata(&script_path)?.permissions();
    perms.set_mode(0o700);
    fs::set_permissions(&script_path, perms)?;
    
    // expectスクリプトを実行
    let status = Command::new("expect")
        .arg(&script_path)
        .status()?;
    
    // 一時ファイルを削除
    let _ = fs::remove_file(&script_path);
    
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    
    Ok(())
}
