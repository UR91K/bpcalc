use eframe::egui;
use egui::{Color32, Pos2, Stroke, Vec2};
use palette::{Srgb, Oklab, IntoColor, Mix};

/// Extension trait to create Srgb from hex color codes
trait ColorExt {
    fn parse_hex(hex: u32) -> Self;
}

impl ColorExt for Srgb {
    fn parse_hex(hex: u32) -> Self {
        let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
        let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
        let b = (hex & 0xFF) as f32 / 255.0;
        Srgb::new(r, g, b)
    }
}

impl ColorExt for Color32 {
    fn parse_hex(hex: u32) -> Self {
        let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
        let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
        let b = (hex & 0xFF) as f32 / 255.0;
        Color32::from_rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
    }
}

struct HarmonicApp {
    string_length: f32,
    weights: [f32; 6], // Weights for harmonics 2-7
    optimal_positions: OptimalPositions,
    heat_map_resolution: usize,
}

impl Default for HarmonicApp {
    fn default() -> Self {
        let string_length = 650.0; // Typical guitar scale length in mm
        let weights = [0.15, 1.50, 1.50, 1.50, 0.75, 0.75]; // Harmonics 2-7
        let optimal_positions = find_optimal_pickup_positions(string_length, &weights);
        
        Self {
            string_length,
            weights,
            optimal_positions,
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
                    self.optimal_positions = find_optimal_pickup_positions(self.string_length, &self.weights);
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
                self.optimal_positions = find_optimal_pickup_positions(self.string_length, &self.weights);
            }
            
            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);
            
            // Results
            ui.horizontal(|ui| {
                ui.label("Bridge Pickup:");
                ui.colored_label(Color32::LIGHT_BLUE, format!("{:.2} mm from bridge", self.optimal_positions.bridge_position));
                ui.label(format!("({:.1}%)", (self.optimal_positions.bridge_position / self.string_length) * 100.0));
            });

