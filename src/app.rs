#![warn(clippy::all)]

use eframe::egui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use serde::{Deserialize, Serialize};

use crate::{make_anthropic_request, PENDING_STATE};

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
pub(crate) struct CheckpointMatch {
    checkpoint_id: String,
    description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ChatMessage {
    pub(crate) content: String,
    pub(crate) from_user: bool,
    pub(crate) cacheable: bool,
    #[serde(skip)]
    pub(crate) analyzed_for_checkpoints: bool,
    #[serde(default)]
    pub(crate) found_checkpoints: Vec<CheckpointMatch>,
}

#[derive(Debug)]
struct ScrollState {
    viewport_height: f32,
    total_height: f32,
    message_heights: Vec<f32>,
    scroll_offset: f32,
    estimated_message_height: f32,
}

impl ScrollState {
    fn new() -> Self {
        Self {
            viewport_height: 0.0,
            total_height: 0.0,
            message_heights: Vec::new(),
            scroll_offset: 0.0,
            estimated_message_height: 100.0, // Initial estimate
        }
    }

    fn update_message_height(&mut self, idx: usize, height: f32) {
        if idx >= self.message_heights.len() {
            self.message_heights
                .resize(idx + 1, self.estimated_message_height);
        }
        self.message_heights[idx] = height;
        self.recalculate_total_height();
    }

    fn recalculate_total_height(&mut self) {
        self.total_height = self.message_heights.iter().sum();
    }

