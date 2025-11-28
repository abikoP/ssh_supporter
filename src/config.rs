use regex::Regex;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SshConfig {
    pub host: String,
    pub hostname: Option<String>,
    pub user: Option<String>,
    pub port: Option<String>,
    pub identity_file: Option<String>,
    pub password: Option<String>,
    pub proxy_command: Option<String>,
}

impl SshConfig {
    pub fn new(host: String) -> Self {
        Self {
            host,
            hostname: None,
            user: None,
            port: None,
            identity_file: None,
            password: None,
            proxy_command: None,
        }
    }

    pub fn to_config_string(&self) -> String {
        let mut config = format!("Host {}\n", self.host);
        
        if let Some(hostname) = &self.hostname {
            config.push_str(&format!("  HostName {}\n", hostname));
        }
        if let Some(user) = &self.user {
            config.push_str(&format!("  User {}\n", user));
        }
        if let Some(port) = &self.port {
            config.push_str(&format!("  Port {}\n", port));
        }
        if let Some(identity_file) = &self.identity_file {
            config.push_str(&format!("  IdentityFile {}\n", identity_file));
        }
        if let Some(proxy_command) = &self.proxy_command {
            config.push_str(&format!("  ProxyCommand {}\n", proxy_command));
        }
        if let Some(password) = &self.password {
            config.push_str(&format!("  #pass {}\n", password));
        }
        
        config
    }
}

pub fn get_ssh_config_path() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME environment variable not set");
    PathBuf::from(home).join(".ssh").join("config")
}

pub fn parse_ssh_config() -> io::Result<Vec<SshConfig>> {
    let config_path = get_ssh_config_path();
    
    if !config_path.exists() {
        return Ok(Vec::new());
    }
    
    let content = fs::read_to_string(&config_path)?;
    let mut configs = Vec::new();
    let mut current_config: Option<SshConfig> = None;
    
    // 大文字小文字を区別しない正規表現に変更
    let host_re = Regex::new(r"(?i)^Host\s+(.+)$").unwrap();
    let hostname_re = Regex::new(r"(?i)^\s*HostName\s+(.+)$").unwrap();
    let user_re = Regex::new(r"(?i)^\s*User\s+(.+)$").unwrap();
    let port_re = Regex::new(r"(?i)^\s*Port\s+(.+)$").unwrap();
    let identity_re = Regex::new(r"(?i)^\s*IdentityFile\s+(.+)$").unwrap();
    let proxy_re = Regex::new(r"(?i)^\s*ProxyCommand\s+(.+)$").unwrap();
    let pass_re = Regex::new(r"(?i)^\s*#pass\s+(.+)$").unwrap();
    
    for line in content.lines() {
        if let Some(caps) = host_re.captures(line) {
            if let Some(config) = current_config.take() {
                configs.push(config);
            }
            current_config = Some(SshConfig::new(caps[1].trim().to_string()));
        } else if let Some(config) = current_config.as_mut() {
            if let Some(caps) = hostname_re.captures(line) {
                config.hostname = Some(caps[1].trim().to_string());
            } else if let Some(caps) = user_re.captures(line) {
                config.user = Some(caps[1].trim().to_string());
            } else if let Some(caps) = port_re.captures(line) {
                config.port = Some(caps[1].trim().to_string());
            } else if let Some(caps) = identity_re.captures(line) {
                config.identity_file = Some(caps[1].trim().to_string());
            } else if let Some(caps) = proxy_re.captures(line) {
                config.proxy_command = Some(caps[1].trim().to_string());
            } else if let Some(caps) = pass_re.captures(line) {
                config.password = Some(caps[1].trim().to_string());
            }
        }
    }
    
    if let Some(config) = current_config {
        configs.push(config);
    }
    
    Ok(configs)
}

pub fn find_config_by_host(host: &str) -> io::Result<Option<SshConfig>> {
    let configs = parse_ssh_config()?;
    Ok(configs.into_iter().find(|c| c.host == host))
}

// 既存のwrite_ssh_configは削除し、より安全な編集ロジックを使用する

pub fn add_ssh_config(new_config: SshConfig) -> io::Result<()> {
    let config_path = get_ssh_config_path();
    
    // .sshディレクトリが存在しない場合は作成
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config_path)?;
    
    // ファイルが空でない場合は改行を追加
    if file.metadata()?.len() > 0 {
        writeln!(file)?;
    }
    
    write!(file, "{}", new_config.to_config_string())?;
    
    Ok(())
}

