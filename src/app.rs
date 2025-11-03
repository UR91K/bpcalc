use egui::Color32;

use crate::calculation::{OptimalPositions, find_optimal_pickup_positions};

pub(crate) struct HarmonicApp {
    pub(crate) string_length: f32,
    pub(crate) weights: [f32; 6], // Weights for harmonics 2-7
    pub(crate) optimal_positions: OptimalPositions,
    pub(crate) heat_map_resolution: usize,
    pub(crate) search_limit: usize,
}

impl Default for HarmonicApp {
    fn default() -> Self {
        let string_length = 650.0; // Typical guitar scale length in mm
        let weights = [0.15, 1.50, 1.50, 1.50, 0.75, 0.75]; // Harmonics 2-7
        let search_limit = (string_length * 0.5) as usize;
        let optimal_positions =
            find_optimal_pickup_positions(string_length, &weights, search_limit);

        Self {
            string_length,
            weights,
            optimal_positions,
            heat_map_resolution: 1000,
            search_limit,
        }
    }
}

impl eframe::App for HarmonicApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Calculate visualizer height first (including separator and spacing)
            let viz_height = self.calculate_visualizer_height() + 30.0; // +30 for separator and spacing

            let available_height = ui.available_height();
            let scroll_area_height = available_height - viz_height;

            // Top section: Scrollable controls
            egui::ScrollArea::vertical()
                .max_height(scroll_area_height)
                .show(ui, |ui| {
                    ui.heading("Harmonic Anti-Node Visualizer");
                    ui.add_space(10.0);

                    // Controls
                    ui.label("String Length (mm):");
                    if ui
                        .add(egui::Slider::new(&mut self.string_length, 500.0..=1000.0))
                        .changed()
                    {
                        self.optimal_positions = find_optimal_pickup_positions(
                            self.string_length,
                            &self.weights,
                            self.search_limit,
                        );
                    }

                    ui.label("Search Limit:");
                    if ui
                        .add(egui::Slider::new(
                            &mut self.search_limit,
                            1..=(self.string_length / 2.0) as usize,
                        ))
                        .changed()
                    {
                        self.optimal_positions = find_optimal_pickup_positions(
                            self.string_length,
                            &self.weights,
                            self.search_limit,
                        );
                    }

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // Weight sliders
                    ui.label("Harmonic Weights:");
                    let mut weights_changed = false;
                    for (i, weight) in self.weights.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(format!("Harmonic {}:", i + 2));
                            if ui.add(egui::Slider::new(weight, 0.0..=2.0)).changed() {
                                weights_changed = true;
                            }
                        });
                    }

                    if weights_changed {
                        self.optimal_positions = find_optimal_pickup_positions(
                            self.string_length,
                            &self.weights,
                            self.search_limit,
                        );
                    }

                    ui.add_space(20.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // Results
                    ui.horizontal(|ui| {
                        ui.label("Bridge Pickup:");
                        ui.colored_label(
                            Color32::LIGHT_BLUE,
                            format!(
                                "{:.2} mm from bridge",
                                self.optimal_positions.bridge_position
                            ),
                        );
                        ui.label(format!(
                            "({:.1}%)",
                            (self.optimal_positions.bridge_position / self.string_length) * 100.0
                        ));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Neck Pickup:");
                        ui.colored_label(
                            Color32::LIGHT_BLUE,
                            format!("{:.2} mm from bridge", self.optimal_positions.neck_position),
                        );
                        ui.label(format!(
                            "({:.1}%)",
                            (self.optimal_positions.neck_position / self.string_length) * 100.0
                        ));
                    });
                });

            // Bottom section: Visualization anchored to bottom
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                self.draw_visualization(ui);
                ui.separator();
                ui.add_space(10.0);
            });
        });
    }
}
