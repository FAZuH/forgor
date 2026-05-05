use std::collections::VecDeque;

use crate::ui::core::AppCore;
use crate::ui::core::Cmd;
use crate::ui::core::Msg;

/// backend-specific effect execution.
pub trait EffectHandler {
    /// Execute a command and return any result messages.
    fn execute(&mut self, cmd: Cmd) -> Vec<Msg>;
}

pub struct Runtime<E: EffectHandler> {
    core: AppCore,
    effects: E,
}

impl<E: EffectHandler> Runtime<E> {
    pub fn new(core: AppCore, effects: E) -> Self {
        Self { core, effects }
    }

    /// Dispatch a message and process all resulting effects
    /// synchronously until the message queue is drained.
    pub fn dispatch(&mut self, msg: Msg) {
        let mut queue = VecDeque::from([msg]);
        while let Some(msg) = queue.pop_front() {
            for cmd in self.core.update(msg) {
                for result_msg in self.effects.execute(cmd) {
                    queue.push_back(result_msg);
                }
            }
        }
    }

    // -- Accessors for rendering --

    pub fn core(&self) -> &AppCore {
        &self.core
    }

    pub fn core_mut(&mut self) -> &mut AppCore {
        &mut self.core
    }

    pub fn effects(&self) -> &E {
        &self.effects
    }

    pub fn effects_mut(&mut self) -> &mut E {
        &mut self.effects
    }

    pub fn split_mut(&mut self) -> (&mut AppCore, &mut E) {
        (&mut self.core, &mut self.effects)
    }
}
