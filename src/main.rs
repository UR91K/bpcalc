use eframe::egui;
use egui::{Color32, Pos2, Stroke, Vec2};
use palette::{Srgb, Oklab, IntoColor, Mix};

const HEATMAP_COLORS: [i32; 7] = [
    0x000000,
    0x0000FF,
    0x00FFFF,
    0x00FF00,
    0xFFFF00,
    0xFF0000,
    0xFFFFFF,
];

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
    search_limit: usize,
}

impl Default for HarmonicApp {
    fn default() -> Self {
        let string_length = 650.0; // Typical guitar scale length in mm
        let weights = [0.15, 1.50, 1.50, 1.50, 0.75, 0.75]; // Harmonics 2-7
        let search_limit = (string_length * 0.5) as usize;
        let optimal_positions = find_optimal_pickup_positions(string_length, &weights, search_limit);

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
                    if ui.add(egui::Slider::new(&mut self.string_length, 500.0..=1000.0)).changed() {
                        self.optimal_positions = find_optimal_pickup_positions(self.string_length, &self.weights, self.search_limit);
                    }

                    ui.label("Search Limit:");
                    if ui.add(egui::Slider::new(&mut self.search_limit, 1..=(self.string_length / 2.0) as usize)).changed() {
                        self.optimal_positions = find_optimal_pickup_positions(self.string_length, &self.weights, self.search_limit);
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
                        self.optimal_positions = find_optimal_pickup_positions(self.string_length, &self.weights, self.search_limit);
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

impl HarmonicApp {
    fn calculate_visualizer_height(&self) -> f32 {
        // These constants must match the values used in draw_visualization()
        const TOP_PADDING: f32 = 50.0; // Distance from rect.min.y to heat map
        const LABEL_HEIGHT: f32 = 10.0; // Space above heat map for label
        const HEAT_MAP_HEIGHT: f32 = 60.0;
        const GAP_AFTER_HEAT_MAP: f32 = 25.0;
        const HARMONIC_SPACING: f32 = 22.0;
        const NUM_HARMONICS: usize = 6; // Harmonics 2-7
        const BOTTOM_PADDING: f32 = 10.0; // Space for bridge/nut labels
        
        TOP_PADDING - LABEL_HEIGHT + HEAT_MAP_HEIGHT + GAP_AFTER_HEAT_MAP + 
        (NUM_HARMONICS as f32 * HARMONIC_SPACING) + BOTTOM_PADDING
    }
    
    fn draw_visualization(&self, ui: &mut egui::Ui) {
        // Constants - must match calculate_visualizer_height()
        const SIDE_MARGIN: f32 = 20.0;
        const TOP_PADDING: f32 = 50.0;
        const LABEL_HEIGHT: f32 = 10.0;
        const HEAT_MAP_HEIGHT: f32 = 60.0;
        const GAP_AFTER_HEAT_MAP: f32 = 25.0;
        const HARMONIC_SPACING: f32 = 22.0;
        const BOTTOM_PADDING: f32 = 10.0;
        
        let available_width = ui.available_width() - (SIDE_MARGIN * 2.0);
        let viz_height = self.calculate_visualizer_height();
        
        let (response, painter) = ui.allocate_painter(
            Vec2::new(available_width + (SIDE_MARGIN * 2.0), viz_height),
            egui::Sense::hover(),
        );
        
        let rect = response.rect;
        let string_start_x = rect.min.x + SIDE_MARGIN;
        let string_end_x = rect.max.x - SIDE_MARGIN;
        let string_width = string_end_x - string_start_x;
        
        // Draw background
        painter.rect_filled(rect, 0.0, Color32::from_gray(20));
        
        // Calculate heat map
        let heat_map = self.calculate_heat_map();
        let max_heat = heat_map.iter().cloned().fold(0.0_f32, f32::max);

        // Draw heat map
        let heat_map_y = rect.min.y + TOP_PADDING;
        
        // Draw the heat map as a series of rectangles that exactly cover the space
        // Calculate the exact width needed to avoid gaps
        let heat_map_rect = egui::Rect::from_min_max(
            Pos2::new(string_start_x, heat_map_y),
            Pos2::new(string_end_x, heat_map_y + HEAT_MAP_HEIGHT),
        );
        
        // Draw each segment as a rectangle spanning the full width
        for (i, &heat) in heat_map.iter().enumerate() {
            let normalized_heat = if max_heat > 0.0 { heat / max_heat } else { 0.0 };
            let color = heat_to_color(normalized_heat, &HEATMAP_COLORS);
            
            // Calculate segment boundaries
            let segment_start = i as f32 / heat_map.len() as f32;
            let segment_end = (i + 1) as f32 / heat_map.len() as f32;
            
            let x_start = (heat_map_rect.left() + segment_start * heat_map_rect.width()).round();
            let x_end = (heat_map_rect.left() + segment_end * heat_map_rect.width()).round();
            
            let segment_rect = egui::Rect::from_min_max(
                Pos2::new(x_start, heat_map_y),
                Pos2::new(x_end, heat_map_y + HEAT_MAP_HEIGHT),
            );
            painter.rect_filled(segment_rect, 0.0, color);
        }
        
        // Draw heat map label
        painter.text(
            Pos2::new(string_start_x, heat_map_y - LABEL_HEIGHT),
            egui::Align2::LEFT_BOTTOM,
            "Heat Map (Anti-Node Proximity)",
            egui::FontId::proportional(12.0),
            Color32::WHITE,
        );
        
        // Draw individual harmonics
        let mut current_y = heat_map_y + HEAT_MAP_HEIGHT + GAP_AFTER_HEAT_MAP;
        
        for harmonic in 2..=7_u8 {
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
                let color = Color32::parse_hex(0xA6CFA1);
                
                painter.circle_filled(Pos2::new(x, string_y), 4.0, color);
                // painter.circle_stroke(Pos2::new(x, string_y), 4.0, Stroke::new(1.0, Color32::WHITE));
            }
            
            // // Draw label (inside the bounds to prevent clipping)
            // painter.text(
            //     Pos2::new(string_start_x + 5.0, string_y),
            //     egui::Align2::LEFT_CENTER,
            //     format!("H{}", harmonic),
            //     egui::FontId::proportional(11.0),
            //     Color32::from_gray(200),
            // );
            
            // // Draw weight indicator (inside the bounds to prevent clipping)
            // painter.text(
            //     Pos2::new(string_end_x - 5.0, string_y),
            //     egui::Align2::RIGHT_CENTER,
            //     format!("w: {:.2}", weight),
            //     egui::FontId::proportional(9.0),
            //     Color32::from_gray(150),
            // );
            
            current_y += HARMONIC_SPACING;
        }
        

        let bridge_color = Color32::parse_hex(0xB57EDC);
        let neck_color = Color32::parse_hex(0xB266FF);

        // Draw bridge pickup position line
        let bridge_x = string_start_x + (self.optimal_positions.bridge_position / self.string_length) * string_width;
        painter.line_segment(
            [
                Pos2::new(bridge_x, heat_map_y),
                Pos2::new(bridge_x, current_y - HARMONIC_SPACING),
            ],
            Stroke::new(2.0, bridge_color),
        );

        // Draw bridge pickup label
        painter.text(
            Pos2::new(bridge_x, heat_map_y - LABEL_HEIGHT - 15.0),
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
                Pos2::new(neck_x, current_y - HARMONIC_SPACING),
            ],
            Stroke::new(2.0, neck_color),
        );

        // Draw neck pickup label
        painter.text(
            Pos2::new(neck_x, heat_map_y - LABEL_HEIGHT - 15.0),
            egui::Align2::CENTER_BOTTOM,
            "Neck",
            egui::FontId::proportional(11.0),
            neck_color,
        );
        
        // Draw bridge and nut labels
        painter.text(
            Pos2::new(string_start_x, rect.max.y - BOTTOM_PADDING),
            egui::Align2::CENTER_BOTTOM,
            "Bridge",
            egui::FontId::proportional(12.0),
            Color32::WHITE,
        );
        
        painter.text(
            Pos2::new(string_end_x, rect.max.y - BOTTOM_PADDING),
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

fn find_optimal_pickup_positions(length: f32, weights: &[f32; 6], search_limit: usize) -> OptimalPositions {
    // Search in the first 50% of string length from bridge (typical pickup placement)
    // let search_limit = (length * 0.5) as usize; //TODO: make this a slider
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

fn heat_to_color(normalized_heat: f32, hex_colors: &[i32]) -> Color32 {
    let heat = normalized_heat.clamp(0.0, 1.0);

    // Convert hex values to Oklab colors
    let color_stops: Vec<Oklab> = hex_colors
        .iter()
        .map(|&hex| Srgb::parse_hex(hex as u32).into_color())
        .collect();

    let num_stops = color_stops.len();
    
    // Handle edge cases
    if num_stops == 0 {
        return Color32::BLACK;
    }
    if num_stops == 1 {
        let rgb: Srgb = color_stops[0].into_color();
        return Color32::from_rgb(
            (rgb.red * 255.0) as u8,
            (rgb.green * 255.0) as u8,
            (rgb.blue * 255.0) as u8,
        );
    }

    // Calculate which segment we're in
    let segment_size = 1.0 / (num_stops - 1) as f32;
    let segment = (heat / segment_size).floor() as usize;
    
    // Clamp to valid range
    let lower_idx = segment.min(num_stops - 2);
    let upper_idx = lower_idx + 1;
    
    // Calculate interpolation factor within this segment
    let segment_start = lower_idx as f32 * segment_size;
    let t = ((heat - segment_start) / segment_size).clamp(0.0, 1.0);

    // Interpolate in Oklab space
    let oklab = color_stops[lower_idx].mix(color_stops[upper_idx], t);
    
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
        viewport: egui::ViewportBuilder::default().with_inner_size([706.0, 678.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Harmonic Anti-Node Visualizer",
        options,
        Box::new(|_cc| Ok(Box::new(HarmonicApp::default()))),
    )
}