pub fn update_ssh_config(host: &str, updated_config: SshConfig) -> io::Result<bool> {
    let config_path = get_ssh_config_path();
    if !config_path.exists() {
        return Ok(false);
    }
    
    let content = fs::read_to_string(&config_path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut new_lines = Vec::new();
    
    let host_re = Regex::new(r"(?i)^Host\s+(.+)$").unwrap();
    let mut in_target_host = false;
    let mut host_found = false;
    
    // 更新対象のフィールドを追跡
    let mut updated_hostname = false;
    let mut updated_user = false;
    let mut updated_port = false;
    let mut updated_identity = false;
    let mut updated_proxy = false;
    let mut updated_password = false;
    
    for line in lines.iter() {
        if let Some(caps) = host_re.captures(line) {
            let current_host = caps[1].trim();
            if current_host == host {
                in_target_host = true;
                host_found = true;
                new_lines.push(line.clone());
                continue;
            } else {
                // ターゲットホストのブロックが終わった場合
                if in_target_host {
                    // まだ追加されていないフィールドがあれば追加
                    append_missing_fields(&mut new_lines, &updated_config, 
                        updated_hostname, updated_user, updated_port, 
                        updated_identity, updated_proxy, updated_password);
                }
                in_target_host = false;
            }
        }
        
        if in_target_host {
            // フィールドの更新ロジック
            // 各行がどのフィールドか判定し、更新対象なら値を書き換える
            // 更新対象でない（コメントや未知のフィールド）ならそのまま残す
            
            let lower_line = line.to_lowercase();
            let trimmed = lower_line.trim_start();
            
            if trimmed.starts_with("hostname ") {
                if let Some(val) = &updated_config.hostname {
                    new_lines.push(format!("  HostName {}", val));
                    updated_hostname = true;
                } else {
                    // 値がNoneなら行を削除（追加しない）
                }
            } else if trimmed.starts_with("user ") {
                if let Some(val) = &updated_config.user {
                    new_lines.push(format!("  User {}", val));
                    updated_user = true;
                }
            } else if trimmed.starts_with("port ") {
                if let Some(val) = &updated_config.port {
                    new_lines.push(format!("  Port {}", val));
                    updated_port = true;
                }
            } else if trimmed.starts_with("identityfile ") {
                if let Some(val) = &updated_config.identity_file {
                    new_lines.push(format!("  IdentityFile {}", val));
                    updated_identity = true;
                }
            } else if trimmed.starts_with("proxycommand ") {
                if let Some(val) = &updated_config.proxy_command {
                    new_lines.push(format!("  ProxyCommand {}", val));
                    updated_proxy = true;
                }
            } else if trimmed.starts_with("#pass ") {
                if let Some(val) = &updated_config.password {
                    new_lines.push(format!("  #pass {}", val));
                    updated_password = true;
                }
            } else {
                // その他の行（コメント、未知の設定など）はそのまま保持
                new_lines.push(line.clone());
            }
        } else {
            // ターゲットホスト以外の行はそのまま保持
            new_lines.push(line.clone());
        }
    }
    
    // ファイル末尾がターゲットホストだった場合の処理
    if in_target_host {
        append_missing_fields(&mut new_lines, &updated_config, 
            updated_hostname, updated_user, updated_port, 
            updated_identity, updated_proxy, updated_password);
    }
    
    if host_found {
        let mut file = fs::File::create(&config_path)?;
        for line in new_lines {
            writeln!(file, "{}", line)?;
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

fn append_missing_fields(
    lines: &mut Vec<String>, 
    config: &SshConfig,
    has_hostname: bool,
    has_user: bool,
    has_port: bool,
    has_identity: bool,
    has_proxy: bool,
    has_password: bool
) {
    if !has_hostname {
        if let Some(val) = &config.hostname {
            lines.push(format!("  HostName {}", val));
        }
    }
    if !has_user {
        if let Some(val) = &config.user {
            lines.push(format!("  User {}", val));
        }
    }
    if !has_port {
        if let Some(val) = &config.port {
            lines.push(format!("  Port {}", val));
        }
    }
    if !has_identity {
        if let Some(val) = &config.identity_file {
            lines.push(format!("  IdentityFile {}", val));
        }
    }
    if !has_proxy {
        if let Some(val) = &config.proxy_command {
            lines.push(format!("  ProxyCommand {}", val));
        }
    }
    if !has_password {
        if let Some(val) = &config.password {
            lines.push(format!("  #pass {}", val));
        }
    }
}
