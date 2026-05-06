# UI Architecture Rules

## Layers

- **Core** (`AppCore`): Business state, effects, session tracking. Single source of truth.
- **Views** (`TuiTimerView`, `TuiSettingsView`): Interaction-local UI state only. Render from borrowed core state.
- **Runner** (`TuiRunner`): Terminal IO, input polling, routing input to messages.

## State Ownership

A field belongs in **Core** when:
- It affects program behavior (sounds, DB, hooks, timer flow).
- Another view, the runner, or core logic reads it.
- It is meaningful when the user is on a different page.
- It is the "source of truth" for business state.

A field belongs in a **View** when:
- It is purely about what's highlighted, visible, or being typed.
- Resetting it has zero consequence outside the view.
- No other code in the system reads it.

### Examples

| Field | Location | Reason |
|-------|----------|--------|
| `is_prompting_transition` | Core | Blocks ticks, drives session-end logic, runner needs it for routing |
| `active_session_id` | Core | Drives DB updates, read by pause/resume/skip handlers |
| `config_snapshot` | Core | Drives dirty flag for save prompting |
| `scroll_state` | View | Only affects which rows are scrolled into view |
| `selected` (SettingsItem) | View | Only affects highlight/description rendering |
| `prompt` (text editing) | View | Temporary input buffer |
| `show_keybinds` | View | Visual toggle for help text |

## Message Flow

```
Input -> Runner -> View.update() -> ViewCmd -> Runner -> Core.dispatch(ViewCmd) -> Core
```

- Runner routes keyboard events to the active view's update function only for **view-local state**.
- Runner dispatches directly to Core (PomodoroMsg, RouterMsg) for **business logic**.
- Views never hold copies of core state — they receive it by reference at render time.
- Runner reads guard conditions (e.g. `core.is_prompting_transition()`) directly from Core, not from views.

## The dsp! Macro

| Branch | Use For |
|--------|---------|
| `dsp!(self, pomo, msg)` | Dispatching PomodoroMsg directly to core |
| `dsp!(self, config, msg)` | Dispatching ConfigMsg directly to core |
| `dsp!(self, router, msg)` | Dispatching RouterMsg to core |
| `dsp!(self, timer, msg)` | Updating view-local timer state, forwarding TimerCmd to core |
| `dsp!(self, setting, msg)` | Updating view-local settings state, forwarding SettingsCmd to core |
| `dsp!(self, msg, msg)` | Dispatching any Msg directly to core (raw dispatch + redraw) |

Always use `dsp!` instead of calling `self.core.dispatch()` directly — it ensures `self.redraw = true`.