//! The versioned application specification and its JSON Schema.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The specification version understood by this host.
pub const SPEC_VERSION: u32 = 1;

/// A complete application description.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct HostSpec {
    /// Specification version. A host rejects a newer version.
    pub version: u32,
    /// Window title and size.
    pub window: WindowSpec,
    /// Theme and motion preferences.
    pub theme: ThemeSpec,
    /// Commands, their shortcuts, and contexts.
    pub commands: Vec<CommandSpec>,
    /// Top-level menus.
    pub menus: Vec<MenuSpec>,
    /// Explorer tree.
    pub explorer: Vec<TreeSpec>,
    /// Editor groups and tabs.
    pub layout: LayoutSpec,
    /// Which side and bottom panels are visible.
    pub panels: PanelsSpec,
    /// Status bar items.
    pub status: StatusSpec,
    /// The content widget tree.
    pub widgets: Vec<WidgetSpec>,
    /// Dialogs declared by the spec.
    pub dialogs: Vec<DialogSpec>,
    /// Toasts queued on startup.
    pub toasts: Vec<ToastSpec>,
    /// Extra application metadata.
    pub metadata: BTreeMap<String, String>,
}

impl Default for HostSpec {
    fn default() -> Self {
        Self {
            version: SPEC_VERSION,
            window: WindowSpec::default(),
            theme: ThemeSpec::default(),
            commands: Vec::new(),
            menus: Vec::new(),
            explorer: Vec::new(),
            layout: LayoutSpec::default(),
            panels: PanelsSpec::default(),
            status: StatusSpec::default(),
            widgets: vec![WidgetSpec::Label {
                text: "Hello from egui-workbench-host".to_owned(),
                size: 13.0,
                tone: "default".to_owned(),
                strong: false,
            }],
            dialogs: Vec::new(),
            toasts: Vec::new(),
            metadata: BTreeMap::new(),
        }
    }
}

impl HostSpec {
    /// The JSON Schema generated from these Rust types.
    pub fn schema() -> serde_json::Value {
        serde_json::to_value(schemars::schema_for!(HostSpec)).unwrap_or(serde_json::Value::Null)
    }

    /// Parse a spec, rejecting a newer version.
    ///
    /// # Errors
    /// When the JSON is invalid or the version is newer than [`SPEC_VERSION`].
    pub fn from_json(json: &str) -> Result<Self, String> {
        let spec: Self = serde_json::from_str(json).map_err(|error| error.to_string())?;
        if spec.version > SPEC_VERSION {
            return Err(format!("unsupported spec version {}", spec.version));
        }
        Ok(spec)
    }
}

/// Window title and size.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct WindowSpec {
    /// Window title.
    pub title: String,
    /// Initial width in points.
    pub width: f32,
    /// Initial height in points.
    pub height: f32,
    /// Allow resizing.
    pub resizable: bool,
    /// Use the workbench's undecorated title bar.
    pub custom_title_bar: bool,
}

impl Default for WindowSpec {
    fn default() -> Self {
        Self {
            title: "Workbench".to_owned(),
            width: 980.0,
            height: 720.0,
            resizable: true,
            custom_title_bar: true,
        }
    }
}

/// Theme preferences.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct ThemeSpec {
    /// Theme id, such as `github-dark` or `dracula`.
    pub id: String,
    /// Follow the desktop light/dark preference.
    pub follow_system: bool,
    /// Disable animations.
    pub reduced_motion: bool,
}

impl Default for ThemeSpec {
    fn default() -> Self {
        Self {
            id: "github-dark".to_owned(),
            follow_system: false,
            reduced_motion: false,
        }
    }
}

/// One command.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct CommandSpec {
    /// Stable namespaced id.
    pub id: String,
    /// Visible title.
    pub title: String,
    /// Menu category.
    pub category: String,
    /// Default shortcut, such as `Ctrl+S`.
    pub shortcut: Option<String>,
    /// A `when` expression.
    pub when: Option<String>,
    /// Enabled state.
    pub enabled: bool,
    /// Checked state.
    pub checked: bool,
}

/// One top-level menu.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct MenuSpec {
    /// Menu title.
    pub title: String,
    /// Rows.
    pub items: Vec<MenuItemSpec>,
}

/// One menu row.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct MenuItemSpec {
    /// Row label.
    pub label: String,
    /// Command id, when the row runs a command.
    pub command: Option<String>,
    /// Draw a separator instead.
    pub separator: bool,
}

/// One explorer node.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct TreeSpec {
    /// Stable key.
    pub key: String,
    /// Visible label.
    pub label: String,
    /// Icon name.
    pub icon: Option<String>,
    /// Expanded state.
    pub expanded: bool,
    /// Children.
    pub children: Vec<TreeSpec>,
}

/// Editor groups and tabs.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct LayoutSpec {
    /// Groups left to right.
    pub groups: Vec<GroupSpec>,
    /// Active group index.
    pub active: usize,
    /// Draw groups horizontally or vertically.
    pub direction: String,
}

impl Default for LayoutSpec {
    fn default() -> Self {
        Self {
            groups: vec![GroupSpec::default()],
            active: 0,
            direction: "horizontal".to_owned(),
        }
    }
}

/// One editor group.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct GroupSpec {
    /// Stable id.
    pub id: String,
    /// Tabs in order.
    pub tabs: Vec<TabSpec>,
    /// Active tab index.
    pub active: usize,
}

