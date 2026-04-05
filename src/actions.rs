use std::process::Command;
use tokio::process::Command as TokioCommand;
use which::which;

/// Запуск приложения: проверка наличия, установка через диалог (вызывается из UI)
pub async fn launch_app(app_name: &str) -> Result<String, String> {
    let pkg = crate::knowledge::lookup_package(app_name)
        .ok_or_else(|| format!("Приложение '{}' не найдено в базе знаний", app_name))?;

    let installed = check_package_installed(pkg);
    if !installed {
        return Err(format!("PACKAGE_NOT_INSTALLED:{}", pkg));
    }

    run_command(pkg)
}

pub async fn install_package(pkg: &str) -> Result<String, String> {
    let status = TokioCommand::new("sudo")
        .arg("pacman")
        .arg("-S")
        .arg("--noconfirm")
        .arg(pkg)
        .status()
        .await
        .map_err(|e| format!("Ошибка установки: {}", e))?;

    if status.success() {
        Ok(format!("Пакет '{}' успешно установлен", pkg))
    } else {
        Err(format!("Не удалось установить '{}'", pkg))
    }
}

pub async fn run_hyprctl(command: &str) -> Result<String, String> {
    let output = TokioCommand::new("hyprctl")
        .arg("dispatch")
        .arg(command)
        .output()
        .await
        .map_err(|e| format!("Ошибка hyprctl: {}", e))?;

    if output.status.success() {
        let out = String::from_utf8_lossy(&output.stdout);
        Ok(format!("hyprctl выполнен: {}", out))
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        Err(format!("hyprctl ошибка: {}", err))
    }
}

pub async fn search_web(query: &str) -> Result<String, String> {
    let url = format!("https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1", urlencoding::encode(query));
    let resp = reqwest::get(&url)
        .await
        .map_err(|e| format!("Ошибка запроса: {}", e))?;
    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Ошибка парсинга: {}", e))?;

    let abstract_text = json["AbstractText"].as_str().unwrap_or("");
    let definition = json["Definition"].as_str().unwrap_or("");
    let answer = json["Answer"].as_str().unwrap_or("");

    let result = if !abstract_text.is_empty() {
        abstract_text.to_string()
    } else if !definition.is_empty() {
        definition.to_string()
    } else if !answer.is_empty() {
        answer.to_string()
    } else {
        "Не удалось найти краткую информацию по запросу.".to_string()
    };
    Ok(result)
}

fn check_package_installed(pkg: &str) -> bool {
    which(pkg).is_ok() || Command::new("pacman")
        .args(&["-Q", pkg])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn run_command(pkg: &str) -> Result<String, String> {
    let status = Command::new(pkg)
        .spawn()
        .map_err(|e| format!("Не удалось запустить '{}': {}", pkg, e))?
        .wait()
        .map_err(|e| format!("Ошибка ожидания процесса: {}", e))?;

    if status.success() {
        Ok(format!("Приложение '{}' запущено", pkg))
    } else {
        Err(format!("Приложение '{}' завершилось с ошибкой", pkg))
    }
}
