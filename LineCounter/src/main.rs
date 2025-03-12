// Утиліта підрахунку рядків (Line Count Utility)
// Програма для підрахунку кількості рядків у текстовому файлі
// Варіант 20

use iced::{
    button, Application, Button, Color, Column, Command, Container, Element,
    HorizontalAlignment, Length, Row, Settings, Text, TextInput, executor,
    text_input, window, Clipboard, Align,
};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use rfd::FileDialog;

// Визначення користувацьких кольорів для приємного дизайну
const PRIMARY_COLOR: Color = Color::from_rgb(0.27, 0.52, 0.90);

const TEXT_COLOR: Color = Color::from_rgb(0.2, 0.2, 0.2);
const BUTTON_COLOR: Color = Color::from_rgb(0.36, 0.61, 0.90);
const BACKGROUND_COLOR: Color = Color::from_rgb(0.95, 0.95, 0.97);

// Структура, яка представляє стан нашої програми
pub struct LineCountApp {
    // Шлях до файлу
    file_path: String,
    // Кількість рядків у файлі
    line_count: Option<usize>,
    // Стани для кнопок
    open_button: button::State,
    count_button: button::State,
    copy_button: button::State,
    // Статус операції
    status_message: String,
    // Стан для текстового поля
    file_path_input: text_input::State,
}

// Повідомлення для обробки подій у програмі
#[derive(Debug, Clone)]
pub enum Message {
    // Відкрити діалог вибору файлу
    OpenFileDialog,
    // Встановити шлях до файлу
    FilePathChanged(String),
    // Підрахувати кількість рядків
    CountLines,
    // Скопіювати результат в буфер обміну
    CopyResult,
}

// Стиль кнопок
struct ButtonStyle;

impl button::StyleSheet for ButtonStyle {
    fn active(&self) -> button::Style {
        button::Style {
            background: Some(BUTTON_COLOR.into()),
            border_radius: 5.0,
            text_color: Color::WHITE,
            shadow_offset: iced::Vector::new(0.0, 1.0),
            ..button::Style::default()
        }
    }

    fn hovered(&self) -> button::Style {
        button::Style {
            background: Some(Color {
                r: BUTTON_COLOR.r - 0.05,
                g: BUTTON_COLOR.g - 0.05,
                b: BUTTON_COLOR.b - 0.05,
                a: BUTTON_COLOR.a,
            }.into()),
            shadow_offset: iced::Vector::new(0.0, 2.0),
            ..self.active()
        }
    }

    fn pressed(&self) -> button::Style {
        button::Style {
            shadow_offset: iced::Vector::new(0.0, 0.0),
            ..self.hovered()
        }
    }
}

// Стиль контейнера
struct ContainerStyle;

impl iced::container::StyleSheet for ContainerStyle {
    fn style(&self) -> iced::container::Style {
        iced::container::Style {
            background: Some(BACKGROUND_COLOR.into()),
            text_color: Some(TEXT_COLOR),
            ..iced::container::Style::default()
        }
    }
}

