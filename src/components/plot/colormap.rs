//! Perceptually uniform, diverging, and color-blind-safe colormaps.

use egui::Color32;

/// A color scale for heatmaps and colorbars.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[non_exhaustive]
pub enum Colormap {
    /// The existing theme-derived heat scale.
    #[default]
    Theme,
    /// Viridis (perceptually uniform, color-blind safe).
    Viridis,
    /// Magma.
    Magma,
    /// Cividis (color-blind safe).
    Cividis,
    /// Turbo (high contrast, not perceptually uniform).
    Turbo,
    /// Cool to warm, diverging around the midpoint.
    CoolWarm,
    /// A categorical palette, safe for the common color-vision deficiencies.
    Colorblind,
}

impl Colormap {
    /// Every built-in colormap.
    pub const ALL: [Self; 7] = [
        Self::Theme,
        Self::Viridis,
        Self::Magma,
        Self::Cividis,
        Self::Turbo,
        Self::CoolWarm,
        Self::Colorblind,
    ];

    /// A label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Theme => "Theme",
            Self::Viridis => "Viridis",
            Self::Magma => "Magma",
            Self::Cividis => "Cividis",
            Self::Turbo => "Turbo",
            Self::CoolWarm => "Cool-Warm",
            Self::Colorblind => "Colorblind",
        }
    }

    /// Sample the scale at `t` in 0..=1. `Theme` needs the active theme.
    pub fn sample(self, t: f32, theme: &crate::theme::Theme) -> Color32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Theme => crate::components::plot::extra::heat_color(t, theme),
            Self::Viridis => gradient(t, &VIRIDIS),
            Self::Magma => gradient(t, &MAGMA),
            Self::Cividis => gradient(t, &CIVIDIS),
            Self::Turbo => gradient(t, &TURBO),
            Self::CoolWarm => gradient(t, &COOL_WARM),
            Self::Colorblind => categorical(t),
        }
    }

    /// The discrete colors of a categorical palette (one per index).
    pub fn categorical(self, index: usize) -> Color32 {
        if self == Self::Colorblind {
            const PALETTE: [Color32; 8] = [
                Color32::from_rgb(0x00, 0x72, 0xb2),
                Color32::from_rgb(0xe6, 0x9f, 0x00),
                Color32::from_rgb(0x00, 0x9e, 0x73),
                Color32::from_rgb(0xcc, 0x79, 0xa7),
                Color32::from_rgb(0x56, 0xb4, 0xe9),
                Color32::from_rgb(0xd5, 0x5e, 0x00),
                Color32::from_rgb(0xf0, 0xe4, 0x42),
                Color32::from_rgb(0x99, 0x99, 0x99),
            ];
            PALETTE[index % PALETTE.len()]
        } else {
            self.sample((index % 20) as f32 / 19.0, &crate::theme::Theme::dark())
        }
    }
}

fn gradient(t: f32, stops: &[[f32; 3]]) -> Color32 {
    let position = t.clamp(0.0, 1.0) * (stops.len() - 1) as f32;
    let low = position.floor() as usize;
    let high = (low + 1).min(stops.len() - 1);
    let fraction = position - low as f32;
    let mix = |a: f32, b: f32| (a + (b - a) * fraction).round() as u8;
    Color32::from_rgb(
        mix(stops[low][0], stops[high][0]),
        mix(stops[low][1], stops[high][1]),
        mix(stops[low][2], stops[high][2]),
    )
}

fn categorical(t: f32) -> Color32 {
    const PALETTE: [Color32; 8] = [
        Color32::from_rgb(0x00, 0x72, 0xb2),
        Color32::from_rgb(0xe6, 0x9f, 0x00),
        Color32::from_rgb(0x00, 0x9e, 0x73),
        Color32::from_rgb(0xcc, 0x79, 0xa7),
        Color32::from_rgb(0x56, 0xb4, 0xe9),
        Color32::from_rgb(0xd5, 0x5e, 0x00),
        Color32::from_rgb(0xf0, 0xe4, 0x42),
        Color32::from_rgb(0x99, 0x99, 0x99),
    ];
    PALETTE[(t * (PALETTE.len() - 1) as f32).round() as usize % PALETTE.len()]
}

const VIRIDIS: [[f32; 3]; 12] = [
    [68.0, 1.0, 84.0],
    [72.0, 26.0, 108.0],
    [71.0, 47.0, 125.0],
    [65.0, 68.0, 135.0],
    [57.0, 86.0, 140.0],
    [49.0, 104.0, 142.0],
    [42.0, 120.0, 142.0],
    [35.0, 136.0, 142.0],
    [31.0, 152.0, 139.0],
    [34.0, 168.0, 132.0],
    [53.0, 183.0, 121.0],
    [84.0, 197.0, 105.0],
];
const MAGMA: [[f32; 3]; 11] = [
    [0.0, 0.0, 4.0],
    [28.0, 16.0, 68.0],
    [57.0, 13.0, 89.0],
    [85.0, 15.0, 109.0],
    [116.0, 22.0, 105.0],
    [147.0, 38.0, 103.0],
    [178.0, 55.0, 96.0],
    [209.0, 74.0, 86.0],
    [235.0, 102.0, 70.0],
    [251.0, 143.0, 60.0],
    [252.0, 253.0, 191.0],
];
const CIVIDIS: [[f32; 3]; 11] = [
    [0.0, 32.0, 76.0],
    [0.0, 44.0, 99.0],
    [0.0, 57.0, 115.0],
    [24.0, 70.0, 124.0],
    [57.0, 83.0, 126.0],
    [80.0, 96.0, 128.0],
    [102.0, 109.0, 130.0],
    [123.0, 123.0, 133.0],
    [145.0, 138.0, 135.0],
    [168.0, 154.0, 134.0],
    [255.0, 234.0, 70.0],
];
const TURBO: [[f32; 3]; 11] = [
    [48.0, 18.0, 59.0],
    [70.0, 62.0, 190.0],
    [42.0, 120.0, 245.0],
    [25.0, 181.0, 220.0],
    [40.0, 225.0, 135.0],
    [130.0, 245.0, 50.0],
    [206.0, 225.0, 30.0],
    [250.0, 171.0, 35.0],
    [246.0, 105.0, 28.0],
    [200.0, 48.0, 35.0],
    [122.0, 4.0, 3.0],
];
const COOL_WARM: [[f32; 3]; 11] = [
    [59.0, 76.0, 192.0],
    [98.0, 130.0, 234.0],
    [141.0, 176.0, 254.0],
    [184.0, 208.0, 249.0],
    [221.0, 221.0, 221.0],
    [245.0, 196.0, 173.0],
    [244.0, 154.0, 123.0],
    [222.0, 96.0, 77.0],
    [180.0, 4.0, 38.0],
    [180.0, 4.0, 38.0],
    [180.0, 4.0, 38.0],
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn samples_are_in_range_and_distinct() {
        let theme = crate::theme::Theme::dark();
        for map in Colormap::ALL {
            let start = map.sample(0.0, &theme);
            let end = map.sample(1.0, &theme);
            assert_ne!(start, end, "{}", map.label());
            assert_eq!(map.sample(-1.0, &theme), start, "{}", map.label());
            assert_eq!(map.sample(2.0, &theme), end, "{}", map.label());
        }
    }

    #[test]
    fn the_categorical_palette_wraps() {
        for index in 0..16 {
            let color = Colormap::Colorblind.categorical(index);
            assert_ne!(color, Color32::TRANSPARENT);
        }
    }
}
