# UI Architecture Rules

## Layers

- **Core** (`AppCore`): Business state, session tracking, configuration. Single source of truth.
- **Views** (`TuiTimerView`, `TuiSettingsView`): Interaction-local UI state only. Render from borrowed core state.
- **Runner** (`TuiRunner`): Terminal IO, input polling, routing input to messages.
- **Repo** (`src/repo/`): Database access. Returns domain models (`Session`, `Task`), never `Row` types. All `Row` → domain mapping happens inside this layer.
- **Effects** (`TuiEffectHandler`): Executes side-effects (sound, notifications, DB ops). Translates domain commands to repo calls.

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
| `overlay` (`Option<Overlay>`) | Core | Unified modal state, drives input routing in runner, guards session-end effects |
| `active_session_id` | Core | Drives DB updates, read by pause/resume/skip handlers |
| `config_snapshot` | Core | Drives dirty flag for save prompting |
| `scroll_state` | View | Only affects which rows are scrolled into view |
| `selected` (SettingsItem) | View | Only affects highlight/description rendering |
| `prompt` (text editing) | View | Temporary input buffer |
| `show_keybinds` | View | Visual toggle for help text |

## Domain vs Data Boundary

```
┌─────────────────────────────┐
│  Core / Views / Runner      │  ← Only use domain models (Mode, Session, Task, Config)
├─────────────────────────────┤
│  TuiEffectHandler           │  ← Bridges Cmd enum (domain) to repository calls
├─────────────────────────────┤
│  repos (traits)             │  ← Trait signatures return domain models, accept Mode
├─────────────────────────────┤
│  repos (sqlite)             │  ← Impls map Row → Domain before returning
│  repos (model)              │  ← From impls: SessionRow → Session, Mode → PomodoroState
└─────────────────────────────┘
```

Rules:
- Domain types (`Model`, `Session`, `Task`, `Project`, `Tag`) are defined in `src/model/`.
- Repository traits in `src/repo/traits.rs` accept and return only domain types.
- Row types (`SessionRow`, `TaskRow`, `PomodoroState`) are private to `src/repo/`.
- `From` / `Into` conversions between Row and domain types live in `src/repo/model.rs`.
- Core (`AppCore`, `Cmd`) must never import or reference `crate::repo::model::*`.

## Message Flow

```
Input → Runner ─┬─→ View.update() → ViewCmd ─→ Core.dispatch(ViewCmd) → Core
                 └─→ Core.dispatch(PomodoroMsg / RouterMsg) directly
```

- Runner routes keyboard events to the active view's update function only for **view-local state**.
- Runner dispatches directly to Core (`PomodoroMsg`, `RouterMsg`) for **business logic**.
- Views never hold copies of core state — they receive it by reference at render time.
- Runner reads guard conditions (e.g. `core.overlay()`) directly from Core, not from views.
- The `dsp!()` macro (`dsp!(self, pomo/timer/setting/router/core, msg)`) provides typed dispatch wrappers and triggers redraws.

## Responsibilities

- **Cmd** is emitted by Core when it needs the outside world (DB, sound, notifications) to do something.
- **Msg** is emitted to Core when the outside world (user, timer, effect result) needs to tell Core something.
- **EffectHandler** translates `Cmd` variants into infrastructure calls and returns result `Msg` variants.
