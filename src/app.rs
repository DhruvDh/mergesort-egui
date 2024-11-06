#![warn(clippy::all)]

use eframe::egui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
enum CheckpointStatus {
    NotStarted,
    InProgress,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Checkpoint {
    id: String,
    description: String,
    status: CheckpointStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub content: String,
    pub from_user: bool, // true if it's a user's message
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct LearningApp {
    label: String,
    checkpoints: Vec<Checkpoint>,
    reset_modal_open: bool, // State variable for managing reset confirmation modal
    chat_history: Vec<ChatMessage>,
    #[serde(skip)] // Caches don't need to be serialized
    message_caches: Vec<CommonMarkCache>,
    current_input: String,
}

impl Default for LearningApp {
    fn default() -> Self {
        Self {
            label: "Hello World!".to_owned(),
            checkpoints: vec![
                Checkpoint {
                    id: "inefficiency_discovery".to_string(),
                    description: "Understanding sorting inefficiency".to_string(),
                    status: CheckpointStatus::InProgress,
                },
                Checkpoint {
                    id: "splitting_insight".to_string(),
                    description: "Discovering divide-and-conquer benefit".to_string(),
                    status: CheckpointStatus::NotStarted,
                },
                Checkpoint {
                    id: "merging_development".to_string(),
                    description: "Understanding systematic merging".to_string(),
                    status: CheckpointStatus::NotStarted,
                },
                Checkpoint {
                    id: "recursive_pattern".to_string(),
                    description: "Grasping recursive nature".to_string(),
                    status: CheckpointStatus::NotStarted,
                },
                Checkpoint {
                    id: "efficiency_analysis".to_string(),
                    description: "Comprehending O(n log n) complexity".to_string(),
                    status: CheckpointStatus::NotStarted,
                },
            ],
            reset_modal_open: false,
            chat_history: Vec::new(),
            message_caches: Vec::new(),
            current_input: String::new(),
        }
    }
}

impl LearningApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            let mut app: Self = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            // Initialize caches for any existing messages
            app.message_caches = Vec::with_capacity(app.chat_history.len());
            for _ in 0..app.chat_history.len() {
                app.message_caches.push(CommonMarkCache::default());
            }
            return app;
        }
        Default::default()
    }

    fn reset_to_default(&mut self) {
        *self = Default::default();
    }

    fn render_side_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new("Learning Progress")
                    .size(18.0)
                    .heading(),
            );
            ui.add_space(8.0);
            // Render checkpoints
            self.checkpoints
                .iter()
                .enumerate()
                .for_each(|(index, checkpoint)| {
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            // Show number + description or ???
                            let display_text = match checkpoint.status {
                                CheckpointStatus::Completed => {
                                    format!("{}. {} ✔", index + 1, checkpoint.description)
                                }
                                CheckpointStatus::InProgress => {
                                    format!("{}.   ? ? ?  (In Progress... ⏳)", index + 1)
                                }
                                CheckpointStatus::NotStarted => {
                                    format!("{}.   ? ? ?", index + 1)
                                }
                            };

                            ui.label(egui::RichText::new(display_text).size(15.0));
                        });
                    });
                    ui.add_space(8.0);
                });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            let available_width = ui.available_width();
            if ui
                .add_sized(
                    egui::vec2(available_width, 30.0),
                    egui::Button::new("🔄 Reset Assignment"),
                )
                .clicked()
            {
                self.reset_modal_open = true;
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    egui::widgets::global_theme_preference_buttons(ui);
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);
                    egui::gui_zoom::zoom_menu_buttons(ui);
                },
            );
        });
    }

    fn render_chat_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Calculate user message background color once
            let user_msg_bg = if ui.visuals().dark_mode {
                egui::Color32::from_rgb(75, 85, 99) // Dark gray for dark mode
            } else {
                egui::Color32::from_rgb(254, 243, 199) // Yellow for light mode
            };

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (idx, message) in self.chat_history.iter().enumerate() {
                    if message.from_user {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                            egui::Frame::none()
                                .fill(user_msg_bg)
                                .rounding(egui::Rounding::same(10.0))
                                .inner_margin(egui::Margin::symmetric(10.0, 10.0))
                                .show(ui, |ui| {
                                    CommonMarkViewer::new().show(
                                        ui,
                                        &mut self.message_caches[idx],
                                        &message.content,
                                    );
                                });
                        });
                    } else {
                        egui::Frame::none()
                            .fill(ui.visuals().extreme_bg_color)
                            .rounding(egui::Rounding::same(10.0))
                            .inner_margin(egui::Margin::symmetric(10.0, 10.0))
                            .show(ui, |ui| {
                                CommonMarkViewer::new().show(
                                    ui,
                                    &mut self.message_caches[idx],
                                    &message.content,
                                );
                            });
                    }
                    ui.add_space(10.0);
                }
            });

            // Input area
            ui.horizontal(|ui| {
                let available_width = ui.available_width();
                ui.add(
                    egui::TextEdit::multiline(&mut self.current_input)
                        .hint_text("Type your message here...")
                        .desired_rows(5)
                        .desired_width(available_width * 0.87)
                        .frame(true),
                );
                let button = egui::Button::new("Send").min_size(egui::vec2(
                    available_width * 0.12,
                    ui.spacing().interact_size.y * 4.0,
                ));

                if ui.add(button).clicked() && !self.current_input.is_empty() {
                    // Add user's message and its cache
                    self.chat_history.push(ChatMessage {
                        content: self.current_input.clone(),
                        from_user: true,
                    });
                    self.message_caches.push(CommonMarkCache::default());
                    self.current_input.clear();

                    // Simulate a response and add its cache
                    self.chat_history.push(ChatMessage {
                        content: "This is a simulated response from the assistant.\n# hi\n[ ] test"
                            .to_string(),
                        from_user: false,
                    });
                    self.message_caches.push(CommonMarkCache::default());
                }
            });
        });
    }

    fn render_reset_modal(&mut self, ctx: &egui::Context) {
        if self.reset_modal_open {
            egui::Window::new("Reset Assignment")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("⚠ This will reset all progress and cannot be undone!");
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        if ui.button("Yes, Reset Everything").clicked() {
                            self.reset_to_default();
                            self.reset_modal_open = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.reset_modal_open = false;
                        }
                    });
                });
        }
    }
}

impl eframe::App for LearningApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("Week 12 - Recursion and MergeSort")
                        .size(24.0)
                        .heading(),
                );
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            self.render_side_panel(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_chat_panel(ui);
        });

        self.render_reset_modal(ctx);
    }
}
