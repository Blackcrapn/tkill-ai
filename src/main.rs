mod config;
mod knowledge;
mod actions;
mod audio;
mod ai;

use iced::{
    Application, Command, Element, Length, Settings, Theme,
    widget::{Column, Container, TextInput, Button, Text, Row, Space, container},
    window, keyboard, event,
};
use iced::keyboard::Key;
use iced::event::Event;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    SendMessage,
    VoiceRecord,
    TranscribeResult(Result<String, String>),
    AiResponse(Result<Vec<ai::ToolCall>, String>),
    ExecuteTool(ai::ToolCall),
    ToolExecuted(Result<String, String>),
    ConfirmInstall(String),
    InstallConfirmed(bool),
    InstallResult(Result<String, String>),
    Close,
}

struct App {
    input: String,
    output: String,
    config: Arc<config::Config>,
    pending_install: Option<String>,
    loading: bool,
}

impl App {
    fn new() -> Self {
        let cfg = config::Config::load().unwrap_or_else(|e| {
            eprintln!("Ошибка конфигурации: {}", e);
            std::process::exit(1);
        });
        Self {
            input: String::new(),
            output: String::new(),
            config: Arc::new(cfg),
            pending_install: None,
            loading: false,
        }
    }

    async fn send_to_ai(input: String, token: String, model: String) -> Result<Vec<ai::ToolCall>, String> {
        ai::query_ai(&input, &token, &model).await
    }

    async fn execute_tool_call(tc: ai::ToolCall) -> Result<String, String> {
        let args: serde_json::Value = serde_json::from_str(&tc.function.arguments)
            .map_err(|e| format!("Ошибка парсинга аргументов: {}", e))?;

        match tc.function.name.as_str() {
            "launch_app" => {
                let app_name = args["app_name"].as_str().unwrap_or("");
                actions::launch_app(app_name).await
            }
            "run_hyprctl" => {
                let command = args["command"].as_str().unwrap_or("");
                actions::run_hyprctl(command).await
            }
            "search_web" => {
                let query = args["query"].as_str().unwrap_or("");
                actions::search_web(query).await
            }
            _ => Err(format!("Неизвестный tool: {}", tc.function.name)),
        }
    }
}

