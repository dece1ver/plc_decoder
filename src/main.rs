#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clipboard::{ClipboardContext, ClipboardProvider};
use core::f32;
use eframe::egui::{self, Button, FontData, ScrollArea, TextEdit};
use regex::Regex;
use std::str;
use std::time::{Duration, Instant};

struct HexConverterApp {
    input: String,
    output: String,
    status: String,
    status_time: Option<Instant>,
}
const ROW_COEFFICIENT: f32 = 14.0;
impl Default for HexConverterApp {
    fn default() -> Self {
        Self {
            input: String::new(),
            output: String::new(),
            status: String::new(),
            status_time: None,
        }
    }
}

impl eframe::App for HexConverterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("PLC Decoder");

            ui.add_space(5.0);

            let available_height = ui.available_height();
            let area_height = available_height * 0.5 - 35.0;
            ui.label("Ввод:");

            ui.horizontal(|ui| {
                ui.set_height(area_height);
                ui.push_id("input_scroll", |ui| {
                    ScrollArea::vertical()
                        .max_height(f32::INFINITY)
                        .show(ui, |ui| {
                            ui.add(
                                TextEdit::multiline(&mut self.input)
                                    .desired_width(ui.available_width() * 0.7)
                                    .desired_rows(
                                        (ui.available_height() / ROW_COEFFICIENT) as usize,
                                    )
                                    .hint_text("Введите сюда данные из PLC"),
                            );
                        });
                });

                if ui
                    .add_sized(
                        [ui.available_width(), ui.available_height()],
                        Button::new("Конвертировать"),
                    )
                    .clicked()
                {
                    if self.input.trim().is_empty() {
                        self.set_status("Ошибка: поле ввода пустое");
                    } else {
                        self.output = self.process_input();
                        if self.output.trim().is_empty() {
                            self.set_status("Ошибка: некорректный ввод");
                        } else {
                            self.set_status("Конвертация выполнена");
                        }
                    }
                }
            });

            ui.add_space(10.0);

            ui.label("Вывод:");

            ui.horizontal(|ui| {
                ui.set_height(area_height);
                ui.push_id("output_scroll", |ui| {
                    ScrollArea::vertical()
                        .max_height(f32::INFINITY)
                        .show(ui, |ui| {
                            ui.add(
                                TextEdit::multiline(&mut self.output)
                                    .desired_width(ui.available_width() * 0.7)
                                    .desired_rows(
                                        (ui.available_height() / ROW_COEFFICIENT) as usize,
                                    )
                                    .interactive(false)
                                    .hint_text("Тут будет вывод"),
                            );
                        });
                });

                if ui
                    .add_sized(
                        [ui.available_width(), ui.available_height()],
                        Button::new("Копировать в\nбуфер обмена"),
                    )
                    .clicked()
                {
                    if self.output.trim().is_empty() {
                        self.set_status("Ошибка: вывод пустой");
                    } else {
                        self.copy_output_to_clipboard();
                        self.set_status("Скопировано в буфер обмена");
                    }
                }
            });
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.label(&self.status);
                });
            });

            if let Some(time) = self.status_time {
                if time.elapsed() > Duration::from_secs(3) {
                    self.status.clear();
                    self.status_time = None;
                }
            };
        });
    }
}

impl HexConverterApp {
    fn copy_output_to_clipboard(&self) {
        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
        ctx.set_contents(self.output.clone()).unwrap();
    }

    fn set_status(&mut self, message: &str) {
        self.status = message.to_string();
        self.status_time = Some(Instant::now());
    }

    fn process_input(&self) -> String {
        let cleaned_strings = self.extract_and_clean_hex_strings(&self.input);
        if cleaned_strings.is_empty() {
            return String::new(); // Пустой результат, если не найдено строк
        }

        let mut result = String::new();
        for hex_string in cleaned_strings {
            let text = self.hex_to_text(&hex_string);
            result.push_str(&text);
            result.push_str("\n\n");
        }

        result
    }

    fn extract_and_clean_hex_strings(&self, text: &str) -> Vec<String> {
        let pattern = Regex::new(r"@04([0-9A-Fa-f]+)01@|@04([0-9A-Fa-f]+)02@").unwrap();
        pattern
            .captures_iter(text)
            .filter_map(|cap| cap.get(1).or(cap.get(2)))
            .map(|m| m.as_str().to_string())
            .collect()
    }

    fn hex_to_text(&self, hex_string: &str) -> String {
        let byte_array = match hex::decode(hex_string) {
            Ok(bytes) => bytes,
            Err(_) => return format!("Ошибка декодирования для: {}", hex_string),
        };

        let encodings = vec![
            ("gb2312", encoding_rs::GB18030),
            ("utf-8", encoding_rs::UTF_8),
            ("windows-1251", encoding_rs::WINDOWS_1251),
            ("latin-1", encoding_rs::WINDOWS_1252),
        ];

        for (_name, encoding) in encodings {
            if let Some(text) = encoding.decode(&byte_array).0.into_owned().into() {
                return text;
            }
        }
        format!("Ошибка декодирования для: {}", hex_string)
    }
}

fn main() {
    let app = HexConverterApp::default();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_resizable(true),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "PLC Decoder",
        native_options,
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "NotoSansSC".to_string(),
                FontData::from_static(include_bytes!(r"..\assets\NotoSansSC-Regular.otf")),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push("NotoSansSC".to_string());

            cc.egui_ctx.set_fonts(fonts);
            Ok(Box::new(app))
        }),
    );
}
