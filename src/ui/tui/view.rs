use crate::ui::view::SettingsRenderCommand;
use crate::ui::view::SettingsView;
use crate::ui::view::SettingsViewState;
use crate::ui::view::TimerRenderCommand;
use crate::ui::view::TimerView;
use crate::ui::view::TimerViewState;

pub struct TuiTimerView;

impl TuiTimerView {
    pub fn new() -> Self {
        Self
    }
}

impl TimerView for TuiTimerView {
    fn render(&self, state: TimerViewState) -> Vec<TimerRenderCommand> {
        todo!()
    }
}

pub struct TuiSettingsView;

impl TuiSettingsView {
    pub fn new() -> Self {
        Self
    }
}

impl SettingsView for TuiSettingsView {
    fn render(&self, state: SettingsViewState) -> Vec<SettingsRenderCommand> {
        todo!()
    }
}
