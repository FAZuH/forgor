use std::fmt::Display;
use std::path::PathBuf;
use std::time::Duration;

use strum::EnumCount;
use strum::EnumDiscriminants;
use strum::EnumIter;
use strum::EnumMessage;
use strum::FromRepr;
use strum::IntoStaticStr;
use strum::VariantArray;

use crate::config::Config;
use crate::config::Percentage;
use crate::config::PomodoroConfig;
use crate::ui::prelude::*;

#[derive(Clone, Debug, PartialEq, EnumDiscriminants, EnumCount, EnumMessage, FromRepr)]
#[strum_discriminants(derive(
    PartialOrd,
    Ord,
    FromRepr,
    EnumCount,
    EnumIter,
    EnumMessage,
    VariantArray,
))]
#[strum_discriminants(name(SettingsItem))]
pub enum ConfigMsg {
    // Timer section
    #[strum(message = "Focus", detailed_message = "Focus Duration")]
    TimerFocus(Duration),
    #[strum(message = "Short Break", detailed_message = "Short Break Duration")]
    TimerShort(Duration),
    #[strum(message = "Long Break", detailed_message = "Long Break Duration")]
    TimerLong(Duration),
    #[strum(
        message = "Long Break Interval",
        detailed_message = "Long Break Interval"
    )]
    TimerLongInterval(u32),
    #[strum(message = "Auto-start on Launch")]
    AutoStartOnLaunch,
    #[strum(message = "Focus")]
    TimerAutoFocus,
    #[strum(message = "Short Break")]
    TimerAutoShort,
    #[strum(message = "Long Break")]
    TimerAutoLong,

    // Hook section
    #[strum(message = "Focus", detailed_message = "Focus Hook Command")]
    HookFocus(String),
    #[strum(message = "Short Break", detailed_message = "Short Break Hook Command")]
    HookShort(String),
    #[strum(message = "Long Break", detailed_message = "Long Break Hook Command")]
    HookLong(String),

    // Alarm section
    #[strum(message = "Focus", detailed_message = "Focus Alarm Sound File Path")]
    AlarmPathFocus(Option<PathBuf>),
    #[strum(
        message = "Short Break",
        detailed_message = "Short Break Alarm Sound File Path"
    )]
    AlarmPathShort(Option<PathBuf>),
    #[strum(
        message = "Long Break",
        detailed_message = "Long Break Alarm Sound File Path"
    )]
    AlarmPathLong(Option<PathBuf>),
    #[strum(message = "Focus", detailed_message = "Focus Alarm Volume")]
    AlarmVolumeFocus(Percentage),
    #[strum(message = "Short Break", detailed_message = "Short Break Alarm Volume")]
    AlarmVolumeShort(Percentage),
    #[strum(message = "Long Break", detailed_message = "Long Break Alarm Volume")]
    AlarmVolumeLong(Percentage),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigCmd {
    None,
}

impl Updateable<ConfigMsg, ConfigCmd> for Config {
    fn update(&mut self, msg: ConfigMsg) -> Vec<ConfigCmd> {
        use ConfigMsg::*;
        let timer = &mut self.pomodoro.timer;
        let hook = &mut self.pomodoro.hook;
        let alarm = &mut self.pomodoro.alarm;
        let cmd = vec![ConfigCmd::None];
        match msg {
            // Timer
            AutoStartOnLaunch => timer.auto_start_on_launch = !timer.auto_start_on_launch,
            TimerFocus(d) => timer.focus = d,
            TimerShort(d) => timer.short = d,
            TimerLong(d) => timer.long = d,
            TimerLongInterval(n) => timer.long_interval = n,
            TimerAutoFocus => timer.auto_focus = !timer.auto_focus,
            TimerAutoShort => timer.auto_short = !timer.auto_short,
            TimerAutoLong => timer.auto_long = !timer.auto_long,
            // Hook
            HookFocus(s) => hook.focus = s,
            HookShort(s) => hook.short = s,
            HookLong(s) => hook.long = s,
            // Alarm
            AlarmPathFocus(p) => alarm.focus.path = p,
            AlarmPathShort(p) => alarm.short.path = p,
            AlarmPathLong(p) => alarm.long.path = p,
            AlarmVolumeFocus(v) => alarm.focus.volume = v,
            AlarmVolumeShort(v) => alarm.short.volume = v,
            AlarmVolumeLong(v) => alarm.long.volume = v,
        }
        cmd
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsMsg<'a> {
    /// Carries a ConfigMsg constructed by the view layer (post-split flow).
    ApplyEdit(ConfigMsg),
    StartEdit(&'a PomodoroConfig),
    CancelEditing,
    SaveConfig,
    SaveEdit,
    ScrollDown,
    ScrollUp,
    SectionNext,
    SectionPrev,
    SectionSelect(u32),
    SelectDown,
    SelectUp,
    SetShowKeybinds(bool),
    SetUnsavedChanges(bool),
    ToggleShowKeybinds,
    SelectForCopy,
    CopyValue(&'a Config),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsCmd {
    SaveEdit(ConfigMsg),
    SaveConfig,
    ShowToast { message: String, r#type: ToastType },
}

impl SettingsItem {
    pub fn index(&self) -> u32 {
        *self as u32
    }

    pub fn from_index(idx: u32) -> Option<Self> {
        Self::from_repr(idx as usize)
    }

    pub fn label_long(&self) -> &'static str {
        self.config_msg().get_detailed_message().unwrap()
    }

    fn config_msg(&self) -> ConfigMsg {
        ConfigMsg::from_repr(self.index() as usize).unwrap()
    }

    pub fn label(&self) -> &'static str {
        self.config_msg().get_message().unwrap()
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

impl Display for ConfigMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", SettingsItem::from(self).label_long())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumCount, FromRepr, IntoStaticStr, VariantArray)]
pub enum SettingsSection {
    Timer,
    Hook,
    Alarm,
}

impl SettingsSection {
    pub fn from_item(item: SettingsItem) -> Self {
        use SettingsItem::*;
        match item {
            AutoStartOnLaunch | TimerFocus | TimerShort | TimerLong | TimerLongInterval
            | TimerAutoFocus | TimerAutoShort | TimerAutoLong => Self::Timer,
            HookFocus | HookShort | HookLong => Self::Hook,
            AlarmPathFocus | AlarmPathShort | AlarmPathLong | AlarmVolumeFocus
            | AlarmVolumeShort | AlarmVolumeLong => Self::Alarm,
        }
    }

    pub fn from_item_index(idx: u32) -> Option<Self> {
        SettingsItem::from_repr(idx as usize).map(Self::from_item)
    }
    pub fn item_begin_idx(&self) -> u32 {
        use SettingsItem::*;
        match self {
            SettingsSection::Timer => TimerFocus as u32,
            SettingsSection::Hook => HookFocus as u32,
            SettingsSection::Alarm => AlarmPathFocus as u32,
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

    pub fn items(&self) -> &[SettingsItem] {
        use SettingsItem::*;
        match self {
            SettingsSection::Timer => {
                &SettingsItem::VARIANTS[AutoStartOnLaunch as usize..=TimerAutoLong as usize]
            }
            SettingsSection::Hook => {
                &SettingsItem::VARIANTS[HookFocus as usize..=HookLong as usize]
            }
            SettingsSection::Alarm => {
                &SettingsItem::VARIANTS[AlarmPathFocus as usize..=AlarmVolumeLong as usize]
            }
        }
    }
}

impl From<SettingsItem> for SettingsSection {
    fn from(value: SettingsItem) -> Self {
        value.section()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastType {
    Error,
    Warning,
    Success,
}
