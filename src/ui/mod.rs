pub mod core;
pub mod prelude;
pub mod router;
pub mod runtime;
pub mod traits;
pub mod tui;
pub mod update;

pub use traits::Runner;

use crate::ui::tui::TuiError;

#[derive(Debug, thiserror::Error)]
pub enum UiError {
    #[error(transparent)]
    TuiError(#[from] TuiError),
}
