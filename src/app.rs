use chrono::{DateTime, Local};
use egui::{Color32, RichText, ScrollArea, Vec2};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
struct Message {
    content: String,
    is_user: bool,
    timestamp: DateTime<Local>,
}

#[derive(Deserialize, Serialize)]
struct Checkpoint {
    name: String,
    description: String,
    start_time: Option<DateTime<Local>>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct LearningApp {
    messages: Vec<Message>,
    current_input: String,
    checkpoints: Vec<Checkpoint>,
    current_checkpoint: usize,
    #[serde(skip)]
    scroll_to_bottom: bool,
}

impl Default for LearningApp {
    fn default() -> Self {
        let checkpoints = vec![
            Checkpoint {
                name: "Understanding Current Limitations".to_string(),
                description: "Explore why simple sorting is inefficient".to_string(),
                start_time: Some(Local::now()),
            },
            Checkpoint {
                name: "Binary Search Connection".to_string(),
                description: "Connect to previous divide-and-conquer success".to_string(),
                start_time: None,
            },
            Checkpoint {
                name: "Split Attempt".to_string(),
                description: "First try at splitting the problem".to_string(),
                start_time: None,
            },
            Checkpoint {
                name: "Merge Discovery".to_string(),
                description: "Discover how to combine sorted sequences".to_string(),
                start_time: None,
            },
            Checkpoint {
                name: "Recursion Insight".to_string(),
                description: "Complete algorithm realization".to_string(),
                start_time: None,
            },
            Checkpoint {
                name: "Complexity Analysis".to_string(),
                description: "Understanding efficiency gains".to_string(),
                start_time: None,
            },
        ];

        let initial_message = Message {
            content: "Let's explore sorting algorithms together! How would you approach sorting a list of numbers?".to_string(),
            is_user: false,
            timestamp: Local::now(),
        };

        Self {
            messages: vec![initial_message],
            current_input: String::new(),
            checkpoints,
            current_checkpoint: 0,
            scroll_to_bottom: false,
        }
    }
}

impl LearningApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        Default::default()
    }

    fn send_message(&mut self) {
        if !self.current_input.trim().is_empty() {
            let user_message = Message {
                content: self.current_input.clone(),
                is_user: true,
                timestamp: Local::now(),
            };
            self.messages.push(user_message);
            self.current_input.clear();
            self.scroll_to_bottom = true;

            // Simulate assistant response (in a real app, this would come from the LLM)
            let assistant_message = Message {
                content: "I see your approach. Let's think about that...".to_string(),
                is_user: false,
                timestamp: Local::now(),
            };
            self.messages.push(assistant_message);
        }
    }
}

impl eframe::App for LearningApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Left panel for checkpoints
        egui::SidePanel::left("checkpoints_panel")
            .resizable(true)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Learning Progress");
                ui.add_space(8.0);

                ScrollArea::vertical().show(ui, |ui| {
                    for (index, checkpoint) in self.checkpoints.iter().enumerate() {
                        let is_current = index == self.current_checkpoint;
                        let text = RichText::new(&checkpoint.name).color(if is_current {
                            Color32::BLUE
                        } else {
                            ui.style().visuals.text_color()
                        });

                        ui.add_space(4.0);
                        if ui
                            .add(egui::Label::new(text).sense(egui::Sense::click()))
                            .clicked()
                        {
                            // Handle checkpoint click
                        }

                        if is_current {
                            ui.label(RichText::new(&checkpoint.description).small());
                            if let Some(start_time) = checkpoint.start_time {
                                let duration = Local::now() - start_time;
                                ui.label(
                                    RichText::new(format!(
                                        "Time: {}:{:02}",
                                        duration.num_minutes(),
                                        duration.num_seconds() % 60
                                    ))
                                    .small(),
                                );
                            }
                        }
                        ui.add_space(4.0);
                        ui.separator();
                    }
                });
            });

        // Main chat panel
        egui::CentralPanel::default().show(ctx, |ui| {
            // Messages area
            let chat_height = ui.available_height() - 50.0; // Reserve space for input
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(self.scroll_to_bottom)
                .show(ui, |ui| {
                    self.scroll_to_bottom = false;
                    for message in &self.messages {
                        ui.add_space(8.0);
                        let bubble_color = if message.is_user {
                            Color32::from_rgb(56, 139, 255)
                        } else {
                            ui.style().visuals.window_fill()
                        };

                        let text_color = if message.is_user {
                            Color32::WHITE
                        } else {
                            ui.style().visuals.text_color()
                        };

                        let layout = if message.is_user {
                            egui::Layout::right_to_left(egui::Align::Min)
                        } else {
                            egui::Layout::left_to_right(egui::Align::Min)
                        };

                        ui.with_layout(layout, |ui| {
                            ui.add(
                                egui::Label::new(RichText::new(&message.content).color(text_color))
                                    .wrap(),
                            );
                        });

                        ui.add_space(4.0);
                    }
                });

            // Input area
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let text_edit = ui.add_sized(
                    ui.available_size() - Vec2::new(60.0, 0.0),
                    egui::TextEdit::singleline(&mut self.current_input)
                        .hint_text("Type your message..."),
                );

                if ui
                    .add_sized([50.0, 20.0], egui::Button::new("Send"))
                    .clicked()
                    || (text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                {
                    self.send_message();
                }
            });
        });
    }
}
