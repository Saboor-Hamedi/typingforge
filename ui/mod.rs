pub mod components;
pub mod confirm;
pub mod header;
pub mod panels;
pub mod results;
pub mod settings;
pub mod theme;
pub mod typing_area;
pub mod updater;
pub mod velocity_graph;

pub use components::{ButtonVariant, UnifiedButton, UnifiedInput, VelocityGraphWidget};
pub use confirm::ConfirmModal;
pub use header::{HeaderScreen, HeaderWidget};
pub use panels::{EditorPanel, ResultsView, SettingsPanel, TypingView};
pub use results::ResultsScreen;
pub use settings::SettingsScreen;
pub use theme::{Theme, ThemeId};
pub use typing_area::TypingAreaWidget;
pub use updater::{AppUpdater, UpdateStatus};

