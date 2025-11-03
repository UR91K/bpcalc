use egui::Color32;
use palette::{IntoColor, Mix, Oklab, Srgb};

pub(crate) const HEATMAP_COLORS: [i32; 7] = [
    0x000000, 0x0000FF, 0x00FFFF, 0x00FF00, 0xFFFF00, 0xFF0000, 0xFFFFFF,
];

/// Extension trait to create Srgb from hex color codes
pub(crate) trait ColorExt {
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

pub(crate) fn heat_to_color(normalized_heat: f32, hex_colors: &[i32]) -> Color32 {
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
