pub mod settings;
pub mod timer;
pub use settings::SettingsCmd;
pub use settings::SettingsMsg;
pub use settings::SettingsUpdate;
pub use timer::TimerCmd;
pub use timer::TimerMsg;
pub use timer::TimerUpdate;

pub trait Update {
    type Model;
    type Msg;
    type Cmd;
    fn update(msg: Self::Msg, model: &mut Self::Model) -> Self::Cmd;
}
