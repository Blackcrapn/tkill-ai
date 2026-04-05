use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub github: GithubConfig,
    pub audio: AudioConfig,
    pub behavior: BehaviorConfig,
}

#[derive(Debug, Deserialize)]
pub struct GithubConfig {
    pub token: String,
    pub model: String,
}

#[derive(Debug, Deserialize)]
pub struct AudioConfig {
    pub input_device: String,
}

#[derive(Debug, Deserialize)]
pub struct BehaviorConfig {
    pub log_level: String,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let home = dirs::home_dir().ok_or("Не удалось определить домашнюю директорию")?;
        let config_path = home.join(".config/tkill-ai/config.toml");

        let contents = fs::read_to_string(&config_path)
            .map_err(|e| format!("Не удалось прочитать конфиг {}: {}", config_path.display(), e))?;

        let config: Config = toml::from_str(&contents)
            .map_err(|e| format!("Ошибка парсинга конфига: {}", e))?;

        if config.github.token == "YOUR_TOKEN_HERE" || config.github.token.is_empty() {
            return Err("Токен GitHub не настроен. Отредактируйте ~/.config/tkill-ai/config.toml".into());
        }

        Ok(config)
    }
}
