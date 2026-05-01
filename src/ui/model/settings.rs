use std::fmt::Display;

use strum::EnumCount;
use strum::EnumMessage;
use strum::FromRepr;
use strum::IntoStaticStr;
use tui_widgets::prompts::FocusState;
use tui_widgets::prompts::State;
use tui_widgets::prompts::TextState;
use tui_widgets::scrollview::ScrollViewState;

use crate::config::pomodoro::PomodoroConfig;
use crate::ui::prelude::*;
use crate::ui::tui::view::settings::SettingsPrompt;

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsMsg {
    SelectUp,
    SelectDown,
    SectionPrev,
    SectionNext,
    SectionSelect(u32),
    ScrollUp,
    ScrollDown,
    // StartEditing(PomodoroConfig),
    CancelEditing,
    TakeEditValue,
    SetUnsavedChanges(bool),
    SetShowKeybinds(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsCmd {
    None,
    EditValue(Option<String>),
}

impl SettingsItem {
    pub fn index(&self) -> u32 {
        *self as u32
    }

    pub fn from_index(idx: u32) -> Option<Self> {
        Self::from_repr(idx as usize)
    }

    pub fn label_long(&self) -> &'static str {
        self.get_detailed_message().unwrap()
    }

    pub fn label(&self) -> &'static str {
        self.get_message().unwrap()
    }

    pub fn section(&self) -> SettingsSection {
        SettingsSection::from_item_index(self.index()).unwrap()
    }

    pub fn is_toggle(&self) -> bool {
        Self::toggles().contains(self)
    }

    pub fn is_percentage(&self) -> bool {
        Self::percentages().contains(self)
    }

    pub fn is_path(&self) -> bool {
        self.paths().contains(self)
    }

    pub fn paths(&self) -> Vec<Self> {
        use SettingsItem::*;
        vec![AlarmPathFocus, AlarmPathLong, AlarmPathShort]
    }

    fn toggles() -> Vec<Self> {
        use SettingsItem::*;
        vec![
            TimerAutoFocus,
            TimerAutoShort,
            TimerAutoLong,
            AutoStartOnLaunch,
        ]
    }

    fn percentages() -> Vec<Self> {
        use SettingsItem::*;
        vec![AlarmVolumeFocus, AlarmVolumeLong, AlarmVolumeShort]
    }
}

impl Display for SettingsItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label_long())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumCount, FromRepr, IntoStaticStr)]
pub enum SettingsSection {
    Timer,
    Hook,
    Alarm,
}

impl SettingsSection {
    pub fn from_item_index(idx: u32) -> Option<Self> {
        use SettingsSection::*;
        let ret = match idx {
            0..=7 => Timer,
            8..=10 => Hook,
            11..=16 => Alarm,
            _ => return None,
        };
        Some(ret)
    }

    pub fn item_begin_idx(&self) -> u32 {
        use SettingsSection::*;
        match self {
            Timer => 0,
            Hook => 8,
            Alarm => 11,
        }
    }

    pub fn from_index(idx: u32) -> Option<Self> {
        Self::from_repr(idx as usize)
    }

    pub fn index(&self) -> u32 {
        *self as u32
    }

    pub fn label(&self) -> &'static str {
        self.into()
    }
}

impl From<SettingsItem> for SettingsSection {
    fn from(value: SettingsItem) -> Self {
        value.section()
    }
}

pub struct SettingsModel {
    selected: SettingsItem,
    scroll_state: ScrollViewState,
    prompt: Option<SettingsPrompt>,
    has_unsaved_changes: bool,
    show_keybinds: bool,
}

impl Updateable for SettingsModel {
    type Msg = SettingsMsg;
    type Cmd = SettingsCmd;

    fn update(&mut self, msg: Self::Msg) -> Self::Cmd {
        use SettingsMsg::*;
        let mut cmd = SettingsCmd::None;

        match msg {
            SelectUp => self.select_up(),
            SelectDown => self.select_down(),
            SectionPrev => self.prev_section(),
            SectionNext => self.next_section(),
            SectionSelect(idx) => self.select_section(SettingsSection::from_index(idx).unwrap()),
            ScrollUp => self.scroll_up(),
            ScrollDown => self.scroll_down(),
            CancelEditing => self.cancel_editing(),
            TakeEditValue => {
                cmd = SettingsCmd::EditValue(
                    self.prompt.take().map(|v| v.text_state.value().to_string()),
                );
            }
            SetUnsavedChanges(v) => self.has_unsaved_changes = v,
            SetShowKeybinds(v) => self.show_keybinds = v,
        }

        cmd
    }
}

impl SettingsModel {
    pub fn new() -> Self {
        Self {
            selected: SettingsItem::TimerFocus,
            scroll_state: ScrollViewState::default(),
            prompt: None,
            has_unsaved_changes: false,
            show_keybinds: false,
        }
    }

