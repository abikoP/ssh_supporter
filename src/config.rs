use regex::Regex;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SshConfig {
    pub host: String,
    pub hostname: Option<String>,
    pub user: Option<String>,
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
    
    let host_re = Regex::new(r"^Host\s+(.+)$").unwrap();
    let hostname_re = Regex::new(r"^\s*HostName\s+(.+)$").unwrap();
    let user_re = Regex::new(r"^\s*User\s+(.+)$").unwrap();
    let identity_re = Regex::new(r"^\s*IdentityFile\s+(.+)$").unwrap();
    let proxy_re = Regex::new(r"^\s*ProxyCommand\s+(.+)$").unwrap();
    let pass_re = Regex::new(r"^\s*#pass\s+(.+)$").unwrap();
    
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

pub fn write_ssh_config(configs: &[SshConfig]) -> io::Result<()> {
    let config_path = get_ssh_config_path();
    
    // .sshディレクトリが存在しない場合は作成
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    let mut content = String::new();
    for (i, config) in configs.iter().enumerate() {
        if i > 0 {
            content.push('\n');
        }
        content.push_str(&config.to_config_string());
    }
    
    let mut file = fs::File::create(&config_path)?;
    file.write_all(content.as_bytes())?;
    
    Ok(())
}

pub fn add_ssh_config(new_config: SshConfig) -> io::Result<()> {
    let mut configs = parse_ssh_config()?;
    configs.push(new_config);
    write_ssh_config(&configs)
}

pub fn update_ssh_config(host: &str, updated_config: SshConfig) -> io::Result<bool> {
    let mut configs = parse_ssh_config()?;
    
    if let Some(pos) = configs.iter().position(|c| c.host == host) {
        configs[pos] = updated_config;
        write_ssh_config(&configs)?;
        Ok(true)
    } else {
        Ok(false)
    }
}