impl Application for LineCountApp {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
                file_path: String::new(),
                line_count: None,
                open_button: button::State::new(),
                count_button: button::State::new(),
                copy_button: button::State::new(),
                status_message: String::from("Готово до роботи"),
                file_path_input: text_input::State::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Утиліта підрахунку рядків")
    }

    fn update(&mut self, message: Message, clipboard: &mut Clipboard) -> Command<Message> {
        match message {
            Message::OpenFileDialog => {
                // Команда для виконання в окремому потоці, щоб не блокувати інтерфейс
                return Command::perform(
                    async {
                        // Використовуємо FileDialog для вибору файлу
                        if let Some(file) = FileDialog::new()
                            .add_filter("Текстові файли", &["txt", "rs", "py", "c", "cpp", "h", "html", "css", "js"])
                            .add_filter("Усі файли", &["*"])
                            .set_title("Виберіть текстовий файл")
                            .pick_file() {
                            return file.to_string_lossy().to_string();
                        }
                        String::new()
                    },
                    |file_path| {
                        if !file_path.is_empty() {
                            Message::FilePathChanged(file_path)
                        } else {
                            // Якщо файл не вибрано, залишаємо все як є
                            Message::FilePathChanged(String::new())
                        }
                    },
                );
            }
            Message::FilePathChanged(path) => {
                if !path.is_empty() {
                    self.file_path = path;
                    self.status_message = format!("Файл вибрано: {}", self.file_path);
                    self.line_count = None;
                }
            }
            Message::CountLines => {
                if self.file_path.is_empty() {
                    self.status_message = String::from("Помилка: спочатку виберіть файл");
                } else {
                    // Спробуємо підрахувати кількість рядків
                    match count_lines_in_file(&self.file_path) {
                        Ok(count) => {
                            self.line_count = Some(count);
                            self.status_message = format!("Підраховано рядків: {}", count);
                        }
                        Err(e) => {
                            self.status_message = format!("Помилка при підрахунку рядків: {}", e);
                            self.line_count = None;
                        }
                    }
                }
            }
            Message::CopyResult => {
                if let Some(count) = self.line_count {
                    let result_text = format!("Файл: {}\nКількість рядків: {}", self.file_path, count);
                    clipboard.write(result_text);
                    self.status_message = String::from("Результат скопійовано в буфер обміну");
                } else {
                    self.status_message = String::from("Немає результату для копіювання");
                }
            }
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        // Заголовок
        let title = Text::new("Утиліта підрахунку рядків")
            .size(32)
            .color(PRIMARY_COLOR)
            .horizontal_alignment(HorizontalAlignment::Center);

        // Опис
        let description = Text::new("Програма для підрахунку кількості рядків у текстовому файлі")
            .size(16)
            .color(TEXT_COLOR);

        // Поле для відображення шляху до файлу
        let file_path_input = TextInput::new(
            &mut self.file_path_input,
            "Шлях до файлу...",
            &self.file_path,
            Message::FilePathChanged,
        )
            .padding(10)
            .size(18);

        // Кнопка для відкриття діалогу вибору файлу
        let open_button = Button::new(
            &mut self.open_button,
            Text::new("Вибрати файл").size(16),
        )
            .on_press(Message::OpenFileDialog)
            .padding(10)
            .style(ButtonStyle);

        // Створюємо рядок з полем вводу та кнопкою вибору файлу
        let file_row = Row::new()
            .spacing(10)
            .push(file_path_input)
            .push(open_button)
            .width(Length::Fill);

        // Кнопка для підрахунку рядків
        let count_button = Button::new(
            &mut self.count_button,
            Text::new("Підрахувати рядки").size(16),
        )
            .on_press(Message::CountLines)
            .padding(10)
            .width(Length::Fill)
            .style(ButtonStyle);

        // Текст з результатом
        let result = if let Some(count) = self.line_count {
            Text::new(format!("Кількість рядків: {}", count))
                .size(24)
                .color(PRIMARY_COLOR)
        } else {
            Text::new("Результат буде відображено тут")
                .size(18)
                .color(Color::from_rgb(0.5, 0.5, 0.5))
        };

        // Кнопка для копіювання результату
        let copy_button = Button::new(
            &mut self.copy_button,
            Text::new("Копіювати результат").size(16),
        )
            .on_press(Message::CopyResult)
            .padding(10)
            .width(Length::Fill)
            .style(ButtonStyle);

        // Текст зі статусом
        let status = Text::new(&self.status_message)
            .size(14)
            .color(if self.status_message.starts_with("Помилка") {
                Color::from_rgb(0.8, 0.0, 0.0)
            } else {
                Color::from_rgb(0.0, 0.5, 0.0)
            });

        // Збираємо всі елементи в одну колонку
        let content = Column::new()
            .spacing(20)
            .padding(20)
            .max_width(600)
            .align_items(Align::Center)
            .push(title)
            .push(description)
            .push(file_row)
            .push(count_button)
            .push(result)
            .push(copy_button)
            .push(status);

        // Поміщаємо контент у контейнер із фоном
        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(ContainerStyle)
            .into()
    }
}

// Функція для підрахунку кількості рядків у файлі
fn count_lines_in_file<P: AsRef<Path>>(path: P) -> io::Result<usize> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(reader.lines().count())
}

fn main() -> iced::Result {
    // Запуск програми з конфігурацією вікна
    LineCountApp::run(Settings {
        window: window::Settings {
            size: (800, 600),
            resizable: true,
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}