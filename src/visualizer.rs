use egui::{Color32, Pos2, Stroke, Vec2};

use crate::calculation::get_anti_nodes_for_harmonic;
use crate::color::{ColorExt, HEATMAP_COLORS, heat_to_color};

use crate::app::HarmonicApp;

impl HarmonicApp {
    pub(crate) fn calculate_visualizer_height(&self) -> f32 {
        // These constants must match the values used in draw_visualization()
        const TOP_PADDING: f32 = 50.0; // Distance from rect.min.y to heat map
        const LABEL_HEIGHT: f32 = 10.0; // Space above heat map for label
        const HEAT_MAP_HEIGHT: f32 = 60.0;
        const GAP_AFTER_HEAT_MAP: f32 = 25.0;
        const HARMONIC_SPACING: f32 = 22.0;
        const NUM_HARMONICS: usize = 6; // Harmonics 2-7
        const BOTTOM_PADDING: f32 = 10.0; // Space for bridge/nut labels

        TOP_PADDING - LABEL_HEIGHT
            + HEAT_MAP_HEIGHT
            + GAP_AFTER_HEAT_MAP
            + (NUM_HARMONICS as f32 * HARMONIC_SPACING)
            + BOTTOM_PADDING
    }

    pub(crate) fn draw_visualization(&self, ui: &mut egui::Ui) {
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
                Stroke::new(1.5, Color32::GRAY),
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
        let bridge_x = string_start_x
            + (self.optimal_positions.bridge_position / self.string_length) * string_width;
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
        let neck_x = string_start_x
            + (self.optimal_positions.neck_position / self.string_length) * string_width;
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

    pub(crate) fn calculate_heat_map(&self) -> Vec<f32> {
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