    pub fn take_edit_value(&mut self) -> String {
        if let SettingsCmd::EditValue(Some(v)) = self.update(SettingsMsg::TakeEditValue) {
            v
        } else {
            String::new()
        }
    }

    pub fn prompt_state_mut(&mut self) -> Option<&mut SettingsPrompt> {
        self.prompt.as_mut()
    }

    pub fn scroll_state_mut(&mut self) -> &mut ScrollViewState {
        &mut self.scroll_state
    }

    pub fn has_unsaved_changes(&self) -> bool {
        self.has_unsaved_changes
    }

    /// Get current selection index
    pub fn selected(&self) -> SettingsItem {
        self.selected
    }

    /// Check if currently editing
    pub fn is_editing(&self) -> bool {
        self.prompt.is_some()
    }

    pub fn show_keybinds(&self) -> bool {
        self.show_keybinds
    }

    pub fn toggle_keybinds(&mut self) {
        let new = !self.show_keybinds;
        self.update(SettingsMsg::SetShowKeybinds(new));
    }

    pub fn start_editing_for_field(&mut self, config: &PomodoroConfig) {
        let alarm = &config.alarm;
        let hook = &config.hook;
        let timer = &config.timer;
        use SettingsItem::*;

        let mut value = match self.selected {
            TimerFocus => format!("{}", timer.focus.as_secs() / 60),
            TimerShort => format!("{}", timer.short.as_secs() / 60),
            TimerLong => format!("{}", timer.long.as_secs() / 60),
            TimerLongInterval => format!("{}", timer.long_interval),
            HookFocus => hook.focus.clone(),
            HookShort => hook.short.clone(),
            HookLong => hook.long.clone(),
            AlarmPathFocus => alarm.focus.path(),
            AlarmPathShort => alarm.short.path(),
            AlarmPathLong => alarm.long.path(),
            AlarmVolumeFocus => alarm.focus.volume(),
            AlarmVolumeShort => alarm.short.volume(),
            AlarmVolumeLong => alarm.long.volume(),
            AutoStartOnLaunch | TimerAutoFocus | TimerAutoShort | TimerAutoLong => return,
        };

        if self.selected.is_percentage() {
            value = value[..value.len() - 1].to_string();
        }

        let value_len = value.len();
        let mut text_state = TextState::new()
            .with_focus(FocusState::Focused)
            .with_value(value);
        *State::position_mut(&mut text_state) = value_len;

        self.prompt = Some(SettingsPrompt {
            text_state,
            label: self.selected().to_string(),
        });
    }

    /// Select item up
    fn select_up(&mut self) {
        let idx = self
            .selected
            .index()
            .saturating_sub(1)
            .clamp(0, SettingsItem::COUNT as u32 - 1); // 13 items total
        self.selected = SettingsItem::from_index(idx).unwrap();
    }

    /// Select item down
    fn select_down(&mut self) {
        let idx = self
            .selected
            .index()
            .saturating_add(1)
            .clamp(0, SettingsItem::COUNT as u32 - 1); // 13 items total
        self.selected = SettingsItem::from_index(idx).unwrap();
    }

    fn prev_section(&mut self) {
        let idx = (self.selected.section().index() + SettingsItem::COUNT as u32 - 1)
            % SettingsItem::COUNT as u32;
        self.selected =
            SettingsItem::from_index(SettingsSection::from_index(idx).unwrap().item_begin_idx())
                .unwrap();
    }

    fn next_section(&mut self) {
        let idx = (self.selected.section().index() + 1) % SettingsItem::COUNT as u32;
        self.selected =
            SettingsItem::from_index(SettingsSection::from_index(idx).unwrap().item_begin_idx())
                .unwrap();
    }

    fn select_section(&mut self, section: SettingsSection) {
        self.selected = SettingsItem::from_index(section.item_begin_idx()).unwrap();
    }

    /// Scroll up by one row
    fn scroll_up(&mut self) {
        self.scroll_state.scroll_up();
    }

    /// Scroll down by one row
    fn scroll_down(&mut self) {
        self.scroll_state.scroll_down();
    }

    /// Cancel editing
    fn cancel_editing(&mut self) {
        self.prompt = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_idx() {
        use SettingsItem::*;
        use SettingsSection::*;
        assert_eq!(TimerFocus.section(), Timer);
        assert_eq!(TimerAutoLong.section(), Timer);

        assert_eq!(HookFocus.section(), Hook);
        assert_eq!(HookLong.section(), Hook);

        assert_eq!(AlarmPathFocus.section(), Alarm);
        assert_eq!(AlarmVolumeLong.section(), Alarm);
    }
}
