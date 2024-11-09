#![warn(clippy::all)]

use eframe::egui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use email_address::*;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{self, Receiver, Sender};

use crate::{initialize_auth_state, save_auth_state};
#[cfg(target_arch = "wasm32")]
use crate::{make_anthropic_request, request_otp_web, verify_otp_web, AUTH_STATE, PENDING_STATE};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

#[cfg(not(target_arch = "wasm32"))]
use crate::{
    make_anthropic_request, request_otp_native, verify_otp_native, AUTH_STATE, PENDING_STATE,
};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
enum MilestoneStatus {
    NotStarted,
    InProgress,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Milestone {
    id: String,
    description: String,
    status: MilestoneStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MilestoneMatch {
    milestone_id: String,
    description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ChatMessage {
    pub(crate) content: String,
    pub(crate) from_user: bool,
    pub(crate) cacheable: bool,
    #[serde(skip)]
    pub(crate) analyzed_for_milestones: bool,
    #[serde(default)]
    pub(crate) found_milestones: Vec<MilestoneMatch>,
}

#[derive(Debug)]
struct ScrollState {
    stick_to_bottom: bool,
}

impl ScrollState {
    fn new() -> Self {
        Self {
            stick_to_bottom: true,
        }
    }
}

// At the top of app.rs, modify the LearningApp struct declaration:

#[derive(serde::Deserialize, serde::Serialize)] // Add Clone here
#[serde(default)]
pub struct LearningApp {
    label: String,
    milestones: Vec<Milestone>,
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
    #[serde(skip)]
    auth_modal_open: bool,
    #[serde(skip)]
    auth_email: String,
    #[serde(skip)]
    auth_code: String,
    #[serde(skip)]
    auth_step: AuthStep,
    #[serde(skip)]
    auth_error: Option<String>,
    #[serde(skip)]
    auth_tx: Sender<AuthMessage>,
    #[serde(skip)]
    auth_rx: Receiver<AuthMessage>,
}

#[derive(Debug, PartialEq)]
enum AuthStep {
    EnterEmail,
    EnterCode,
    HaveCode,
}

#[allow(dead_code)]
enum AuthMessage {
    OTPRequested(Result<(), String>),
    OTPVerified(Result<(), String>),
}

impl Default for LearningApp {
    fn default() -> Self {
        let (auth_tx, auth_rx) = mpsc::channel();

        let initial_messages = vec![
            ChatMessage {
                content: "I am ready, please begin.".to_string(),
                from_user: true,
                cacheable: false,
                analyzed_for_milestones: false,
                found_milestones: Vec::new(),
            },
            ChatMessage {
                content: [
                    "Hello! I'm excited to explore sorting algorithms with you today. Before we dive in, let's start with something simple.\n\n",
                    "Imagine you have these numbers: [7, 2, 4, 1]\n\n",
                    "Could you try sorting these numbers from smallest to largest? Just tell me your steps as you do it.\n\n",
                    "(Note: I want to see your natural approach without any specific instructions from me. Just do what feels intuitive!)"
                ].concat(),
                from_user: false,
                cacheable: true,
                analyzed_for_milestones: false,
                found_milestones: Vec::new(),
            },
        ];

        Self {
            label: "Hello World!".to_owned(),
            milestones: vec![
                Milestone {
                    id: "inefficiency_discovery".to_string(),
                    description: "Understanding sorting inefficiency".to_string(),
                    status: MilestoneStatus::InProgress,
                },
                Milestone {
                    id: "splitting_insight".to_string(),
                    description: "Discovering divide-and-conquer benefit".to_string(),
                    status: MilestoneStatus::NotStarted,
                },
                Milestone {
                    id: "merging_development".to_string(),
                    description: "Understanding systematic merging".to_string(),
                    status: MilestoneStatus::NotStarted,
                },
                Milestone {
                    id: "recursive_pattern".to_string(),
                    description: "Grasping recursive nature".to_string(),
                    status: MilestoneStatus::NotStarted,
                },
                Milestone {
                    id: "efficiency_analysis".to_string(),
                    description: "Comprehending O(n log n) complexity".to_string(),
                    status: MilestoneStatus::NotStarted,
                },
            ],
            reset_modal_open: false,
            chat_history: initial_messages,
            message_caches: vec![CommonMarkCache::default(), CommonMarkCache::default()],
            current_input: String::new(),
            error_modal: None,
            pending_message: None,
            is_loading: false,
            scroll_state: ScrollState::new(),
            auth_modal_open: false,
            auth_email: String::new(),
            auth_code: String::new(),
            auth_step: AuthStep::EnterEmail,
            auth_error: None,
            auth_tx,
            auth_rx,
        }
    }
}

impl LearningApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            // Initialize auth state from storage
            initialize_auth_state(storage);

            let mut app: Self = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            // Initialize caches for any existing messages
            app.message_caches = Vec::with_capacity(app.chat_history.len());
            for _ in 0..app.chat_history.len() {
                app.message_caches.push(CommonMarkCache::default());
            }
            app
        } else {
            Default::default()
        }
    }

