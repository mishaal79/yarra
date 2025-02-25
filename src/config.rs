use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub blocked_sites: Vec<String>,
}

pub fn config_path() -> Result<PathBuf, std::io::Error> {
    dirs::config_dir()
        .ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Failed to find config directory"
        ))
        .map(|p| p.join("yarra/config.toml"))
}

pub fn load_blocked_sites() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let config = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&config)?;
    Ok(config.blocked_sites)
}

pub fn add_blocked_site(site: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config {
        blocked_sites: load_blocked_sites()?,
    };
    
    if !config.blocked_sites.contains(&site.to_string()) {
        config.blocked_sites.push(site.to_string());
    }

    let path = config_path()?;
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(&path, toml::to_string(&config)?)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_add_blocked_site() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());

        add_blocked_site("youtube.com")?;
        let sites = load_blocked_sites()?;
        assert!(sites.contains(&"youtube.com".to_string()));

        Ok(())
    }
}