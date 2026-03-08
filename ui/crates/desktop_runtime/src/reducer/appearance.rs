//! Reducer helpers for desktop theme transitions.

use crate::{
    model::DesktopState,
    reducer::{DesktopAction, ReducerError, RuntimeEffect},
};

pub(super) fn reduce_appearance_action(
    state: &mut DesktopState,
    action: &DesktopAction,
    effects: &mut Vec<RuntimeEffect>,
) -> Result<bool, ReducerError> {
    match action {
        DesktopAction::HydrateTheme { theme, revision } => {
            if revision.is_some_and(|incoming| {
                state
                    .theme_revision
                    .is_some_and(|current| incoming <= current)
            }) {
                return Ok(true);
            }
            state.theme = theme.clone();
            if let Some(revision) = revision {
                state.theme_revision = Some(*revision);
            }
        }
        DesktopAction::SetHighContrast { enabled } => {
            state.theme.high_contrast = *enabled;
            effects.push(RuntimeEffect::PersistTheme);
        }
        DesktopAction::SetThemeMode { mode } => {
            state.theme.mode = *mode;
            effects.push(RuntimeEffect::PersistTheme);
        }
        DesktopAction::SetReducedMotion { enabled } => {
            state.theme.reduced_motion = *enabled;
            effects.push(RuntimeEffect::PersistTheme);
        }
        _ => return Ok(false),
    }

    Ok(true)
}
