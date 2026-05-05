pub mod backend;
pub mod effect;
pub mod runner;
pub mod toast;
pub mod view;

pub use effect::TuiEffectHandler;
pub use runner::TuiRunner;

#[derive(Debug, thiserror::Error)]
pub enum TuiError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("error during initialization: {0}")]
    InitializeError(String),
}
