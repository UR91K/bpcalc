use eframe::egui;
use egui::{Color32, Pos2, Stroke, Vec2};
use palette::{Srgb, Oklab, IntoColor, Mix};

/// Extension trait to create Srgb from hex color codes
trait SrgbExt {
    fn from_hex(hex: u32) -> Self;
}

impl SrgbExt for Srgb {
    fn from_hex(hex: u32) -> Self {
        let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
        let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
        let b = (hex & 0xFF) as f32 / 255.0;
        Srgb::new(r, g, b)
    }
}

struct HarmonicApp {
    string_length: f32,
    weights: [f32; 6], // Weights for harmonics 2-7
    optimal_position: f32,
    heat_map_resolution: usize,
}

impl Default for HarmonicApp {
    fn default() -> Self {
        let string_length = 650.0; // Typical guitar scale length in mm
        let weights = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let optimal_position = find_optimal_pickup_position_v2(string_length, &weights);
        
        Self {
            string_length,
            weights,
            optimal_position,
            heat_map_resolution: 1000,
        }
    }
}

impl eframe::App for HarmonicApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Harmonic Anti-Node Visualizer");
            ui.add_space(10.0);
            
            // Controls
            ui.horizontal(|ui| {
                ui.label("String Length (mm):");
                if ui.add(egui::Slider::new(&mut self.string_length, 500.0..=1000.0)).changed() {
                    self.optimal_position = find_optimal_pickup_position_v2(self.string_length, &self.weights);
                }
            });
            
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
                self.optimal_position = find_optimal_pickup_position_v2(self.string_length, &self.weights);
            }
            
            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);
            
            // Results
            ui.horizontal(|ui| {
                ui.label("Optimal Pickup Position:");
                ui.colored_label(Color32::GREEN, format!("{:.2} mm from bridge", self.optimal_position));
                ui.label(format!("({:.1}% of string length)", (self.optimal_position / self.string_length) * 100.0));
            });
            
            ui.add_space(20.0);
            
            // Visualization
            self.draw_visualization(ui);
        });
    }
}

