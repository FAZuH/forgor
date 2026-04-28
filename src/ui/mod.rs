pub mod router;
pub mod tui;
pub mod update;

pub use router::Navigation;
pub use router::Page;
pub use router::Router;
pub use update::SettingsCmd;
pub use update::SettingsMsg;
pub use update::SettingsUpdate;
pub use update::TimerCmd;
pub use update::TimerMsg;
pub use update::TimerUpdate;
pub use update::Update;

use crate::config::Config;
use crate::models::Pomodoro;
use crate::ui::tui::TuiError;

#[derive(Debug, thiserror::Error)]
pub enum UiError {
    #[error(transparent)]
    TuiError(#[from] TuiError),
}

pub struct AppModel {
    pub timer: Pomodoro,
    pub settings: Config,
}

pub trait View {
    fn run(&mut self) -> Result<(), UiError>;
}
