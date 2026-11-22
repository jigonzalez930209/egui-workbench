//! The demo charts and their default appearance.

use egui_workbench::components::plot::{LegendPosition, Minor, PlotConfig, Scale};

/// The demo charts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chart {
    /// Signal, samples with error bars, and a fit.
    Signal,
    /// The live stream.
    Live,
    /// The histogram.
    Histogram,
    /// The log–log response.
    Bode,
    /// Temperature (left axis) and rainfall (right axis).
    Climate,
}

impl Chart {
    /// Every chart, in page order.
    pub const ALL: [Self; 5] = [Self::Signal, Self::Live, Self::Histogram, Self::Bode, Self::Climate];

    /// A short name for dialogs and file names.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Signal => "Signal",
            Self::Live => "Live stream",
            Self::Histogram => "Histogram",
            Self::Bode => "Low-pass response",
            Self::Climate => "Climate (two axes)",
        }
    }

    /// The base of exported file names.
    pub fn file_stem(self) -> &'static str {
        match self {
            Self::Signal => "signal",
            Self::Live => "live-stream",
            Self::Histogram => "histogram",
            Self::Bode => "low-pass-response",
            Self::Climate => "climate",
        }
    }

    /// Position in [`Self::ALL`].
    pub fn index(self) -> usize {
        self as usize
    }

    /// The default appearance of this chart.
    pub fn default_config(self) -> PlotConfig {
        let mut config = PlotConfig::default();
        match self {
            Self::Signal => {
                config.title = Some("Signal, samples, and a least-squares fit".to_owned());
                config.x.label = Some("Time (s)".to_owned());
                config.y.label = Some("Amplitude (V)".to_owned());
            }
            Self::Live => {
                config.title = Some("Live stream (last 10 s)".to_owned());
                config.x.label = Some("t (s)".to_owned());
                config.y.label = Some("Value".to_owned());
                config.legend = LegendPosition::BottomLeft;
            }
            Self::Histogram => {
                config.title = Some("Histogram".to_owned());
                config.x.label = Some("Bin".to_owned());
                config.y.label = Some("Count".to_owned());
            }
            Self::Bode => {
                config.title = Some("Low-pass response (log–log)".to_owned());
                config.x.label = Some("Frequency (Hz)".to_owned());
                config.y.label = Some("Gain".to_owned());
                config.x.scale = Scale::Log10;
                config.y.scale = Scale::Log10;
                config.legend = LegendPosition::BottomLeft;
            }
            Self::Climate => {
                config.title = Some("Temperature and rainfall".to_owned());
                config.x.label = Some("Month".to_owned());
                config.x.major_step = Some(1.0);
                config.x.minor = Minor::Off;
                config.y.label = Some("Temperature (°C)".to_owned());
                config.y2.label = Some("Rainfall (mm)".to_owned());
                config.y2.min = Some(0.0);
            }
        }
        config
    }
}
