/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct LearningApp {
    label: String,
    current_question: usize,
    total_questions: usize,
    assignment_title: String,
    assignment_started: bool,
    current_question_answered: bool,

    #[serde(skip)]
    value: f32,
}

impl Default for LearningApp {
    fn default() -> Self {
        Self {
            label: "Hello World!".to_owned(),
            value: 2.7,
            current_question: 0,
            total_questions: 0,
            assignment_title: "".to_owned(),
            assignment_started: false,
            current_question_answered: false,
        }
    }
}

impl LearningApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    fn render_top_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Center section - Controls
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                // Start Assignment button
                if ui
                    .button(if !self.assignment_started {
                        "▶ Start"
                    } else {
                        "Started"
                    })
                    .clicked()
                {
                    self.assignment_started = true;
                }

                // Reset Session button
                if ui.button("↺ Reset Session").clicked() {
                    self.assignment_started = false;
                    self.current_question = 1;
                    self.current_question_answered = false;
                }

                ui.separator();

                // Question navigation
                ui.horizontal(|ui| {
                    if ui.button("⏴").clicked() && self.current_question > 1 {
                        self.current_question -= 1;
                    }
                    ui.label(format!(
                        "Question {} of {}",
                        self.current_question, self.total_questions
                    ));
                    if ui.button("⏵").clicked() && self.current_question < self.total_questions {
                        self.current_question += 1;
                    }
                });

                ui.separator();

                // Mark as Answered button
                let answer_button = egui::Button::new(if self.current_question_answered {
                    "☑ Answered?"
                } else {
                    "☐ Answered?"
                });
                if ui.add(answer_button).clicked() {
                    self.current_question_answered = !self.current_question_answered;
                }

                // Submit Assignment button
                if ui.button("🏆 Submit").clicked() {
                    // Handle submission
                }

                ui.separator();
                egui::widgets::global_theme_preference_buttons(ui);
                ui.separator();
                egui::gui_zoom::zoom_menu_buttons(ui);
            });
        });
    }
}

impl eframe::App for LearningApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::Area::new("splash".into())
            .order(egui::Order::Background)
            .default_width(f32::INFINITY)
            .fixed_pos(egui::Pos2 { x: 32.0, y: 0.0 })
            .show(ctx, |ui| {
                egui::ScrollArea::horizontal()
                    .max_width(f32::INFINITY)
                    .show(ui, |ui| {
                        ui.set_opacity(0.42);
                        ui.horizontal_centered(|ui| {
                            ui.label(egui::RichText::new("ITSC 2214 -").size(42.0).heading());
                            ui.label(egui::RichText::new("MergeSort").size(60.0).heading());
                            ui.add_space(32.0);
                            ui.set_opacity(1.0);

                            self.render_top_panel(ui);
                        });
                    });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("eframe template");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(&mut self.label);
            });

            ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                self.value += 1.0;
            }

            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/main/",
                "Source code."
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