/// One editor tab.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct TabSpec {
    /// Stable id.
    pub id: String,
    /// Visible title.
    pub title: String,
    /// Pinned tab.
    pub pinned: bool,
}

/// Visible panels.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct PanelsSpec {
    /// Explorer.
    pub sidebar: bool,
    /// Right inspector.
    pub inspector: bool,
    /// Bottom panel.
    pub bottom: bool,
    /// Status bar.
    pub status: bool,
}

impl Default for PanelsSpec {
    fn default() -> Self {
        Self {
            sidebar: true,
            inspector: false,
            bottom: false,
            status: true,
        }
    }
}

/// Status bar items.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct StatusSpec {
    /// Left-aligned items.
    pub left: Vec<String>,
    /// Right-aligned items.
    pub right: Vec<String>,
}

/// One content widget.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[non_exhaustive]
pub enum WidgetSpec {
    /// A label.
    Label {
        /// Text.
        text: String,
        /// Point size.
        size: f32,
        /// `default`, `muted`, `accent`, `danger`, `success`.
        tone: String,
        /// Strong weight.
        strong: bool,
    },
    /// A button that emits its command id.
    Button {
        /// Stable id.
        id: String,
        /// Label.
        label: String,
        /// `primary`, `secondary`, `ghost`, `outline`, `danger`.
        variant: String,
        /// Enabled state.
        enabled: bool,
    },
    /// A single-line text input.
    TextField {
        /// Stable id.
        id: String,
        /// Label.
        label: String,
        /// Initial value.
        value: String,
        /// Hint.
        hint: String,
    },
    /// A checkbox.
    Checkbox {
        /// Stable id.
        id: String,
        /// Label.
        label: String,
        /// Initial state.
        checked: bool,
    },
    /// A switch.
    Switch {
        /// Stable id.
        id: String,
        /// Label.
        label: String,
        /// Initial state.
        on: bool,
    },
    /// A select.
    Select {
        /// Stable id.
        id: String,
        /// Label.
        label: String,
        /// Options.
        options: Vec<String>,
        /// Selected index.
        selected: usize,
    },
    /// A slider.
    Slider {
        /// Stable id.
        id: String,
        /// Label.
        label: String,
        /// Minimum.
        min: f64,
        /// Maximum.
        max: f64,
        /// Value.
        value: f64,
        /// Step.
        step: f64,
    },
    /// A horizontal rule.
    Separator,
    /// Empty space.
    Spacer {
        /// Height in points.
        height: f32,
    },
    /// A horizontal row.
    Row {
        /// Children.
        children: Vec<WidgetSpec>,
        /// Gap in points.
        gap: f32,
    },
    /// A vertical column.
    Column {
        /// Children.
        children: Vec<WidgetSpec>,
        /// Gap in points.
        gap: f32,
    },
    /// A titled card.
    Card {
        /// Title.
        title: String,
        /// Children.
        children: Vec<WidgetSpec>,
    },
    /// A tab strip and its active content.
    Tabs {
        /// Stable id.
        id: String,
        /// Tabs.
        tabs: Vec<String>,
        /// Active index.
        active: usize,
        /// Content per tab.
        children: Vec<Vec<WidgetSpec>>,
    },
    /// An explorer-style tree.
    Tree {
        /// Stable id.
        id: String,
        /// Nodes.
        nodes: Vec<TreeSpec>,
        /// Selected key.
        selected: Option<String>,
    },
    /// A typed data grid over a host dataset.
    DataGrid {
        /// Stable id.
        id: String,
        /// Dataset name registered with `Host::set_dataset`.
        dataset: String,
        /// Viewport height.
        height: f32,
    },
    /// A plot over points or a host dataset.
    Plot {
        /// Stable id.
        id: String,
        /// Title.
        title: String,
        /// Series.
        series: Vec<SeriesSpec>,
        /// Height.
        height: f32,
        /// `cpu` or `gpu`.
        renderer: String,
    },
    /// A Rust extension widget registered with `Host::register_custom`.
    Custom {
        /// Registered widget kind.
        widget: String,
        /// Opaque properties passed to the extension.
        props: serde_json::Value,
    },
}

/// A plot series in a spec.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct SeriesSpec {
    /// Series name.
    pub name: String,
    /// `line` or `scatter`.
    pub style: String,
    /// Inline points.
    pub points: Vec<[f64; 2]>,
    /// Dataset name, when the points come from a dataset.
    pub dataset: Option<String>,
    /// X column for a dataset series.
    pub x: Option<String>,
    /// Y column for a dataset series.
    pub y: Option<String>,
    /// Color as `#rrggbb`.
    pub color: Option<String>,
    /// Line width or marker size.
    pub width: Option<f32>,
}

/// A dialog declaration.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct DialogSpec {
    /// Stable id.
    pub id: String,
    /// Title.
    pub title: String,
    /// Message.
    pub message: String,
    /// Confirm label.
    pub confirm: String,
    /// Cancel label.
    pub cancel: String,
    /// Whether it starts open.
    pub open: bool,
}

/// A toast declaration.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct ToastSpec {
    /// `info`, `success`, `warning`, `error`, or `loading`.
    pub kind: String,
    /// Title.
    pub title: String,
    /// Description.
    pub description: Option<String>,
}