    fn reset_to_default(&mut self) {
        // Save the current auth state
        let auth_state = AUTH_STATE.lock().unwrap();
        let saved_email = auth_state.email.clone();
        let saved_token = auth_state.access_token.clone();
        let saved_signed_in = auth_state.signed_in;
        drop(auth_state);

        // Reset the app
        *self = Default::default();

        // Restore auth state
        let mut auth_state = AUTH_STATE.lock().unwrap();
        auth_state.email = saved_email;
        auth_state.access_token = saved_token;
        auth_state.signed_in = saved_signed_in;
    }

    fn render_side_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Add login status at the top
            let auth_state = AUTH_STATE.lock().unwrap();
            if auth_state.signed_in {
                if let Some(email) = &auth_state.email {
                    ui.horizontal(|ui| {
                        ui.label("Logged in as:");
                        ui.label(egui::RichText::new(email).strong());
                    });
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);
                }
            }
            drop(auth_state);

            ui.label(
                egui::RichText::new("Milestone Progress")
                    .size(18.0)
                    .heading(),
            );
            ui.add_space(8.0);
            // Render milestones
            self.milestones
                .iter()
                .enumerate()
                .for_each(|(index, milestone)| {
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            // Show number + description or ???
                            let display_text = match milestone.status {
                                MilestoneStatus::Completed => {
                                    format!("{}. {} ✔", index + 1, milestone.description)
                                }
                                MilestoneStatus::InProgress => {
                                    format!("{}.   ? ? ?  (In Progress... ⏳)", index + 1)
                                }
                                MilestoneStatus::NotStarted => {
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

        ui.vertical(|ui| {
            let scroll_area = egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(self.scroll_state.stick_to_bottom)
                .max_height(available_height - 100.0); // Account for input area

            scroll_area.show(ui, |ui| {
                let old_override_text_color = ui.style().visuals.override_text_color;
                ui.style_mut().visuals.override_text_color = Some(user_msg_stroke);

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
                                    )
                                });
                        });
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
                                    )
                                });
                        });
                    }
                    ui.add_space(10.0);
                }

                ui.style_mut().visuals.override_text_color = old_override_text_color;
            });

            // Input area with frame
            egui::Frame::none()
                .fill(ui.visuals().faint_bg_color)
                .inner_margin(egui::Margin::same(8.0))
                .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let available_width = ui.available_width();

                        // Create a scrollable input area
                        egui::ScrollArea::vertical()
                            .max_height(150.0) // Limit maximum height
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.current_input)
                                        .hint_text("Type your message here...")
                                        .desired_width(available_width * 0.87)
                                        .frame(true)
                                        .desired_rows(4), // Start with 3 rows
                                );
                            });

                        let button_width = available_width * 0.12;
                        let button_height = ui.spacing().interact_size.y * 4.0;

                        let auth_state = AUTH_STATE.lock().unwrap();
                        let is_signed_in = auth_state.signed_in;
                        drop(auth_state);

                        if self.is_loading {
                            ui.add_sized(
                                egui::vec2(button_width, button_height),
                                egui::Spinner::new(),
                            );
                        } else {
                            let button = egui::Button::new("Send")
                                .min_size(egui::vec2(button_width, button_height));

                            if !is_signed_in {
                                if ui.add(button).clicked() {
                                    self.auth_modal_open = true;
                                }
                            } else if ui.add(button).clicked()
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
            analyzed_for_milestones: false,
            found_milestones: Vec::new(),
        });
        self.message_caches.push(CommonMarkCache::default());

        // Set stick_to_bottom before and after adding the message
        self.scroll_state.stick_to_bottom = true;
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

    fn scan_message_for_milestones(&mut self, message_idx: usize) {
        let message = &self.chat_history[message_idx];

        if message.analyzed_for_milestones || message.from_user {
            return;
        }

        let milestone_ids = [
            "inefficiency_discovery",
            "splitting_insight",
            "merging_development",
            "recursive_pattern",
            "efficiency_analysis",
        ];

        let mut found_milestones = Vec::new();

        for line in message.content.lines() {
            if let Some(start) = line.find("MILESTONE[") {
                if let Some(end) = line[start..].find("]") {
                    let milestone_id = &line[(start + "MILESTONE[".len())..start + end];
                    let milestone_id = milestone_id.trim();

                    if milestone_ids.contains(&milestone_id) {
                        if let Some(milestone) = self
                            .milestones
                            .iter_mut()
                            .find(|c| c.id.as_str() == milestone_id)
                        {
                            milestone.status = MilestoneStatus::Completed;
                            found_milestones.push(MilestoneMatch {
                                milestone_id: milestone_id.to_string(),
                                description: milestone.description.clone(),
                            });
                        }
                    }
                }
            }
        }

        let message = &mut self.chat_history[message_idx];
        message.found_milestones = found_milestones;
        message.analyzed_for_milestones = true;
    }

    fn render_auth_modal(&mut self, ctx: &egui::Context) {
        if self.auth_modal_open {
            egui::Window::new("Sign In")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    if let Some(error) = &self.auth_error {
                        ui.label(
                            egui::RichText::new("Error:")
                                .color(egui::Color32::RED)
                                .strong(),
                        );
                        egui::ScrollArea::vertical()
                            .max_height(100.0)
                            .show(ui, |ui| {
                                ui.label(error);
                            });
                        ui.add_space(8.0);
                    }

                    match self.auth_step {
                        AuthStep::EnterEmail => {
                            ui.label("Enter your email to receive a sign-in code:");
                            ui.text_edit_singleline(&mut self.auth_email);
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new("(Sign-in codes are valid for 1 hour)")
                                    .weak()
                                    .italics(),
                            );
                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                if ui.button("Get Code").clicked() {
                                    self.request_otp();
                                }
                                if ui.button("I have a code").clicked() {
                                    self.auth_step = AuthStep::HaveCode;
                                }
                            });
                        }
                        AuthStep::EnterCode => {
                            ui.label(format!("Enter the code sent to {}:", self.auth_email));
                            ui.text_edit_singleline(&mut self.auth_code);
                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                if ui.button("Verify").clicked() {
                                    self.verify_otp();
                                }
                                if ui.button("Back").clicked() {
                                    self.auth_step = AuthStep::EnterEmail;
                                    self.auth_code.clear();
                                }
                            });
                        }
                        AuthStep::HaveCode => {
                            ui.label("Enter your email:");
                            ui.text_edit_singleline(&mut self.auth_email);
                            ui.add_space(8.0);
                            ui.label("Enter your code:");
                            ui.text_edit_singleline(&mut self.auth_code);
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new("(Sign-in codes are valid for 1 hour)")
                                    .weak()
                                    .italics(),
                            );
                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                if ui.button("Verify").clicked() {
                                    self.verify_otp();
                                }
                                if ui.button("Back").clicked() {
                                    self.auth_step = AuthStep::EnterEmail;
                                    self.auth_code.clear();
                                }
                            });
                        }
                    }
                });
        }
    }

    fn request_otp(&mut self) {
        if self.auth_email.is_empty() {
            self.auth_error = Some("Please enter an email address".to_string());
            return;
        }

        if !EmailAddress::is_valid(&self.auth_email) {
            self.auth_error = Some("Please enter a valid email address".to_string());
            return;
        }

        let email = self.auth_email.clone();
        let tx = self.auth_tx.clone();

        #[cfg(target_arch = "wasm32")]
        {
            spawn_local(async move {
                let result = request_otp_web(&email).await;
                let _ = tx.send(AuthMessage::OTPRequested(result));
            });
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            tokio::spawn(async move {
                let result = request_otp_native(&email).await;
                let _ = tx.send(AuthMessage::OTPRequested(result));
            });
        }
    }

    fn verify_otp(&mut self) {
        if self.auth_email.is_empty() {
            self.auth_error = Some("Please enter an email address".to_string());
            return;
        }

        if !EmailAddress::is_valid(&self.auth_email) {
            self.auth_error = Some("Please enter a valid email address".to_string());
            return;
        }

        if self.auth_code.is_empty() {
            self.auth_error = Some("Please enter the verification code".to_string());
            return;
        }

        let email = self.auth_email.clone();
        let token = self.auth_code.clone();
        let tx = self.auth_tx.clone();

        #[cfg(target_arch = "wasm32")]
        {
            spawn_local(async move {
                let result = verify_otp_web(&email, &token).await;
                let _ = tx.send(AuthMessage::OTPVerified(result));
            });
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            tokio::spawn(async move {
                let result = verify_otp_native(&email, &token).await;
                let _ = tx.send(AuthMessage::OTPVerified(result));
            });
        }
    }
}