impl Application for App {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (Self::new(), Command::none())
    }

    fn title(&self) -> String {
        String::from("TKILL AI")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::InputChanged(val) => {
                self.input = val;
                Command::none()
            }
            Message::SendMessage => {
                if self.input.trim().is_empty() || self.loading {
                    return Command::none();
                }
                let input = self.input.clone();
                self.input.clear();
                self.loading = true;
                self.output = "⏳ Обрабатываю запрос...".to_string();
                let token = self.config.github.token.clone();
                let model = self.config.github.model.clone();
                Command::perform(
                    Self::send_to_ai(input, token, model),
                    Message::AiResponse,
                )
            }
            Message::VoiceRecord => {
                if self.loading { return Command::none(); }
                self.loading = true;
                self.output = "🎤 Запись... Говорите 5 секунд".to_string();
                Command::perform(audio::record_and_transcribe(), Message::TranscribeResult)
            }
            Message::TranscribeResult(res) => {
                self.loading = false;
                match res {
                    Ok(text) => {
                        self.input = text;
                        return self.update(Message::SendMessage);
                    }
                    Err(e) => {
                        self.output = format!("❌ Ошибка распознавания: {}", e);
                    }
                }
                Command::none()
            }
            Message::AiResponse(res) => {
                self.loading = false;
                match res {
                    Ok(tool_calls) => {
                        if tool_calls.is_empty() {
                            self.output = "🤖 ИИ не вернул действий. Попробуйте переформулировать.".to_string();
                            return Command::none();
                        }
                        let first = tool_calls[0].clone();
                        self.update(Message::ExecuteTool(first))
                    }
                    Err(e) => {
                        self.output = format!("❌ Ошибка ИИ: {}", e);
                        Command::none()
                    }
                }
            }
            Message::ExecuteTool(tc) => {
                self.loading = true;
                self.output = format!("🔧 Выполняю {}...", tc.function.name);
                Command::perform(Self::execute_tool_call(tc), Message::ToolExecuted)
            }
            Message::ToolExecuted(res) => {
                self.loading = false;
                match res {
                    Ok(msg) => {
                        self.output = format!("✅ {}", msg);
                    }
                    Err(err) => {
                        if err.starts_with("PACKAGE_NOT_INSTALLED:") {
                            let pkg = err.trim_start_matches("PACKAGE_NOT_INSTALLED:");
                            self.pending_install = Some(pkg.to_string());
                            self.output = format!("❓ Пакет '{}' не установлен. Установить?", pkg);
                        } else {
                            self.output = format!("❌ Ошибка: {}", err);
                        }
                    }
                }
                Command::none()
            }
            Message::ConfirmInstall(pkg) => {
                self.pending_install = Some(pkg);
                self.output = format!("❓ Установить пакет '{}'? (Да/Нет)", pkg);
                Command::none()
            }
            Message::InstallConfirmed(confirm) => {
                if let Some(pkg) = self.pending_install.take() {
                    if confirm {
                        self.loading = true;
                        self.output = format!("📦 Устанавливаю {}...", pkg);
                        let pkg_clone = pkg.clone();
                        return Command::perform(actions::install_package(&pkg_clone), Message::InstallResult);
                    } else {
                        self.output = "❌ Установка отменена.".to_string();
                    }
                }
                Command::none()
            }
            Message::InstallResult(res) => {
                self.loading = false;
                match res {
                    Ok(msg) => {
                        self.output = format!("✅ {}", msg);
                        if let Some(pkg) = &self.pending_install {
                            let pkg = pkg.clone();
                            self.pending_install = None;
                            self.loading = true;
                            self.output = format!("🚀 Запускаю {}...", pkg);
                            return Command::perform(actions::launch_app(&pkg), Message::ToolExecuted);
                        }
                    }
                    Err(e) => {
                        self.output = format!("❌ Ошибка установки: {}", e);
                    }
                }
                Command::none()
            }
            Message::Close => {
                return window::close(window::Id::MAIN);
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let input_field = TextInput::new("Напишите или скажите команду...", &self.input)
            .on_input(Message::InputChanged)
            .on_submit(Message::SendMessage)
            .padding(10)
            .size(20);

        let voice_button = Button::new(Text::new("🎤 Voice"))
            .on_press(Message::VoiceRecord)
            .padding(8);

        let send_button = Button::new(Text::new("➤"))
            .on_press(Message::SendMessage)
            .padding(8);

        let output_text = Text::new(&self.output)
            .size(16)
            .color(iced::Color::from_rgb(0.8, 0.8, 0.8));

        let row = Row::with_children(vec![
            input_field.into(),
            voice_button.into(),
            send_button.into(),
        ])
        .spacing(10)
        .align_items(iced::Alignment::Center);

        let content = Column::with_children(vec![
            row.into(),
            Space::with_height(10).into(),
            output_text.into(),
        ])
        .padding(iced::Padding::from(20))
        .spacing(10)
        .width(Length::Fill);

        let container = Container::new(content)
            .width(Length::Units(800))
            .height(Length::Units(120))
            .center_x()
            .center_y()
            .style(iced::theme::Container::Custom(Box::new(GlassStyle)));

        container.into()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::keyboard::on_key_press(|key, _modifiers| {
            if key == Key::Character("Escape".into()) {
                Some(Message::Close)
            } else {
                None
            }
        })
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

struct GlassStyle;

impl iced::container::StyleSheet for GlassStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::container::Appearance {
        iced::container::Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgba(0.1, 0.2, 0.5, 0.7))),
            border_radius: 15.0.into(),
            border_width: 1.0,
            border_color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.3),
            ..Default::default()
        }
    }
}

fn main() -> iced::Result {
    App::run(Settings {
        window: window::Settings {
            size: iced::Size::new(800.0, 120.0),
            decorations: false,
            transparent: true,
            ..Default::default()
        },
        ..Default::default()
    })
}
