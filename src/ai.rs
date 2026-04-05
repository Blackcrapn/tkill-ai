use serde::{Deserialize, Serialize};
use serde_json::json;
use reqwest::Client;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

pub async fn query_ai(
    user_input: &str,
    token: &str,
    model: &str,
) -> Result<Vec<ToolCall>, String> {
    let client = Client::new();
    let url = "https://models.inference.ai.azure.com/chat/completions";

    let tools = json!([
        {
            "type": "function",
            "function": {
                "name": "launch_app",
                "description": "Запускает приложение (или устанавливает, если не найдено)",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "app_name": { "type": "string", "description": "Название приложения (на русском или английском)" }
                    },
                    "required": ["app_name"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "run_hyprctl",
                "description": "Выполняет команду hyprctl dispatch",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": { "type": "string", "description": "Команда для hyprctl dispatch (например, 'killactive', 'workspace 2')" }
                    },
                    "required": ["command"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "search_web",
                "description": "Ищет информацию в интернете (DuckDuckGo) и возвращает краткую выжимку",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Поисковый запрос" }
                    },
                    "required": ["query"]
                }
            }
        }
    ]);

    let messages = json!([
        {
            "role": "system",
            "content": "Ты ассистент, который управляет компьютером. Если пользователь просит запустить программу, используй функцию launch_app. Если просит выполнить hyprctl команду — run_hyprctl. Если нужна информация из интернета — search_web. Отвечай только вызовом функции (одной или несколькими)."
        },
        {
            "role": "user",
            "content": user_input
        }
    ]);

    let request_body = json!({
        "messages": messages,
        "model": model,
        "tools": tools,
        "tool_choice": "auto",
    });

    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Ошибка запроса к GitHub Models: {}", e))?;

    if !response.status().is_success() {
        let err_text = response.text().await.unwrap_or_default();
        return Err(format!("API вернул ошибку: {}", err_text));
    }

    let json_resp: serde_json::Value = response.json().await.map_err(|e| format!("Ошибка парсинга ответа: {}", e))?;
    let tool_calls = json_resp["choices"][0]["message"]["tool_calls"]
        .as_array()
        .ok_or("Нет вызовов функций в ответе")?;

    let calls: Vec<ToolCall> = serde_json::from_value(json!(tool_calls))
        .map_err(|e| format!("Ошибка десериализации tool_calls: {}", e))?;

    Ok(calls)
}