impl HarmonicApp {
    fn draw_visualization(&self, ui: &mut egui::Ui) {
        let available_width = ui.available_width() - 40.0;
        let viz_height = 500.0;
        
        let (response, painter) = ui.allocate_painter(
            Vec2::new(available_width, viz_height),
            egui::Sense::hover(),
        );
        
        let rect = response.rect;
        let margin = 20.0;
        let string_start_x = rect.min.x + margin;
        let string_end_x = rect.max.x - margin;
        let string_width = string_end_x - string_start_x;
        
        // Draw background
        painter.rect_filled(rect, 0.0, Color32::from_gray(20));
        
        // Calculate heat map
        let heat_map = self.calculate_heat_map();
        let max_heat = heat_map.iter().cloned().fold(0.0_f32, f32::max);
        
        // Draw heat map
        let heat_map_y = rect.min.y + 50.0;
        let heat_map_height = 60.0;
        
        for (i, &heat) in heat_map.iter().enumerate() {
            let x = string_start_x + (i as f32 / heat_map.len() as f32) * string_width;
            let normalized_heat = if max_heat > 0.0 { heat / max_heat } else { 0.0 };
            let color = heat_to_color(normalized_heat);
            
            painter.line_segment(
                [
                    Pos2::new(x, heat_map_y),
                    Pos2::new(x, heat_map_y + heat_map_height),
                ],
                Stroke::new(1.0, color),
            );
        }
        
        // Draw heat map label
        painter.text(
            Pos2::new(string_start_x, heat_map_y - 10.0),
            egui::Align2::LEFT_BOTTOM,
            "Heat Map (Anti-Node Proximity)",
            egui::FontId::proportional(12.0),
            Color32::WHITE,
        );
        
        // Draw individual harmonics
        let mut current_y = heat_map_y + heat_map_height + 40.0;
        let harmonic_spacing = 45.0;
        
        for harmonic in 2..=7_u8 {
            let weight = self.weights[(harmonic - 2) as usize];
            let anti_nodes = get_anti_nodes_for_harmonic(self.string_length, harmonic);
            
            // Draw string line
            let string_y = current_y;
            painter.line_segment(
                [
                    Pos2::new(string_start_x, string_y),
                    Pos2::new(string_end_x, string_y),
                ],
                Stroke::new(2.0, Color32::GRAY),
            );
            
            // Draw anti-nodes
            for anti_node in anti_nodes {
                let x = string_start_x + (anti_node / self.string_length) * string_width;
                let alpha = (weight * 255.0).min(255.0) as u8;
                let color = Color32::from_rgba_premultiplied(255, 100, 100, alpha);
                
                painter.circle_filled(Pos2::new(x, string_y), 6.0, color);
                painter.circle_stroke(Pos2::new(x, string_y), 6.0, Stroke::new(1.0, Color32::WHITE));
            }
            
            // Draw label
            painter.text(
                Pos2::new(string_start_x - 10.0, string_y),
                egui::Align2::RIGHT_CENTER,
                format!("H{}", harmonic),
                egui::FontId::proportional(12.0),
                Color32::from_gray(200),
            );
            
            // Draw weight indicator
            painter.text(
                Pos2::new(string_end_x + 10.0, string_y),
                egui::Align2::LEFT_CENTER,
                format!("w: {:.2}", weight),
                egui::FontId::proportional(10.0),
                Color32::from_gray(150),
            );
            
            current_y += harmonic_spacing;
        }
        
        // Draw optimal position line across all visualizations
        let optimal_x = string_start_x + (self.optimal_position / self.string_length) * string_width;
        painter.line_segment(
            [
                Pos2::new(optimal_x, heat_map_y),
                Pos2::new(optimal_x, current_y - harmonic_spacing),
            ],
            Stroke::new(2.0, Color32::from_rgb(0, 255, 0)),
        );
        
        // Draw optimal position label
        painter.text(
            Pos2::new(optimal_x, heat_map_y - 25.0),
            egui::Align2::CENTER_BOTTOM,
            "Optimal",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(0, 255, 0),
        );
        
        // Draw bridge and nut labels
        painter.text(
            Pos2::new(string_start_x, rect.max.y - 10.0),
            egui::Align2::CENTER_BOTTOM,
            "Bridge",
            egui::FontId::proportional(12.0),
            Color32::WHITE,
        );
        
        painter.text(
            Pos2::new(string_end_x, rect.max.y - 10.0),
            egui::Align2::CENTER_BOTTOM,
            "Nut",
            egui::FontId::proportional(12.0),
            Color32::WHITE,
        );
    }
    
    fn calculate_heat_map(&self) -> Vec<f32> {
        let mut heat_map = vec![0.0; self.heat_map_resolution];

        for (i, heat) in heat_map.iter_mut().enumerate() {
            let pos = (i as f32 / self.heat_map_resolution as f32) * self.string_length;

            // Calculate sine wave falloff
            let total_heat: f32 = (2..=7_u8)
                .zip(self.weights.iter())
                .map(|(harmonic, &weight)| {
                    let min_dist = get_anti_nodes_for_harmonic(self.string_length, harmonic)
                        .into_iter()
                        .map(|anti_node| (pos - anti_node).abs())
                        .min_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap();

                    // Sine wave falloff: use cosine for smooth bell curve
                    // The wavelength determines how far the influence extends
                    let wavelength = self.string_length / (harmonic as f32 * 2.0);
                    let normalized_dist = (min_dist / wavelength).min(1.0);
                    let falloff = (normalized_dist * std::f32::consts::PI / 2.0).cos();

                    weight * falloff
                })
                .sum();

            *heat = total_heat;
        }

        heat_map
    }
}