            ui.horizontal(|ui| {
                ui.label("Neck Pickup:");
                ui.colored_label(Color32::LIGHT_BLUE, format!("{:.2} mm from bridge", self.optimal_positions.neck_position));
                ui.label(format!("({:.1}%)", (self.optimal_positions.neck_position / self.string_length) * 100.0));
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
        let mut current_y = heat_map_y + heat_map_height + 25.0;
        let harmonic_spacing = 22.0; // More compact spacing
        
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
                Stroke::new(1.5, Color32::GRAY)
            );
            
            // Draw anti-nodes (all with consistent opacity)
            for anti_node in anti_nodes {
                let x = string_start_x + (anti_node / self.string_length) * string_width;
                // Use consistent opacity for all anti-nodes, regardless of weight
                let color = Color32::from_rgb(255, 100, 100);
                
                painter.circle_filled(Pos2::new(x, string_y), 4.0, color);
                painter.circle_stroke(Pos2::new(x, string_y), 4.0, Stroke::new(1.0, Color32::WHITE));
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
        

        let bridge_color = Color32::parse_hex(0xB57EDC);
        let neck_color = Color32::parse_hex(0xB266FF);

        // Draw bridge pickup position line
        let bridge_x = string_start_x + (self.optimal_positions.bridge_position / self.string_length) * string_width;
        painter.line_segment(
            [
                Pos2::new(bridge_x, heat_map_y),
                Pos2::new(bridge_x, current_y - harmonic_spacing),
            ],
            Stroke::new(2.0, bridge_color),
        );

        // Draw bridge pickup label
        painter.text(
            Pos2::new(bridge_x, heat_map_y - 25.0),
            egui::Align2::CENTER_BOTTOM,
            "Bridge",
            egui::FontId::proportional(11.0),
            bridge_color,
        );

        // Draw neck pickup position line
        let neck_x = string_start_x + (self.optimal_positions.neck_position / self.string_length) * string_width;
        painter.line_segment(
            [
                Pos2::new(neck_x, heat_map_y),
                Pos2::new(neck_x, current_y - harmonic_spacing),
            ],
            Stroke::new(2.0, neck_color),
        );
        
        // Draw neck pickup label
        painter.text(
            Pos2::new(neck_x, heat_map_y - 25.0),
            egui::Align2::CENTER_BOTTOM,
            "Neck",
            egui::FontId::proportional(11.0),
            neck_color,
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

struct OptimalPositions {
    bridge_position: f32,
    neck_position: f32,
}

fn find_optimal_pickup_positions(length: f32, weights: &[f32; 6]) -> OptimalPositions {
    // Search in the first 50% of string length from bridge (typical pickup placement)
    let search_limit = (length * 0.5) as usize;
    let resolution = 1000;

    // Calculate score at each position
    let scores: Vec<(f32, f32)> = (0..=search_limit)
        .map(|i| {
            let pos = (i as f32 / resolution as f32) * length;
            let score: f32 = (2..=7_u8)
                .zip(weights.iter())
                .map(|(harmonic, &weight)| {
                    let min_dist = get_anti_nodes_for_harmonic(length, harmonic)
                        .into_iter()
                        .map(|anti_node| (pos - anti_node).abs())
                        .min_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap();

                    // Use same sine wave falloff as heat map
                    let wavelength = length / (harmonic as f32 * 2.0);
                    let normalized_dist = (min_dist / wavelength).min(1.0);
                    let falloff = (normalized_dist * std::f32::consts::PI / 2.0).cos();

                    weight * falloff
                })
                .sum();

            (pos, score)
        })
        .collect();

    // Find the first peak (bridge pickup) - global maximum
    let (bridge_idx, &(bridge_pos, bridge_score)) = scores
        .iter()
        .enumerate()
        .max_by(|(_, (_, a)), (_, (_, b))| a.partial_cmp(b).unwrap())
        .unwrap();

    // Exclude region around the bridge peak
    // We need to expand outward from the peak until values stop decreasing
    // Use a minimum exclusion window as well (e.g., 10% of string length)
    let min_exclusion_distance = length * 0.1; // Pickups should be at least 10% of length apart

    // Find exclusion range by walking away from peak until score increases again
    let mut left_bound = bridge_idx;
    let mut right_bound = bridge_idx;

    // Walk left
    let mut prev_score = bridge_score;
    for i in (0..bridge_idx).rev() {
        let curr_score = scores[i].1;
        if curr_score > prev_score {
            // Score started increasing again, stop here
            break;
        }
        if (bridge_pos - scores[i].0).abs() >= min_exclusion_distance {
            // We've gone far enough to consider this outside the peak region
            if curr_score < bridge_score * 0.5 {
                // If we've dropped below 50% of peak, we're definitely clear
                break;
            }
        }
        left_bound = i;
        prev_score = curr_score;
    }

    // Walk right
    prev_score = bridge_score;
    for i in (bridge_idx + 1)..scores.len() {
        let curr_score = scores[i].1;
        if curr_score > prev_score {
            // Score started increasing again, stop here
            break;
        }
        if (scores[i].0 - bridge_pos).abs() >= min_exclusion_distance {
            // We've gone far enough to consider this outside the peak region
            if curr_score < bridge_score * 0.5 {
                // If we've dropped below 50% of peak, we're definitely clear
                break;
            }
        }
        right_bound = i;
        prev_score = curr_score;
    }

    // Find the second peak (neck pickup) from the remaining data
    // Search in regions [0..left_bound] and [right_bound..end]
    let neck_pos = scores[0..left_bound]
        .iter()
        .chain(scores[right_bound..].iter())
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|&(pos, _)| pos)
        .unwrap_or(length * 0.3); // Fallback to 30% if no second peak found

    OptimalPositions {
        bridge_position: bridge_pos,
        neck_position: neck_pos,
    }
}

fn heat_to_color(normalized_heat: f32) -> Color32 {
    let heat = normalized_heat.clamp(0.0, 1.0);

    // Blue to yellow gradient (colorblind-friendly)
    // Oklab interpolation provides smooth perceptual transition
    let viridis_stops: [(f32, Oklab); 2] = [
        (0.0, Srgb::parse_hex(0x0000FF).into_color()), // Blue
        (1.0, Srgb::parse_hex(0xFFFF00).into_color()), // Yellow
    ];

    // Simple linear interpolation between the two colors
    let (lower_idx, upper_idx, t) = (0, 1, heat);

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