    fn get_visible_range(&self) -> (usize, usize) {
        let mut current_height = 0.0;
        let mut start_idx = 0;
        let mut end_idx = self.message_heights.len();

        // Find start index
        for (idx, height) in self.message_heights.iter().enumerate() {
            if current_height + height > self.scroll_offset {
                start_idx = idx;
                break;
            }
            current_height += height;
        }

        // Find end index
        current_height = 0.0;
        for (idx, height) in self.message_heights.iter().enumerate() {
            current_height += height;
            if current_height > self.scroll_offset + self.viewport_height {
                end_idx = idx + 1;
                break;
            }
        }

        // Add buffer
        start_idx = start_idx.saturating_sub(2);
        end_idx = (end_idx + 2).min(self.message_heights.len());

        (start_idx, end_idx)
    }
}

// At the top of app.rs, modify the LearningApp struct declaration:

#[derive(serde::Deserialize, serde::Serialize)] // Add Clone here
#[serde(default)]
pub struct LearningApp {
    label: String,
    checkpoints: Vec<Checkpoint>,
    reset_modal_open: bool,
    chat_history: Vec<ChatMessage>,
    #[serde(skip)]
    message_caches: Vec<CommonMarkCache>, // CommonMarkCache must also be Clone
    current_input: String,
    #[serde(skip)]
    error_modal: Option<String>,
    #[serde(skip)]
    pending_message: Option<String>,
    #[serde(skip)]
    is_loading: bool,
    #[serde(skip)]
    scroll_state: ScrollState,
}

impl Default for LearningApp {
    fn default() -> Self {
        let initial_message = ChatMessage {
            content: concat!(
                "Welcome! I'm excited to help you discover MergeSort through an interactive learning experience. ",
                "Let's start with a simple problem to get us thinking about sorting.\n\n",
                "Imagine you have this sequence of numbers: `[7, 2, 4, 1, 5, 3]`\n\n",
                "If you had to sort these numbers by hand, what would be your natural approach? ",
                "How would you go about it?\n\n",
                "Remember, there's no wrong answer here - I want to understand how you think about sorting intuitively."
            ).to_string(),
            from_user: false,
            cacheable: true,
            analyzed_for_checkpoints: false,
            found_checkpoints: Vec::new(),
        };

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
            chat_history: vec![initial_message],
            message_caches: vec![CommonMarkCache::default()],
            current_input: String::new(),
            error_modal: None,
            pending_message: None,
            is_loading: false,
            scroll_state: ScrollState::new(),
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
                egui::RichText::new("Checkpoint Progress")
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
        let available_height = ui.available_height();
        let (user_msg_bg, user_msg_stroke) = if ui.visuals().dark_mode {
            (egui::Color32::from_rgb(75, 85, 99), egui::Color32::WHITE)
        } else {
            (egui::Color32::from_rgb(254, 243, 199), egui::Color32::BLACK)
        };

        self.scroll_state.viewport_height = available_height;

        // Ensure we have height estimates for all messages
        if self.scroll_state.message_heights.len() < self.chat_history.len() {
            self.scroll_state.message_heights.resize(
                self.chat_history.len(),
                self.scroll_state.estimated_message_height,
            );
            self.scroll_state.recalculate_total_height();
        }

        ui.vertical(|ui| {
            let scroll_area = egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .max_height(available_height - 100.0); // Leave space for input area

            scroll_area.show(ui, |ui| {
                // Get visible range
                let (start_idx, end_idx) = self.scroll_state.get_visible_range();

                // Add spacing for messages above viewport
                let space_above: f32 = self.scroll_state.message_heights[..start_idx].iter().sum();
                ui.add_space(space_above);
                let old_override_text_color = ui.style().visuals.override_text_color;
                ui.style_mut().visuals.override_text_color = Some(user_msg_stroke);
                // Render visible messages
                for idx in start_idx..end_idx {
                    let message = &self.chat_history[idx];
                    let response = if message.from_user {
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
                                })
                                .inner
                        })
                    } else {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
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
                                })
                                .inner
                        })
                    };

                    // Update message height
                    let rect = response.response.rect;
                    self.scroll_state
                        .update_message_height(idx, rect.height() + 10.0); // Add spacing

                    ui.add_space(10.0);
                }
                ui.style_mut().visuals.override_text_color = old_override_text_color;

                // Add spacing for messages below viewport
                let space_below: f32 = self.scroll_state.message_heights[end_idx..].iter().sum();
                ui.add_space(space_below);
            });

            // Input area with frame
            egui::Frame::none()
                .fill(ui.visuals().faint_bg_color)
                .inner_margin(egui::Margin::same(8.0))
                .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let available_width = ui.available_width();
                        ui.add(
                            egui::TextEdit::multiline(&mut self.current_input)
                                .hint_text("Type your message here...")
                                .desired_rows(5)
                                .desired_width(available_width * 0.87)
                                .frame(true),
                        );

                        let button_width = available_width * 0.12;
                        let button_height = ui.spacing().interact_size.y * 4.0;

                        if self.is_loading {
                            ui.add_sized(
                                egui::vec2(button_width, button_height),
                                egui::Spinner::new().size(16.0),
                            );
                        } else {
                            let button = egui::Button::new("Send")
                                .min_size(egui::vec2(button_width, button_height));

                            if ui.add(button).clicked()
                                && !self.current_input.is_empty()
                                && !self.is_loading
                            {
                                let message = self.current_input.clone();
                                self.current_input.clear();
                                self.send_message(message);
                            }
                        }
                    });
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

    fn handle_api_error(&mut self, error: String) {
        self.error_modal = Some(error);
        if let Some(last_message) = self.chat_history.last() {
            if last_message.from_user {
                self.pending_message = Some(last_message.content.clone());
            }
        }
    }

    fn retry_last_message(&mut self) {
        if let Some(message) = self.pending_message.take() {
            self.send_message(message);
        }
        self.error_modal = None;
    }

    fn render_error_modal(&mut self, ctx: &egui::Context) {
        // Clone the values we need for the closure
        let error_message = self.error_modal.clone();
        let has_pending = self.pending_message.is_some();

        if let Some(error) = error_message {
            egui::Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("An error occurred:");
                    ui.label(error);
                    ui.add_space(8.0);
                    if has_pending && ui.button("Retry").clicked() {
                        self.retry_last_message();
                    }
                    if ui.button("Close").clicked() {
                        self.error_modal = None;
                        self.pending_message = None;
                    }
                });
        }
    }

    fn send_message(&mut self, message: String) {
        self.chat_history.push(ChatMessage {
            content: message.clone(),
            from_user: true,
            cacheable: false,
            analyzed_for_checkpoints: false,
            found_checkpoints: Vec::new(),
        });
        self.message_caches.push(CommonMarkCache::default());
        self.is_loading = true;

        // Pass the chat history before adding the new message
        let history = self.chat_history[..self.chat_history.len() - 1].to_vec();

        make_anthropic_request(message, history, |result| {
            let mut state = PENDING_STATE.lock().unwrap();
            match result {
                Ok(response) => state.response = Some(response),
                Err(error) => state.error = Some(error),
            }
        });
    }

    fn scan_message_for_checkpoints(&mut self, message_idx: usize) {
        let message = &self.chat_history[message_idx];

        if message.analyzed_for_checkpoints || message.from_user {
            return;
        }

        let checkpoint_ids = [
            "inefficiency_discovery",
            "splitting_insight",
            "merging_development",
            "recursive_pattern",
            "efficiency_analysis",
        ];

        let mut found_checkpoints = Vec::new();

        for line in message.content.lines() {
            if let Some(start) = line.find("CHECKPOINT[") {
                if let Some(end) = line.find("]:") {
                    let checkpoint_id = &line[(start + "CHECKPOINT[".len())..end].trim();

                    if checkpoint_ids.contains(checkpoint_id) {
                        let description = line[end + 2..].trim().to_string();

                        if let Some(checkpoint) = self
                            .checkpoints
                            .iter_mut()
                            .find(|c| c.id.as_str() == *checkpoint_id)
                        {
                            checkpoint.status = CheckpointStatus::Completed;

                            found_checkpoints.push(CheckpointMatch {
                                checkpoint_id: checkpoint_id.to_string(),
                                description,
                            });
                        }
                    }
                }
            }
        }

        let message = &mut self.chat_history[message_idx];
        message.found_checkpoints = found_checkpoints;
        message.analyzed_for_checkpoints = true;
    }
}

impl eframe::App for LearningApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for pending messages
        let mut state = PENDING_STATE.lock().unwrap();
        if let Some(response) = state.response.take() {
            self.chat_history.push(ChatMessage {
                content: response,
                from_user: false,
                cacheable: false,
                analyzed_for_checkpoints: false,
                found_checkpoints: Vec::new(),
            });
            self.message_caches.push(CommonMarkCache::default());

            let last_idx = self.chat_history.len() - 1;
            self.scan_message_for_checkpoints(last_idx);

            self.is_loading = false;
            ctx.request_repaint();
        }
        if let Some(error) = state.error.take() {
            self.handle_api_error(error);
            self.is_loading = false; // Reset loading state on error
        }

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
        self.render_error_modal(ctx);
    }
}
