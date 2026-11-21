//! Cross-cutting building blocks: sizes, colors, and icon caches.
//!
//! Nothing here draws a panel. `metrics` owns every chrome size, `theme` maps the
//! nine palettes onto egui visuals, and the icon caches rasterize SVGs on demand.

pub mod metrics;
pub mod style;
pub mod theme;

pub(crate) mod icons;
#[cfg(feature = "lucide-icons")]
pub(crate) mod lucide;
#[cfg(feature = "material-icons")]
pub(crate) mod material;
