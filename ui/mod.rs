pub mod components;
pub mod confirm;
pub mod fuzzy;
pub mod header;
pub mod panels;
pub mod style;
pub mod theme;
pub mod toggle;
pub mod typing_area;
pub mod updater;
pub mod velocity_graph;

pub use components::{ButtonVariant, UnifiedButton, UnifiedInput, VelocityGraphWidget};
pub use confirm::ConfirmModal;
pub use fuzzy::{FuzzyPalette, PaletteAction};
pub use header::{HeaderScreen, HeaderWidget};
pub use panels::{EditorPanel, ResultsView, SettingsPanel, TypingView};
pub use theme::{Theme, ThemeId};
pub use toggle::UnifiedToggle;
pub use typing_area::TypingAreaWidget;
pub use updater::{AppUpdater, UpdateStatus};

