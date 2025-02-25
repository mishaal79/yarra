use std::fs::{self, File};
use std::io::{self, Read};
use std::path::PathBuf;
use crate::config;

const YARRA_MARKER: &str = "# Yarra Blocked Sites";
const YARRA_END_MARKER: &str = "# End Yarra Blocked Sites";

pub fn enable_blocking() -> io::Result<()> {
    let hosts_path = get_hosts_path();
    
    // Create backup if it doesn't exist
    let backup_path = hosts_path.with_extension("yarra.backup");
    if !backup_path.exists() {
        fs::copy(&hosts_path, &backup_path)?;
    }

    // Read current hosts file
    let mut content = String::new();
    File::open(&hosts_path)?.read_to_string(&mut content)?;

    // Remove any existing Yarra blocks
    if let Some(start) = content.find(YARRA_MARKER) {
        if let Some(end) = content.find(YARRA_END_MARKER) {
            content.replace_range(start..=end + YARRA_END_MARKER.len(), "");
        }
    }

    // Add new blocks
    let blocked_sites = config::load_blocked_sites().unwrap_or_default();
    if !blocked_sites.is_empty() {
        content.push_str("\n");
        content.push_str(YARRA_MARKER);
        content.push_str("\n");
        
        for site in blocked_sites {
            content.push_str(&format!("127.0.0.1\t{}\n", site));
            content.push_str(&format!("::1\t{}\n", site));  // IPv6 support
        }
        
        content.push_str(YARRA_END_MARKER);
        content.push_str("\n");
    }

    // Write back to hosts file
    fs::write(hosts_path, content.trim_end().as_bytes())?;
    
    Ok(())
}

pub fn disable_blocking() -> io::Result<()> {
    let hosts_path = get_hosts_path();
    let backup_path = hosts_path.with_extension("yarra.backup");
    
    if backup_path.exists() {
        fs::copy(&backup_path, &hosts_path)?;
        fs::remove_file(backup_path)?;
    } else {
        // If no backup exists, just remove Yarra blocks
        let mut content = String::new();
        File::open(&hosts_path)?.read_to_string(&mut content)?;
        
        if let Some(start) = content.find(YARRA_MARKER) {
            if let Some(end) = content.find(YARRA_END_MARKER) {
                content.replace_range(start..=end + YARRA_END_MARKER.len(), "");
                fs::write(hosts_path, content.trim_end().as_bytes())?;
            }
        }
    }
    
    Ok(())
}

fn get_hosts_path() -> PathBuf {
    if cfg!(test) {
        PathBuf::from(std::env::var("YARRA_TEST_HOSTS").unwrap_or_else(|_| "hosts.test".to_string()))
    } else if cfg!(target_os = "windows") {
        PathBuf::from(r"C:\Windows\System32\drivers\etc\hosts")
    } else {
        PathBuf::from("/etc/hosts")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    fn setup_test_hosts() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "127.0.0.1 localhost").unwrap();
        writeln!(file, "::1 localhost").unwrap();
        file
    }

    #[test]
    fn test_block_sites() -> io::Result<()> {
        let hosts_file = setup_test_hosts();
        std::env::set_var("YARRA_TEST_HOSTS", hosts_file.path());

        let temp_dir = tempfile::tempdir()?;
        std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        config::add_blocked_site("youtube.com").unwrap();

        enable_blocking()?;
        let content = fs::read_to_string(hosts_file.path())?;
        assert!(content.contains("youtube.com"));
        assert!(content.contains(YARRA_MARKER));

        disable_blocking()?;
        let content = fs::read_to_string(hosts_file.path())?;
        assert!(!content.contains("youtube.com"));
        assert!(!content.contains(YARRA_MARKER));

        Ok(())
    }
}