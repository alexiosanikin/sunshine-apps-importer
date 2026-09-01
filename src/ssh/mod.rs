//! SSH operations for remote file access.
//!
//! Uses the `ssh2` crate for SFTP-based file operations and command execution
//! via SSH sessions. Wraps synchronous operations in `spawn_blocking` for
//! async compatibility.

use ssh2::Session;
use std::io::{Read, Write};
use std::path::Path;

/// SSH connection configuration.
pub struct SshConfig {
    /// Remote host address.
    pub host: String,

    /// SSH port (typically 22).
    pub port: i32,

    /// Username for authentication.
    pub user: String,

    /// Password for password-based authentication (optional).
    pub password: Option<String>,

    /// Path to SSH private key for key-based authentication (optional).
    pub key_path: Option<String>,
}

impl SshConfig {
    /// Check if the SSH configuration is complete.
    #[must_use]
    pub fn is_configured(&self) -> bool {
        !self.host.is_empty() && !self.user.is_empty()
    }
}

/// SSH connection manager.
pub struct SshManager {
    config: SshConfig,
}

impl SshManager {
    /// Create a new SSH manager.
    #[must_use]
    pub fn new(config: SshConfig) -> Self {
        Self { config }
    }

    /// Establish an SSH session.
    fn connect(&self) -> Result<Session, anyhow::Error> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let mut session = Session::new()?;

        session.set_timeout(30_000);

        let tcp = tokio::task::block_in_place(|| std::net::TcpStream::connect(&addr))?;
        session.set_tcp_stream(tcp);
        session.handshake()?;

        // Authenticate
        let authenticated = if let Some(ref password) = self.config.password {
            tokio::task::block_in_place(|| {
                session
                    .userauth_password(&self.config.user, password)
                    .is_ok()
            })
        } else {
            false
        };

        if !authenticated {
            if let Some(ref key_path) = self.config.key_path {
                let home = dirs::home_dir()
                    .map(|h| h.to_string_lossy().to_string())
                    .unwrap_or_default();
                let expanded = key_path.replace("~", &home).replace("$HOME", &home);

                let path = Path::new(&expanded);
                tokio::task::block_in_place(|| {
                    let _ = session.userauth_pubkey_file(&self.config.user, None, path, None);
                });
            } else {
                anyhow::bail!("no authentication method provided");
            }
        }

        Ok(session)
    }

    /// Check if a file exists on the remote server.
    pub fn file_exists(&self, path: &str) -> Result<bool, anyhow::Error> {
        let session = self.connect()?;
        let sftp = session.sftp()?;
        Ok(sftp.stat(Path::new(path)).is_ok())
    }

    /// Check if a directory exists on the remote server.
    pub fn dir_exists(&self, path: &str) -> Result<bool, anyhow::Error> {
        let session = self.connect()?;
        let sftp = session.sftp()?;
        match sftp.stat(Path::new(path)) {
            Ok(meta) => Ok(meta.is_dir()),
            Err(_) => Ok(false),
        }
    }

    /// Read a file from the remote server.
    pub fn read_file(&self, path: &str) -> Result<Vec<u8>, anyhow::Error> {
        let session = self.connect()?;
        let sftp = session.sftp()?;
        let mut file = sftp.open(Path::new(path))?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        Ok(data)
    }

    /// Write data to a file on the remote server.
    pub fn write_file(&self, path: &str, data: &[u8]) -> Result<(), anyhow::Error> {
        let session = self.connect()?;
        let sftp = session.sftp()?;
        let mut file = sftp.create(Path::new(path))?;
        file.write_all(data)?;
        Ok(())
    }

    /// Create a directory on the remote server.
    pub fn create_dir(&self, path: &str) -> Result<(), anyhow::Error> {
        let session = self.connect()?;
        let sftp = session.sftp()?;
        sftp.mkdir(Path::new(path), 0o755)?;
        Ok(())
    }

    /// List files in a directory on the remote server.
    pub fn list_dir(&self, path: &str) -> Result<Vec<String>, anyhow::Error> {
        let session = self.connect()?;
        let sftp = session.sftp()?;
        let entries = sftp.readdir(Path::new(path))?;
        let mut names = Vec::new();
        for (path, _) in entries {
            names.push(
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
            );
        }
        Ok(names)
    }

    /// Execute a command on the remote server.
    pub fn execute_command(&self, command: &str) -> Result<String, anyhow::Error> {
        let session = self.connect()?;
        let mut channel = session.channel_session()?;
        channel.exec(command)?;

        let mut output = String::new();
        channel.read_to_string(&mut output)?;
        channel.close()?;

        Ok(output)
    }

    /// Create a backup of a remote file.
    pub fn create_backup(
        &self,
        remote_path: &str,
        suffix: &str,
    ) -> Result<Option<String>, anyhow::Error> {
        let backup_path = format!("{remote_path}.{suffix}");

        if !self.file_exists(remote_path)? {
            return Ok(None);
        }

        self.execute_command(&format!(
            "cp {} {}",
            shell_escape(remote_path),
            shell_escape(&backup_path)
        ))?;

        Ok(Some(backup_path))
    }

    /// Load apps.json content from a remote server.
    pub fn load_apps_json(&self, remote_path: &str) -> Result<Vec<u8>, anyhow::Error> {
        self.read_file(remote_path)
    }

    /// Save apps.json content to a remote server.
    pub fn save_apps_json(&self, content: &[u8], remote_path: &str) -> Result<(), anyhow::Error> {
        self.write_file(remote_path, content)
    }
}

/// Escape a string for safe use in shell commands.
#[must_use]
pub fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_escape() {
        assert_eq!(shell_escape("hello"), "'hello'");
        assert_eq!(shell_escape("it's"), "'it'\\''s'");
        assert_eq!(shell_escape("path/with spaces"), "'path/with spaces'");
    }

    #[test]
    fn test_ssh_config() {
        let config = SshConfig {
            host: "192.168.1.1".to_string(),
            port: 22,
            user: "user".to_string(),
            password: None,
            key_path: Some("~/.ssh/id_rsa".to_string()),
        };
        assert!(config.is_configured());

        let empty = SshConfig {
            host: String::new(),
            port: 0,
            user: String::new(),
            password: None,
            key_path: None,
        };
        assert!(!empty.is_configured());
    }
}