impl eframe::App for LearningApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // Save both app state and auth state
        eframe::set_value(storage, eframe::APP_KEY, self);
        save_auth_state(storage);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for auth messages on both platforms
        while let Ok(msg) = self.auth_rx.try_recv() {
            match msg {
                AuthMessage::OTPRequested(result) => {
                    match result {
                        Ok(_) => {
                            self.auth_step = AuthStep::EnterCode;
                            self.auth_error = None;
                        }
                        Err(e) => {
                            self.auth_error = Some(match e.as_str() {
                                s if s.contains("over_email_send_rate_limit") => 
                                    "Too many attempts. Please wait a few minutes before trying again.".to_string(),
                                _ => format!("Failed to send code: {}", e)
                            });
                        }
                    }
                }
                AuthMessage::OTPVerified(result) => match result {
                    Ok(_) => {
                        self.auth_modal_open = false;
                        self.auth_error = None;
                    }
                    Err(e) => {
                        // Log the full error for debugging
                        log::error!("OTP verification error: {}", e);
                        self.auth_error = Some(match e.as_str() {
                            s if s.contains("otp_expired") => {
                                "Code has expired or is invalid. Please try again.".to_string()
                            }
                            s if s.contains("invalid_token") => {
                                "Invalid code. Please check and try again.".to_string()
                            }
                            s if s.contains("rate_limit") => {
                                "Too many attempts. Please wait a few minutes before trying again."
                                    .to_string()
                            }
                            _ => format!("Error: {}", e),
                        });
                    }
                },
            }
        }

        // Check for pending messages
        let mut state = PENDING_STATE.lock().unwrap();
        if let Some(response) = state.response.take() {
            self.scroll_state.stick_to_bottom = true; // Set before adding AI response
            self.chat_history.push(ChatMessage {
                content: response,
                from_user: false,
                cacheable: false,
                analyzed_for_milestones: false,
                found_milestones: Vec::new(),
            });
            self.message_caches.push(CommonMarkCache::default());

            let last_idx = self.chat_history.len() - 1;
            self.scan_message_for_milestones(last_idx);

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
        self.render_auth_modal(ctx);
    }
}