fn get_anti_nodes_for_harmonic(length: f32, harmonic: u8) -> Vec<f32> {
    let segment_length = length / harmonic as f32;
    
    (0..harmonic)
        .map(|i| (i as f32 + 0.5) * segment_length)
        .collect()
}

fn find_optimal_pickup_position_v2(length: f32, weights: &[f32; 6]) -> f32 {
    (0..=1000)
        .map(|i| (i as f32 / 1000.0) * length)
        .min_by_key(|&pos| {
            let score: f32 = (2..=7_u8)
                .zip(weights.iter())
                .map(|(harmonic, &weight)| {
                    // Find minimum distance to any anti-node of this harmonic
                    let min_dist = get_anti_nodes_for_harmonic(length, harmonic)
                        .into_iter()
                        .map(|anti_node| (pos - anti_node).abs())
                        .min_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap();
                    min_dist * weight
                })
                .sum();
            
            (score * 10000.0) as i32
        })
        .unwrap()
}

fn heat_to_color(normalized_heat: f32) -> Color32 {
    // Color gradient: Blue (cold) -> Cyan -> Green -> Yellow -> Red (hot)
    let heat = normalized_heat.clamp(0.0, 1.0);
    
    // Viridis color palette (colorblind-friendly)
    // Convert hex colors to Oklab for perceptually uniform interpolation
    let viridis_stops: [(f32, Oklab); 20] = [
        // Purple-violet
        (0.00, Srgb::from_hex(0x0E0154).into_color()),
        (0.05, Srgb::from_hex(0x481567).into_color()),
        (0.10, Srgb::from_hex(0x482677).into_color()),
        (0.15, Srgb::from_hex(0x453781).into_color()),
        (0.20, Srgb::from_hex(0x404788).into_color()),
        // Blue-teal
        (0.25, Srgb::from_hex(0x39568C).into_color()),
        (0.30, Srgb::from_hex(0x33638D).into_color()),
        (0.35, Srgb::from_hex(0x2D708E).into_color()),
        (0.40, Srgb::from_hex(0x287D8E).into_color()),
        (0.45, Srgb::from_hex(0x238A8D).into_color()),
        // Teal-green
        (0.50, Srgb::from_hex(0x1F968B).into_color()),
        (0.55, Srgb::from_hex(0x20A387).into_color()),
        (0.60, Srgb::from_hex(0x29AF7F).into_color()),
        (0.65, Srgb::from_hex(0x3CBB75).into_color()),
        (0.70, Srgb::from_hex(0x55C667).into_color()),
        // Lime-yellow
        (0.75, Srgb::from_hex(0x73D055).into_color()),
        (0.80, Srgb::from_hex(0x95D840).into_color()),
        (0.85, Srgb::from_hex(0xB8DE29).into_color()),
        (0.90, Srgb::from_hex(0xDCE319).into_color()),
        (0.95, Srgb::from_hex(0xFDE725).into_color()),
    ];

    // Find the two color stops to interpolate between using match
    let (lower_idx, upper_idx, t) = match heat {
        h if h >= 0.95 => (19, 19, 0.0),
        h => {
            let idx = (h * 20.0).floor() as usize;
            let lower = idx.min(18);
            let upper = (idx + 1).min(19);
            let t = (h - viridis_stops[lower].0) / (viridis_stops[upper].0 - viridis_stops[lower].0);
            (lower, upper, t)
        }
    };

    // Interpolate in Oklab space
    let oklab = viridis_stops[lower_idx].1.mix(viridis_stops[upper_idx].1, t);
    
    // Convert back to sRGB
    let rgb: Srgb = oklab.into_color();
    
    Color32::from_rgb(
        (rgb.red * 255.0) as u8,
        (rgb.green * 255.0) as u8,
        (rgb.blue * 255.0) as u8,
    )
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Harmonic Anti-Node Visualizer",
        options,
        Box::new(|_cc| Ok(Box::new(HarmonicApp::default()))),
    )
}