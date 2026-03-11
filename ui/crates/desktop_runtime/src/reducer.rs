//! Reducer actions, side-effect intents, and transition logic for the desktop runtime.

mod appearance;

use desktop_app_contract::{AppCapability, AppCommand, AppEvent, AppLifecycleEvent, ApplicationId};
use serde_json::{Value, json};
use thiserror::Error;

use crate::apps;
use crate::model::{
    DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH, DeepLinkOpenTarget, DeepLinkState,
    DesktopNotification, DesktopSnapshot, DesktopState, DesktopTheme, InteractionState,
    OpenWindowRequest, PointerPosition, ResizeEdge, ResizeSession, ThemeMode, WindowId,
    WindowRecord, WindowRect,
};
use crate::window_manager::{
    MIN_WINDOW_HEIGHT, MIN_WINDOW_WIDTH, focus_window_internal, normalize_window_stack,
    resize_rect, snap_window_to_viewport_edge,
};

#[derive(Debug, Clone, PartialEq)]
/// Actions accepted by [`reduce_desktop`] to mutate [`DesktopState`].
pub enum DesktopAction {
    /// Activate an application from a launcher surface.
    ///
    /// For single-instance apps, this focuses/restores the existing window if present.
    /// For multi-instance apps (or when no instance exists), this opens a new window.
    ActivateApp {
        /// Application to activate.
        app_id: ApplicationId,
        /// Optional desktop viewport hint for adaptive default window sizing.
        viewport: Option<WindowRect>,
    },
    /// Open a new window using the supplied request.
    OpenWindow(OpenWindowRequest),
    /// Close a window by id.
    CloseWindow {
        /// Window to close.
        window_id: WindowId,
    },
    /// Focus (and raise) a window by id.
    FocusWindow {
        /// Window to focus.
        window_id: WindowId,
    },
    /// Minimize a window.
    MinimizeWindow {
        /// Window to minimize.
        window_id: WindowId,
    },
    /// Maximize a window to the provided viewport.
    MaximizeWindow {
        /// Window to maximize.
        window_id: WindowId,
        /// Viewport rectangle to maximize into.
        viewport: WindowRect,
    },
    /// Restore a minimized or maximized window.
    RestoreWindow {
        /// Window to restore.
        window_id: WindowId,
    },
    /// Toggle taskbar behavior for a window (focus, minimize, or restore).
    ToggleTaskbarWindow {
        /// Window associated with the taskbar button.
        window_id: WindowId,
    },
    /// Toggle the start menu open/closed.
    ToggleStartMenu,
    /// Close the start menu if open.
    CloseStartMenu,
    /// Toggle the right-side control center panel.
    ToggleControlCenter,
    /// Close the right-side control center panel.
    CloseControlCenter,
    /// Toggle the right-side notification center panel.
    ToggleNotificationCenter,
    /// Close the right-side notification center panel.
    CloseNotificationCenter,
    /// Begin dragging a window.
    BeginMove {
        /// Window being dragged.
        window_id: WindowId,
        /// Pointer position at drag start.
        pointer: PointerPosition,
    },
    /// Update an in-progress window drag.
    UpdateMove {
        /// Current pointer position.
        pointer: PointerPosition,
    },
    /// End the active window drag.
    EndMove,
    /// End the active window drag and apply viewport-edge snapping.
    EndMoveWithViewport {
        /// Current desktop viewport rectangle.
        viewport: WindowRect,
    },
    /// Begin resizing a window.
    BeginResize {
        /// Window being resized.
        window_id: WindowId,
        /// Edge or corner being dragged.
        edge: ResizeEdge,
        /// Pointer position at resize start.
        pointer: PointerPosition,
        /// Current desktop viewport rectangle used for resize bounds.
        viewport: WindowRect,
    },
    /// Update an in-progress window resize.
    UpdateResize {
        /// Current pointer position.
        pointer: PointerPosition,
    },
    /// End the active window resize.
    EndResize,
    /// Suspend a window instance.
    SuspendWindow {
        /// Window to suspend.
        window_id: WindowId,
    },
    /// Resume a suspended window instance.
    ResumeWindow {
        /// Window to resume.
        window_id: WindowId,
    },
    /// Handle an app-originated command for a managed window.
    HandleAppCommand {
        /// Source window id.
        window_id: WindowId,
        /// App command payload.
        command: AppCommand,
    },
    /// Hydrate theme state independently from layout restore.
    HydrateTheme {
        /// Persisted theme payload.
        theme: DesktopTheme,
        /// Revision associated with the theme payload.
        revision: Option<u64>,
    },
    /// Toggle high-contrast rendering.
    SetHighContrast {
        /// Whether high contrast is enabled.
        enabled: bool,
    },
    /// Toggle light/dark theme family.
    SetThemeMode {
        /// Theme family to activate.
        mode: ThemeMode,
    },
    /// Toggle reduced-motion rendering.
    SetReducedMotion {
        /// Whether reduced motion is enabled.
        enabled: bool,
    },
    /// Remove one retained shell notification.
    DismissNotification {
        /// Notification identifier.
        id: u64,
    },
    /// Mark retained notifications as read.
    MarkNotificationsRead,
    /// Clear retained notifications from the shell center.
    ClearNotifications,
    /// Append a command to terminal history (subject to preferences and limits).
    PushTerminalHistory {
        /// Terminal command text.
        command: String,
    },
    /// Replace the app-specific state payload for a window.
    SetAppState {
        /// Window whose app state should be replaced.
        window_id: WindowId,
        /// New app state payload.
        app_state: Value,
    },
    /// Replace app-shared state payload for `<app_id>:<key>`.
    SetSharedAppState {
        /// Source app id.
        app_id: ApplicationId,
        /// Shared state key.
        key: String,
        /// Shared state payload.
        state: Value,
    },
    /// Complete boot hydration in a single deterministic transition.
    CompleteBootHydration {
        /// Authoritative snapshot to restore when boot restore is enabled.
        snapshot: Option<DesktopSnapshot>,
        /// Durable revision for the authoritative snapshot when one exists.
        snapshot_revision: Option<u64>,
        /// Persisted theme payload.
        theme: Option<DesktopTheme>,
        /// Persisted policy overlay payload.
        privileged_app_ids: Vec<String>,
        /// Initial deep-link payload captured at mount.
        deep_link: Option<DeepLinkState>,
    },
    /// Hydrate runtime state from a persisted snapshot.
    HydrateSnapshot {
        /// Snapshot payload to restore.
        snapshot: DesktopSnapshot,
        /// Hydration intent controlling lifecycle replay.
        mode: HydrationMode,
        /// Revision associated with the snapshot payload.
        revision: Option<u64>,
    },
    /// Apply URL-derived deep-link instructions.
    ApplyDeepLink {
        /// Parsed deep-link payload.
        deep_link: DeepLinkState,
    },
    /// Records the latest applied sync revision for a state domain.
    RecordAppliedRevision {
        /// State domain whose monotonic revision advanced.
        domain: SyncDomain,
        /// New monotonic revision value.
        revision: u64,
    },
}

#[derive(Debug, Clone, PartialEq)]
/// Side-effect intents emitted by [`reduce_desktop`] for the shell runtime to execute.
pub enum RuntimeEffect {
    /// Persist the current desktop layout snapshot.
    PersistLayout,
    /// Persist theme changes.
    PersistTheme,
    /// Persist terminal history changes.
    PersistTerminalHistory,
    /// Move focus into the newly focused window's primary input.
    FocusWindowInput(WindowId),
    /// Open an external URL (for app actions that leave the shell).
    OpenExternalUrl(String),
    /// Play a named UI sound effect.
    PlaySound(&'static str),
    /// Dispatches a lifecycle signal to a managed app instance.
    DispatchLifecycle {
        /// Target window id.
        window_id: WindowId,
        /// Lifecycle event payload.
        event: AppLifecycleEvent,
    },
    /// Delivers a direct app event to a managed window inbox.
    DeliverAppEvent {
        /// Target window id.
        window_id: WindowId,
        /// Event payload.
        event: AppEvent,
    },
    /// Subscribes a window to an app-bus topic.
    SubscribeWindowTopic {
        /// Target window id.
        window_id: WindowId,
        /// Topic name.
        topic: String,
    },
    /// Unsubscribes a window from an app-bus topic.
    UnsubscribeWindowTopic {
        /// Target window id.
        window_id: WindowId,
        /// Topic name.
        topic: String,
    },
    /// Publishes an app-bus event from the source window.
    PublishTopicEvent {
        /// Source window id.
        source_window_id: WindowId,
        /// Topic name.
        topic: String,
        /// Event payload.
        payload: Value,
        /// Optional correlation id.
        correlation_id: Option<String>,
        /// Optional reply topic.
        reply_to: Option<String>,
    },
    /// Persist a namespaced config key/value through host prefs.
    SaveConfig {
        /// Config namespace.
        namespace: String,
        /// Config key.
        key: String,
        /// Config payload.
        value: Value,
    },
    /// Emit a host notification request.
    Notify {
        /// Notification title.
        title: String,
        /// Notification body.
        body: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Controls whether snapshot hydration is restoring boot state or synchronizing a live session.
pub enum HydrationMode {
    /// Restores a boot snapshot into an empty/new runtime session.
    BootRestore,
    /// Synchronizes persisted state into an already-running session.
    SyncRefresh,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// State domains that participate in monotonic cross-context synchronization.
pub enum SyncDomain {
    /// Desktop layout snapshot state.
    Layout,
    /// Theme and accessibility preference state.
    Theme,
}

#[derive(Debug, Error, Clone, PartialEq)]
/// Reducer errors for invalid actions (for example, referencing a missing window).
pub enum ReducerError {
    /// The target window id was not found in the current state.
    #[error("window not found")]
    WindowNotFound,
    /// The requested transition is blocked by an active modal window.
    #[error("active modal window {active_modal:?} blocks this transition")]
    ModalBlocked {
        /// The modal window that must be resolved first.
        active_modal: WindowId,
    },
    /// The app attempted to use a capability that is not granted.
    #[error("app `{app_id}` is not authorized for capability `{capability:?}`")]
    CapabilityDenied {
        /// Canonical application identifier.
        app_id: String,
        /// The missing capability.
        capability: AppCapability,
        /// Source window receiving the denial diagnostic.
        window_id: WindowId,
        /// App-visible denial diagnostic envelope.
        diagnostic_event: Box<AppEvent>,
    },
}

fn clamp_window_rect_to_viewport(rect: WindowRect, viewport: WindowRect) -> WindowRect {
    let min_w = MIN_WINDOW_WIDTH.min(viewport.w.max(MIN_WINDOW_WIDTH));
    let min_h = MIN_WINDOW_HEIGHT.min(viewport.h.max(MIN_WINDOW_HEIGHT));
    let max_w = (viewport.w - 20).max(min_w);
    let max_h = (viewport.h - 20).max(min_h);
    let w = rect.w.clamp(min_w, max_w);
    let h = rect.h.clamp(min_h, max_h);
    let max_x = (viewport.x + viewport.w - w - 10).max(viewport.x + 10);
    let max_y = (viewport.y + viewport.h - h - 10).max(viewport.y + 10);
    let x = rect.x.clamp(viewport.x + 10, max_x);
    let y = rect.y.clamp(viewport.y + 10, max_y);
    WindowRect { x, y, w, h }
}

fn close_shell_panels(state: &mut DesktopState) {
    state.panels.launcher_open = false;
    state.panels.control_center_open = false;
    state.panels.notification_center_open = false;
}

fn retain_notification(
    state: &mut DesktopState,
    source_app_id: Option<String>,
    title: String,
    body: String,
) {
    let notification = DesktopNotification {
        id: state.next_notification_id,
        title,
        body,
        source_app_id,
        unread: true,
    };
    state.next_notification_id = state.next_notification_id.saturating_add(1);
    state.notifications.insert(0, notification);
    if state.notifications.len() > 24 {
        state.notifications.truncate(24);
    }
}

/// Applies a [`DesktopAction`] to the desktop runtime state and collects resulting side effects.
///
/// This function is the authoritative state transition engine for desktop window management and
/// shell-level preferences.
///
/// # Errors
///
/// Returns [`ReducerError::WindowNotFound`] when an action references a window that is not present.
pub fn reduce_desktop(
    state: &mut DesktopState,
    interaction: &mut InteractionState,
    action: DesktopAction,
) -> Result<Vec<RuntimeEffect>, ReducerError> {
    let mut effects = Vec::new();
    if appearance::reduce_appearance_action(state, &action, &mut effects)? {
        return Ok(effects);
    }
    match action {
        DesktopAction::ActivateApp { app_id, viewport } => {
            if app_id == ApplicationId::trusted("system.control-center") {
                close_shell_panels(state);
                state.panels.control_center_open = true;
                return Ok(effects);
            }
            let descriptor = apps::app_descriptor_by_id(&app_id);

            if descriptor.single_instance
                && let Some(window_id) = preferred_window_for_app(state, &app_id)
            {
                let nested = if state
                    .windows
                    .iter()
                    .find(|w| w.id == window_id)
                    .map(|w| w.minimized)
                    .unwrap_or(false)
                {
                    reduce_desktop(
                        state,
                        interaction,
                        DesktopAction::RestoreWindow { window_id },
                    )?
                } else if state.focused_window_id() != Some(window_id) {
                    reduce_desktop(state, interaction, DesktopAction::FocusWindow { window_id })?
                } else {
                    Vec::new()
                };
                effects.extend(nested);
                return Ok(effects);
            }

            let nested = reduce_desktop(
                state,
                interaction,
                DesktopAction::OpenWindow(
                    apps::default_open_request_by_id(&app_id, viewport).expect("built-in app id"),
                ),
            )?;
            effects.extend(nested);
            return Ok(effects);
        }
        DesktopAction::OpenWindow(req) => {
            let previously_focused = state.focused_window_id();
            let window_id = next_window_id(state);
            begin_modal_open(state, window_id, &req)?;
            let default_offset = ((window_id.0 as i32) - 1) % 8 * 20;
            let viewport = req.viewport.unwrap_or(WindowRect {
                x: 0,
                y: 0,
                w: 1280,
                h: 760,
            });
            let rect = req
                .rect
                .unwrap_or(WindowRect {
                    x: 40 + default_offset,
                    y: 48 + default_offset,
                    w: DEFAULT_WINDOW_WIDTH,
                    h: DEFAULT_WINDOW_HEIGHT,
                })
                .offset(default_offset / 2, default_offset / 2)
                .clamped_min(MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT);
            let rect = clamp_window_rect_to_viewport(rect, viewport);
            let record = WindowRecord {
                id: window_id,
                app_id: req.app_id.clone(),
                title: req
                    .title
                    .unwrap_or_else(|| apps::app_title_by_id(&req.app_id).to_string()),
                icon_id: req
                    .icon_id
                    .unwrap_or_else(|| apps::app_icon_id_by_id(&req.app_id).to_string()),
                rect,
                restore_rect: None,
                z_index: 0,
                is_focused: false,
                minimized: false,
                maximized: false,
                suspended: false,
                flags: req.flags,
                persist_key: req.persist_key,
                app_state: req.app_state,
                launch_params: req.launch_params,
                last_lifecycle_event: None,
            };
            state.windows.push(record);
            if !focus_window_internal(state, window_id) {
                return Err(ReducerError::WindowNotFound);
            }
            close_shell_panels(state);
            record_window_lifecycle(state, window_id, AppLifecycleEvent::Mounted);
            effects.push(RuntimeEffect::DispatchLifecycle {
                window_id,
                event: AppLifecycleEvent::Mounted,
            });
            emit_focus_transition(previously_focused, Some(window_id), state, &mut effects);
            effects.push(RuntimeEffect::PersistLayout);
            effects.push(RuntimeEffect::FocusWindowInput(window_id));
        }
        DesktopAction::CloseWindow { window_id } => {
            ensure_parent_close_allowed(state, window_id)?;
            let was_focused = state.focused_window_id() == Some(window_id);
            let modal_parent_to_focus = complete_modal_close(state, window_id);
            clear_interaction_for_window(interaction, window_id);
            effects.push(RuntimeEffect::DispatchLifecycle {
                window_id,
                event: AppLifecycleEvent::Closing,
            });
            let before_len = state.windows.len();
            state.windows.retain(|w| w.id != window_id);
            if state.windows.len() == before_len {
                return Err(ReducerError::WindowNotFound);
            }
            normalize_window_stack(state);
            effects.push(RuntimeEffect::DispatchLifecycle {
                window_id,
                event: AppLifecycleEvent::Closed,
            });
            if let Some(parent_id) = modal_parent_to_focus {
                if focus_window_internal(state, parent_id) {
                    emit_focus_transition(Some(window_id), Some(parent_id), state, &mut effects);
                    effects.push(RuntimeEffect::FocusWindowInput(parent_id));
                }
            } else if was_focused {
                let new_focus = state.focused_window_id();
                emit_focus_transition(Some(window_id), new_focus, state, &mut effects);
            }
            effects.push(RuntimeEffect::PersistLayout);
        }
        DesktopAction::FocusWindow { window_id } => {
            ensure_modal_allows_target(state, Some(window_id))?;
            let previous_focus = state.focused_window_id();
            if !focus_window_internal(state, window_id) {
                return Err(ReducerError::WindowNotFound);
            }
            close_shell_panels(state);
            emit_focus_transition(previous_focus, Some(window_id), state, &mut effects);
            effects.push(RuntimeEffect::FocusWindowInput(window_id));
        }
        DesktopAction::MinimizeWindow { window_id } => {
            ensure_modal_allows_target(state, Some(window_id))?;
            let previous_focus = state.focused_window_id();
            let should_suspend = {
                let window = find_window_mut(state, window_id)?;
                window.minimized = true;
                window.is_focused = false;
                let should_suspend = matches!(
                    apps::app_suspend_policy_by_id(&window.app_id),
                    desktop_app_contract::SuspendPolicy::OnMinimize
                ) && !window.suspended;
                if should_suspend {
                    window.suspended = true;
                }
                should_suspend
            };
            record_window_lifecycle(state, window_id, AppLifecycleEvent::Minimized);
            effects.push(RuntimeEffect::DispatchLifecycle {
                window_id,
                event: AppLifecycleEvent::Minimized,
            });
            if should_suspend {
                record_window_lifecycle(state, window_id, AppLifecycleEvent::Suspended);
                effects.push(RuntimeEffect::DispatchLifecycle {
                    window_id,
                    event: AppLifecycleEvent::Suspended,
                });
            }
            normalize_window_stack(state);
            if previous_focus == Some(window_id) {
                let next_focus = state.focused_window_id();
                emit_focus_transition(Some(window_id), next_focus, state, &mut effects);
            }
            effects.push(RuntimeEffect::PersistLayout);
        }
        DesktopAction::MaximizeWindow {
            window_id,
            viewport,
        } => {
            ensure_modal_allows_target(state, Some(window_id))?;
            let previous_focus = state.focused_window_id();
            let was_suspended = {
                let window = find_window_mut(state, window_id)?;
                if !window.maximized {
                    window.restore_rect = Some(window.rect);
                }
                window.rect = viewport.clamped_min(MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT);
                window.maximized = true;
                window.minimized = false;
                let was_suspended = window.suspended;
                window.suspended = false;
                was_suspended
            };
            if !focus_window_internal(state, window_id) {
                return Err(ReducerError::WindowNotFound);
            }
            if was_suspended {
                record_window_lifecycle(state, window_id, AppLifecycleEvent::Resumed);
                effects.push(RuntimeEffect::DispatchLifecycle {
                    window_id,
                    event: AppLifecycleEvent::Resumed,
                });
            }
            emit_focus_transition(previous_focus, Some(window_id), state, &mut effects);
            effects.push(RuntimeEffect::PersistLayout);
        }
        DesktopAction::RestoreWindow { window_id } => {
            ensure_modal_allows_target(state, Some(window_id))?;
            let previous_focus = state.focused_window_id();
            let was_suspended = {
                let window = find_window_mut(state, window_id)?;
                if window.maximized {
                    if let Some(restore_rect) = window.restore_rect {
                        window.rect = restore_rect;
                    }
                    window.maximized = false;
                }
                window.minimized = false;
                let was_suspended = window.suspended;
                window.suspended = false;
                was_suspended
            };
            if !focus_window_internal(state, window_id) {
                return Err(ReducerError::WindowNotFound);
            }
            record_window_lifecycle(state, window_id, AppLifecycleEvent::Restored);
            effects.push(RuntimeEffect::DispatchLifecycle {
                window_id,
                event: AppLifecycleEvent::Restored,
            });
            if was_suspended {
                record_window_lifecycle(state, window_id, AppLifecycleEvent::Resumed);
                effects.push(RuntimeEffect::DispatchLifecycle {
                    window_id,
                    event: AppLifecycleEvent::Resumed,
                });
            }
            emit_focus_transition(previous_focus, Some(window_id), state, &mut effects);
            effects.push(RuntimeEffect::PersistLayout);
            effects.push(RuntimeEffect::FocusWindowInput(window_id));
        }
        DesktopAction::ToggleTaskbarWindow { window_id } => {
            ensure_modal_allows_target(state, Some(window_id))?;
            let focused = state.focused_window_id() == Some(window_id);
            let minimized = state
                .windows
                .iter()
                .find(|w| w.id == window_id)
                .map(|w| w.minimized)
                .ok_or(ReducerError::WindowNotFound)?;
            if minimized {
                let nested = reduce_desktop(
                    state,
                    interaction,
                    DesktopAction::RestoreWindow { window_id },
                )?;
                effects.extend(nested);
            } else if focused {
                let nested = reduce_desktop(
                    state,
                    interaction,
                    DesktopAction::MinimizeWindow { window_id },
                )?;
                effects.extend(nested);
            } else {
                let nested =
                    reduce_desktop(state, interaction, DesktopAction::FocusWindow { window_id })?;
                effects.extend(nested);
            }
        }
        DesktopAction::ToggleStartMenu => {
            let will_open = !state.panels.launcher_open;
            close_shell_panels(state);
            state.panels.launcher_open = will_open;
        }
        DesktopAction::CloseStartMenu => {
            state.panels.launcher_open = false;
        }
        DesktopAction::ToggleControlCenter => {
            let will_open = !state.panels.control_center_open;
            close_shell_panels(state);
            state.panels.control_center_open = will_open;
        }
        DesktopAction::CloseControlCenter => {
            state.panels.control_center_open = false;
        }
        DesktopAction::ToggleNotificationCenter => {
            let will_open = !state.panels.notification_center_open;
            close_shell_panels(state);
            state.panels.notification_center_open = will_open;
            if will_open {
                for notification in &mut state.notifications {
                    notification.unread = false;
                }
            }
        }
        DesktopAction::CloseNotificationCenter => {
            state.panels.notification_center_open = false;
        }
        DesktopAction::BeginMove { window_id, pointer } => {
            ensure_modal_allows_target(state, Some(window_id))?;
            let previous_focus = state.focused_window_id();
            let rect_start = find_window_mut(state, window_id)?.rect;
            if !focus_window_internal(state, window_id) {
                return Err(ReducerError::WindowNotFound);
            }
            emit_focus_transition(previous_focus, Some(window_id), state, &mut effects);
            interaction.dragging = Some(crate::model::DragSession {
                window_id,
                pointer_start: pointer,
                rect_start,
            });
        }
        DesktopAction::UpdateMove { pointer } => {
            if let Some(session) = interaction.dragging.as_ref() {
                let dx = pointer.x - session.pointer_start.x;
                let dy = pointer.y - session.pointer_start.y;
                let window = find_window_mut(state, session.window_id)?;
                if !window.maximized {
                    window.rect = session.rect_start.offset(dx, dy);
                }
            }
        }
        DesktopAction::EndMove => {
            interaction.dragging = None;
            effects.push(RuntimeEffect::PersistLayout);
        }
        DesktopAction::EndMoveWithViewport { viewport } => {
            let dragged_window_id = interaction
                .dragging
                .as_ref()
                .map(|session| session.window_id);
            interaction.dragging = None;

            if let Some(window_id) = dragged_window_id {
                snap_window_to_viewport_edge(state, window_id, viewport);
            }

            effects.push(RuntimeEffect::PersistLayout);
        }
        DesktopAction::BeginResize {
            window_id,
            edge,
            pointer,
            viewport,
        } => {
            ensure_modal_allows_target(state, Some(window_id))?;
            let previous_focus = state.focused_window_id();
            let rect_start = find_window_mut(state, window_id)?.rect;
            if !focus_window_internal(state, window_id) {
                return Err(ReducerError::WindowNotFound);
            }
            emit_focus_transition(previous_focus, Some(window_id), state, &mut effects);
            interaction.resizing = Some(ResizeSession {
                window_id,
                edge,
                pointer_start: pointer,
                rect_start,
                viewport,
            });
        }
        DesktopAction::UpdateResize { pointer } => {
            if let Some(session) = interaction.resizing.as_ref() {
                let dx = pointer.x - session.pointer_start.x;
                let dy = pointer.y - session.pointer_start.y;
                let window = find_window_mut(state, session.window_id)?;
                if !window.maximized && window.flags.resizable {
                    let resized = resize_rect(session.rect_start, session.edge, dx, dy)
                        .clamped_min(MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT);
                    window.rect = clamp_window_rect_to_viewport(resized, session.viewport);
                }
            }
        }
        DesktopAction::EndResize => {
            interaction.resizing = None;
            effects.push(RuntimeEffect::PersistLayout);
        }
        DesktopAction::SuspendWindow { window_id } => {
            ensure_modal_allows_target(state, Some(window_id))?;
            let should_emit = {
                let window = find_window_mut(state, window_id)?;
                if window.suspended {
                    false
                } else {
                    window.suspended = true;
                    true
                }
            };
            if should_emit {
                record_window_lifecycle(state, window_id, AppLifecycleEvent::Suspended);
                effects.push(RuntimeEffect::DispatchLifecycle {
                    window_id,
                    event: AppLifecycleEvent::Suspended,
                });
                effects.push(RuntimeEffect::PersistLayout);
            }
        }
        DesktopAction::ResumeWindow { window_id } => {
            ensure_modal_allows_target(state, Some(window_id))?;
            let should_emit = {
                let window = find_window_mut(state, window_id)?;
                if window.suspended {
                    window.suspended = false;
                    true
                } else {
                    false
                }
            };
            if should_emit {
                record_window_lifecycle(state, window_id, AppLifecycleEvent::Resumed);
                effects.push(RuntimeEffect::DispatchLifecycle {
                    window_id,
                    event: AppLifecycleEvent::Resumed,
                });
                effects.push(RuntimeEffect::PersistLayout);
            }
        }
        DesktopAction::HandleAppCommand { window_id, command } => {
            let source_app_id = state
                .windows
                .iter()
                .find(|w| w.id == window_id)
                .map(|w| w.app_id.clone())
                .ok_or(ReducerError::WindowNotFound)?;
            if let Some(required) = command_required_capability(&command)
                && !command_allowed_for_app(state, &source_app_id, required)
            {
                return Err(ReducerError::CapabilityDenied {
                    app_id: source_app_id.to_string(),
                    capability: required,
                    window_id,
                    diagnostic_event: Box::new(AppEvent::capability_denied(
                        source_app_id.to_string(),
                        required,
                        window_id.0,
                        command_label(&command),
                    )),
                });
            }

            match command {
                AppCommand::SetWindowTitle { title } => {
                    let window = find_window_mut(state, window_id)?;
                    if window.title != title {
                        window.title = title;
                        effects.push(RuntimeEffect::PersistLayout);
                    }
                }
                AppCommand::PersistState { state: app_state } => {
                    let nested = reduce_desktop(
                        state,
                        interaction,
                        DesktopAction::SetAppState {
                            window_id,
                            app_state,
                        },
                    )?;
                    effects.extend(nested);
                }
                AppCommand::PersistSharedState { key, state: shared } => {
                    let app_id = state
                        .windows
                        .iter()
                        .find(|w| w.id == window_id)
                        .map(|w| w.app_id.clone())
                        .ok_or(ReducerError::WindowNotFound)?;
                    let nested = reduce_desktop(
                        state,
                        interaction,
                        DesktopAction::SetSharedAppState {
                            app_id,
                            key,
                            state: shared,
                        },
                    )?;
                    effects.extend(nested);
                }
                AppCommand::SaveConfig {
                    namespace,
                    key,
                    value,
                } => {
                    if !namespace.trim().is_empty() && !key.trim().is_empty() {
                        effects.push(RuntimeEffect::SaveConfig {
                            namespace,
                            key,
                            value,
                        });
                    }
                }
                AppCommand::OpenExternalUrl { url } => {
                    effects.push(RuntimeEffect::OpenExternalUrl(url));
                }
                AppCommand::Subscribe { topic } => {
                    if !topic.trim().is_empty() {
                        effects.push(RuntimeEffect::SubscribeWindowTopic { window_id, topic });
                    }
                }
                AppCommand::Unsubscribe { topic } => {
                    if !topic.trim().is_empty() {
                        effects.push(RuntimeEffect::UnsubscribeWindowTopic { window_id, topic });
                    }
                }
                AppCommand::PublishEvent {
                    topic,
                    payload,
                    correlation_id,
                    reply_to,
                } => {
                    if !topic.trim().is_empty() {
                        effects.push(RuntimeEffect::PublishTopicEvent {
                            source_window_id: window_id,
                            topic,
                            payload,
                            correlation_id,
                            reply_to,
                        });
                    }
                }
                AppCommand::SetDesktopHighContrast { enabled } => {
                    let nested = reduce_desktop(
                        state,
                        interaction,
                        DesktopAction::SetHighContrast { enabled },
                    )?;
                    effects.extend(nested);
                }
                AppCommand::SetDesktopReducedMotion { enabled } => {
                    let nested = reduce_desktop(
                        state,
                        interaction,
                        DesktopAction::SetReducedMotion { enabled },
                    )?;
                    effects.extend(nested);
                }
                AppCommand::SetDesktopDarkMode { enabled } => {
                    let nested = reduce_desktop(
                        state,
                        interaction,
                        DesktopAction::SetThemeMode {
                            mode: if enabled {
                                ThemeMode::Dark
                            } else {
                                ThemeMode::Light
                            },
                        },
                    )?;
                    effects.extend(nested);
                }
                AppCommand::Notify { title, body } => {
                    retain_notification(
                        state,
                        Some(source_app_id.to_string()),
                        title.clone(),
                        body.clone(),
                    );
                    effects.push(RuntimeEffect::Notify { title, body });
                }
            }
        }
        DesktopAction::PushTerminalHistory { command } => {
            if state.preferences.terminal_history_enabled && !command.trim().is_empty() {
                state.terminal_history.push(command);
                if state.terminal_history.len() > 100 {
                    let overflow = state.terminal_history.len() - 100;
                    state.terminal_history.drain(0..overflow);
                }
                effects.push(RuntimeEffect::PersistTerminalHistory);
            }
        }
        DesktopAction::SetAppState {
            window_id,
            app_state,
        } => {
            let window = find_window_mut(state, window_id)?;
            window.app_state = app_state;
            effects.push(RuntimeEffect::PersistLayout);
        }
        DesktopAction::SetSharedAppState {
            app_id,
            key,
            state: shared,
        } => {
            let storage_key = format!("{}:{}", app_id.as_str(), key.trim());
            state.app_shared_state.insert(storage_key, shared);
            effects.push(RuntimeEffect::PersistLayout);
        }
        DesktopAction::DismissNotification { id } => {
            state
                .notifications
                .retain(|notification| notification.id != id);
        }
        DesktopAction::MarkNotificationsRead => {
            for notification in &mut state.notifications {
                notification.unread = false;
            }
        }
        DesktopAction::ClearNotifications => {
            state.notifications.clear();
        }
        DesktopAction::CompleteBootHydration {
            snapshot,
            snapshot_revision,
            theme,
            privileged_app_ids,
            deep_link,
        } => {
            if let Some(snapshot) = snapshot {
                restore_snapshot(
                    state,
                    interaction,
                    snapshot,
                    HydrationMode::BootRestore,
                    snapshot_revision,
                    &mut effects,
                );
            } else {
                *interaction = InteractionState::default();
                state.layout_revision = snapshot_revision;
            }
            if let Some(theme) = theme {
                state.theme = theme;
            }
            state.privileged_app_ids = privileged_app_ids.into_iter().collect();
            if let Some(deep_link) = deep_link {
                apply_deep_link_targets(state, interaction, deep_link, &mut effects)?;
            }
            state.boot_hydrated = true;
        }
        DesktopAction::HydrateSnapshot {
            snapshot,
            mode,
            revision,
        } => {
            if revision.is_some_and(|incoming| {
                state
                    .layout_revision
                    .is_some_and(|current| incoming <= current)
            }) {
                return Ok(effects);
            }
            restore_snapshot(state, interaction, snapshot, mode, revision, &mut effects);
        }
        DesktopAction::ApplyDeepLink { deep_link } => {
            apply_deep_link_targets(state, interaction, deep_link, &mut effects)?;
        }
        DesktopAction::RecordAppliedRevision { domain, revision } => match domain {
            SyncDomain::Layout => state.layout_revision = Some(revision),
            SyncDomain::Theme => state.theme_revision = Some(revision),
        },
        DesktopAction::HydrateTheme { .. }
        | DesktopAction::SetHighContrast { .. }
        | DesktopAction::SetThemeMode { .. }
        | DesktopAction::SetReducedMotion { .. } => {
            unreachable!("appearance actions are handled by reducer::appearance")
        }
    }

    normalize_window_stack(state);
    Ok(effects)
}

/// Converts a parsed deep-link target into an [`OpenWindowRequest`].
pub fn build_open_request_from_deeplink(target: DeepLinkOpenTarget) -> OpenWindowRequest {
    match target {
        DeepLinkOpenTarget::App(app_id) => OpenWindowRequest::new(app_id),
        DeepLinkOpenTarget::NotesSlug(slug) => {
            // Notes deep links remain a compatibility route into the Settings experience.
            let mut req = OpenWindowRequest::new(apps::settings_application_id());
            req.title = Some(format!("Settings - Notes {slug}"));
            req.persist_key = Some(format!("notes:{slug}"));
            req.launch_params = json!({ "section": "personalize", "note_slug": slug });
            req
        }
        DeepLinkOpenTarget::ProjectSlug(slug) => {
            // Project deep links remain a compatibility route into Control Center.
            let mut req = OpenWindowRequest::new(ApplicationId::trusted("system.control-center"));
            req.title = Some(format!("Project - {slug}"));
            req.persist_key = Some(format!("projects:{slug}"));
            req.launch_params = json!({ "section": "overview", "project_slug": slug });
            req
        }
    }
}

fn restore_snapshot(
    state: &mut DesktopState,
    interaction: &mut InteractionState,
    snapshot: DesktopSnapshot,
    mode: HydrationMode,
    revision: Option<u64>,
    effects: &mut Vec<RuntimeEffect>,
) {
    let theme = state.theme.clone();
    let privileged_app_ids = state.privileged_app_ids.clone();
    let theme_revision = state.theme_revision;
    *state = DesktopState::from_snapshot(snapshot);
    *interaction = InteractionState::default();
    state.theme = theme;
    state.privileged_app_ids = privileged_app_ids;
    state.theme_revision = theme_revision;
    state.layout_revision = revision;
    let max_restore = state.preferences.max_restore_windows;
    if state.windows.len() > max_restore {
        state.windows.truncate(max_restore);
    }
    normalize_window_stack(state);
    if matches!(mode, HydrationMode::BootRestore) {
        for window in state.windows.iter_mut() {
            record_window_lifecycle_by_id(window, AppLifecycleEvent::Mounted);
            effects.push(RuntimeEffect::DispatchLifecycle {
                window_id: window.id,
                event: AppLifecycleEvent::Mounted,
            });
        }
        if let Some(focused) = state.focused_window_id() {
            record_window_lifecycle(state, focused, AppLifecycleEvent::Focused);
            effects.push(RuntimeEffect::DispatchLifecycle {
                window_id: focused,
                event: AppLifecycleEvent::Focused,
            });
        }
    }
}

fn apply_deep_link_targets(
    state: &mut DesktopState,
    interaction: &mut InteractionState,
    deep_link: DeepLinkState,
    effects: &mut Vec<RuntimeEffect>,
) -> Result<(), ReducerError> {
    for target in deep_link.open {
        if deep_link_target_satisfied(state, &target) {
            continue;
        }

        let nested = match target {
            DeepLinkOpenTarget::App(app_id) => reduce_desktop(
                state,
                interaction,
                DesktopAction::ActivateApp {
                    app_id,
                    viewport: None,
                },
            )?,
            other => reduce_desktop(
                state,
                interaction,
                DesktopAction::OpenWindow(build_open_request_from_deeplink(other)),
            )?,
        };
        effects.extend(nested);
    }

    Ok(())
}

fn deep_link_target_satisfied(state: &DesktopState, target: &DeepLinkOpenTarget) -> bool {
    match target {
        DeepLinkOpenTarget::App(app_id) => {
            state.windows.iter().any(|window| window.app_id == *app_id)
        }
        DeepLinkOpenTarget::NotesSlug(slug) => {
            let persist_key = format!("notes:{slug}");
            state
                .windows
                .iter()
                .any(|window| window.persist_key.as_deref() == Some(persist_key.as_str()))
        }
        DeepLinkOpenTarget::ProjectSlug(slug) => {
            let persist_key = format!("projects:{slug}");
            state
                .windows
                .iter()
                .any(|window| window.persist_key.as_deref() == Some(persist_key.as_str()))
        }
    }
}

fn next_window_id(state: &mut DesktopState) -> WindowId {
    let id = WindowId(state.next_window_id);
    state.next_window_id = state.next_window_id.saturating_add(1);
    id
}

fn clear_interaction_for_window(interaction: &mut InteractionState, window_id: WindowId) {
    if interaction
        .dragging
        .as_ref()
        .is_some_and(|session| session.window_id == window_id)
    {
        interaction.dragging = None;
    }
    if interaction
        .resizing
        .as_ref()
        .is_some_and(|session| session.window_id == window_id)
    {
        interaction.resizing = None;
    }
}

fn preferred_window_for_app(state: &DesktopState, app_id: &ApplicationId) -> Option<WindowId> {
    state
        .windows
        .iter()
        .rev()
        .find(|win| win.app_id == *app_id && !win.minimized && win.is_focused)
        .or_else(|| {
            state
                .windows
                .iter()
                .rev()
                .find(|win| win.app_id == *app_id && !win.minimized)
        })
        .or_else(|| state.windows.iter().rev().find(|win| win.app_id == *app_id))
        .map(|win| win.id)
}

fn find_window_mut(
    state: &mut DesktopState,
    window_id: WindowId,
) -> Result<&mut WindowRecord, ReducerError> {
    state
        .windows
        .iter_mut()
        .find(|w| w.id == window_id)
        .ok_or(ReducerError::WindowNotFound)
}

fn record_window_lifecycle(
    state: &mut DesktopState,
    window_id: WindowId,
    event: AppLifecycleEvent,
) {
    if let Some(window) = state.windows.iter_mut().find(|w| w.id == window_id) {
        record_window_lifecycle_by_id(window, event);
    }
}

fn record_window_lifecycle_by_id(window: &mut WindowRecord, event: AppLifecycleEvent) {
    window.last_lifecycle_event = Some(event.token().to_string());
}

fn emit_focus_transition(
    previous_focus: Option<WindowId>,
    next_focus: Option<WindowId>,
    state: &mut DesktopState,
    effects: &mut Vec<RuntimeEffect>,
) {
    if previous_focus == next_focus {
        return;
    }

    if let Some(previous) = previous_focus
        && state.windows.iter().any(|window| window.id == previous)
    {
        record_window_lifecycle(state, previous, AppLifecycleEvent::Blurred);
        effects.push(RuntimeEffect::DispatchLifecycle {
            window_id: previous,
            event: AppLifecycleEvent::Blurred,
        });
    }

    if let Some(next) = next_focus
        && state.windows.iter().any(|window| window.id == next)
    {
        record_window_lifecycle(state, next, AppLifecycleEvent::Focused);
        effects.push(RuntimeEffect::DispatchLifecycle {
            window_id: next,
            event: AppLifecycleEvent::Focused,
        });
    }
}

fn command_required_capability(command: &AppCommand) -> Option<AppCapability> {
    match command {
        AppCommand::SetWindowTitle { .. } => Some(AppCapability::Window),
        AppCommand::PersistState { .. } | AppCommand::PersistSharedState { .. } => {
            Some(AppCapability::State)
        }
        AppCommand::SaveConfig { .. } => Some(AppCapability::Config),
        AppCommand::OpenExternalUrl { .. } => Some(AppCapability::ExternalUrl),
        AppCommand::Subscribe { .. }
        | AppCommand::Unsubscribe { .. }
        | AppCommand::PublishEvent { .. } => Some(AppCapability::Ipc),
        AppCommand::SetDesktopHighContrast { .. }
        | AppCommand::SetDesktopReducedMotion { .. }
        | AppCommand::SetDesktopDarkMode { .. } => Some(AppCapability::Theme),
        AppCommand::Notify { .. } => Some(AppCapability::Notifications),
    }
}

fn command_allowed_for_app(
    state: &DesktopState,
    app_id: &ApplicationId,
    required: AppCapability,
) -> bool {
    apps::resolved_capabilities(state, app_id).contains(&required)
}

fn active_modal_window(state: &DesktopState) -> Option<&WindowRecord> {
    let active_modal = state.active_modal?;
    state
        .windows
        .iter()
        .find(|window| window.id == active_modal)
}

fn begin_modal_open(
    state: &mut DesktopState,
    incoming_window_id: WindowId,
    req: &OpenWindowRequest,
) -> Result<(), ReducerError> {
    let modal_parent = req.flags.modal_parent;
    if let Some(active_modal) = active_modal_window(state) {
        return Err(ReducerError::ModalBlocked {
            active_modal: active_modal.id,
        });
    }

    if let Some(parent_id) = modal_parent {
        if !state.windows.iter().any(|window| window.id == parent_id) {
            return Err(ReducerError::WindowNotFound);
        }
        state.active_modal = Some(incoming_window_id);
    }

    Ok(())
}

fn ensure_modal_allows_target(
    state: &DesktopState,
    target_window_id: Option<WindowId>,
) -> Result<(), ReducerError> {
    let Some(active_modal) = active_modal_window(state) else {
        return Ok(());
    };
    if Some(active_modal.id) == target_window_id {
        return Ok(());
    }

    Err(ReducerError::ModalBlocked {
        active_modal: active_modal.id,
    })
}

fn complete_modal_close(state: &mut DesktopState, closed_window_id: WindowId) -> Option<WindowId> {
    let active_modal = active_modal_window(state)?;
    if active_modal.id != closed_window_id {
        return None;
    }

    let parent_id = active_modal.flags.modal_parent;
    state.active_modal = None;
    parent_id.filter(|parent_id| state.windows.iter().any(|window| window.id == *parent_id))
}

fn ensure_parent_close_allowed(
    state: &DesktopState,
    parent_window_id: WindowId,
) -> Result<(), ReducerError> {
    let Some(active_modal) = active_modal_window(state) else {
        return Ok(());
    };
    if active_modal.flags.modal_parent == Some(parent_window_id) {
        return Err(ReducerError::ModalBlocked {
            active_modal: active_modal.id,
        });
    }

    Ok(())
}

fn command_label(command: &AppCommand) -> &'static str {
    match command {
        AppCommand::SetWindowTitle { .. } => "set-window-title",
        AppCommand::PersistState { .. } => "persist-state",
        AppCommand::PersistSharedState { .. } => "persist-shared-state",
        AppCommand::SaveConfig { .. } => "save-config",
        AppCommand::OpenExternalUrl { .. } => "open-external-url",
        AppCommand::Subscribe { .. } => "subscribe",
        AppCommand::Unsubscribe { .. } => "unsubscribe",
        AppCommand::PublishEvent { .. } => "publish-event",
        AppCommand::SetDesktopHighContrast { .. } => "set-desktop-high-contrast",
        AppCommand::SetDesktopReducedMotion { .. } => "set-desktop-reduced-motion",
        AppCommand::SetDesktopDarkMode { .. } => "set-desktop-dark-mode",
        AppCommand::Notify { .. } => "notify",
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::model::{InteractionState, OpenWindowRequest, WindowFlags};
    use desktop_app_contract::ApplicationId;

    fn open(
        state: &mut DesktopState,
        interaction: &mut InteractionState,
        app_id: impl Into<ApplicationId>,
    ) -> WindowId {
        let _ = reduce_desktop(
            state,
            interaction,
            DesktopAction::OpenWindow(OpenWindowRequest::new(app_id)),
        )
        .expect("open window");
        state.windows.last().expect("window").id
    }

    #[test]
    fn open_window_focuses_new_window_and_updates_stack() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();

        let first = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.control-center"),
        );
        let second = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.settings"),
        );

        assert_eq!(state.focused_window_id(), Some(second));
        assert_eq!(state.windows.len(), 2);
        assert_eq!(state.windows[0].id, first);
        assert_eq!(state.windows[1].id, second);
        assert_eq!(state.windows[1].z_index, 2);
    }

    #[test]
    fn taskbar_toggle_minimizes_if_focused_and_restores_if_minimized() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();

        let win = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.control-center"),
        );
        reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::ToggleTaskbarWindow { window_id: win },
        )
        .expect("minimize");

        let record = state.windows.iter().find(|w| w.id == win).unwrap();
        assert!(record.minimized);
        assert!(!record.is_focused);

        reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::ToggleTaskbarWindow { window_id: win },
        )
        .expect("restore");
        let record = state.windows.iter().find(|w| w.id == win).unwrap();
        assert!(!record.minimized);
        assert!(record.is_focused);
    }

    #[test]
    fn taskbar_toggle_restore_preserves_focus_effects() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();

        let win = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.control-center"),
        );
        reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::MinimizeWindow { window_id: win },
        )
        .expect("minimize");

        let effects = reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::ToggleTaskbarWindow { window_id: win },
        )
        .expect("restore");

        assert!(effects.contains(&RuntimeEffect::FocusWindowInput(win)));
        assert_eq!(
            effects
                .iter()
                .filter(|effect| matches!(effect, RuntimeEffect::PersistLayout))
                .count(),
            1
        );
    }

    #[test]
    fn activate_app_reuses_existing_single_instance_window() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();

        let first = reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::ActivateApp {
                app_id: ApplicationId::trusted("system.terminal"),
                viewport: None,
            },
        )
        .expect("activate terminal");
        assert_eq!(state.windows.len(), 1);
        let win_id = state.windows[0].id;
        assert!(first.contains(&RuntimeEffect::PersistLayout));

        let effects = reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::ActivateApp {
                app_id: ApplicationId::trusted("system.terminal"),
                viewport: None,
            },
        )
        .expect("reactivate terminal");

        assert_eq!(state.windows.len(), 1);
        assert_eq!(state.windows[0].id, win_id);
        assert!(effects.is_empty());
    }

    #[test]
    fn activate_app_reuses_existing_window_for_single_instance_apps() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();

        reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::ActivateApp {
                app_id: ApplicationId::trusted("system.control-center"),
                viewport: None,
            },
        )
        .expect("activate control center first");
        reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::ActivateApp {
                app_id: ApplicationId::trusted("system.control-center"),
                viewport: None,
            },
        )
        .expect("activate control center second");

        assert!(state.windows.is_empty());
        assert!(state.panels.control_center_open);
    }

    #[test]
    fn activate_settings_uses_default_open_request_without_theme_launch_params() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();
        state.theme.high_contrast = true;
        state.theme.reduced_motion = true;

        reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::ActivateApp {
                app_id: ApplicationId::trusted("system.settings"),
                viewport: None,
            },
        )
        .expect("activate settings");

        assert_eq!(state.windows[0].launch_params, Value::Null);
    }

    #[test]
    fn focusing_already_focused_top_window_is_noop_for_stack_order() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();

        let first = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.control-center"),
        );
        let second = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.settings"),
        );
        let before = state.windows.clone();

        let effects = reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::FocusWindow { window_id: second },
        )
        .expect("focus focused window");

        assert_eq!(state.windows, before);
        assert_eq!(state.focused_window_id(), Some(second));
        assert_ne!(state.focused_window_id(), Some(first));
        assert!(effects.contains(&RuntimeEffect::FocusWindowInput(second)));
    }

    #[test]
    fn moving_window_updates_rect_during_drag_and_persists_on_end() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();

        let win = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.terminal"),
        );
        let original = state.windows.iter().find(|w| w.id == win).unwrap().rect;

        reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::BeginMove {
                window_id: win,
                pointer: PointerPosition { x: 10, y: 10 },
            },
        )
        .unwrap();
        reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::UpdateMove {
                pointer: PointerPosition { x: 35, y: 50 },
            },
        )
        .unwrap();

        let moved = state.windows.iter().find(|w| w.id == win).unwrap().rect;
        assert_eq!(moved.x, original.x + 25);
        assert_eq!(moved.y, original.y + 40);
        let effects = reduce_desktop(&mut state, &mut interaction, DesktopAction::EndMove).unwrap();
        assert!(effects.contains(&RuntimeEffect::PersistLayout));
    }

    #[test]
    fn end_move_with_viewport_snaps_window_to_left_half() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();
        let viewport = WindowRect {
            x: 0,
            y: 0,
            w: 1000,
            h: 700,
        };

        let win = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.control-center"),
        );
        reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::BeginMove {
                window_id: win,
                pointer: PointerPosition { x: 0, y: 0 },
            },
        )
        .unwrap();
        reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::UpdateMove {
                pointer: PointerPosition { x: -35, y: 80 },
            },
        )
        .unwrap();

        reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::EndMoveWithViewport { viewport },
        )
        .unwrap();

        let record = state.windows.iter().find(|w| w.id == win).unwrap();
        assert_eq!(record.rect.x, 0);
        assert_eq!(record.rect.y, 0);
        assert_eq!(record.rect.w, 500);
        assert_eq!(record.rect.h, 700);
        assert!(!record.maximized);
    }

    #[test]
    fn end_move_with_viewport_snaps_window_to_top_maximize() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();
        let viewport = WindowRect {
            x: 0,
            y: 0,
            w: 1200,
            h: 760,
        };

        let win = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.terminal"),
        );
        reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::BeginMove {
                window_id: win,
                pointer: PointerPosition { x: 0, y: 0 },
            },
        )
        .unwrap();
        reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::UpdateMove {
                pointer: PointerPosition { x: 150, y: -40 },
            },
        )
        .unwrap();

        reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::EndMoveWithViewport { viewport },
        )
        .unwrap();

        let record = state.windows.iter().find(|w| w.id == win).unwrap();
        assert_eq!(record.rect, viewport);
        assert!(record.maximized);
        assert!(record.restore_rect.is_some());
    }

    #[test]
    fn set_high_contrast_updates_theme_and_persists() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();

        let effects = reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::SetHighContrast { enabled: true },
        )
        .expect("set high contrast");

        assert!(state.theme.high_contrast);
        assert_eq!(effects, vec![RuntimeEffect::PersistTheme]);
    }

    #[test]
    fn handle_app_command_persist_state_updates_window_record_and_persists() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();
        let window_id = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.control-center"),
        );
        let payload = serde_json::json!({ "cwd": "/Projects" });

        let effects = reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::HandleAppCommand {
                window_id,
                command: AppCommand::PersistState {
                    state: payload.clone(),
                },
            },
        )
        .expect("persist state command");

        let window = state.windows.iter().find(|w| w.id == window_id).unwrap();
        assert_eq!(window.app_state, payload);
        assert!(effects.contains(&RuntimeEffect::PersistLayout));
    }

    #[test]
    fn handle_app_command_set_window_title_updates_taskbar_and_chrome_title() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();
        let window_id = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.settings"),
        );

        let effects = reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::HandleAppCommand {
                window_id,
                command: AppCommand::SetWindowTitle {
                    title: "Notes - roadmap".to_string(),
                },
            },
        )
        .expect("set title command");

        let window = state.windows.iter().find(|w| w.id == window_id).unwrap();
        assert_eq!(window.title, "Notes - roadmap");
        assert!(effects.contains(&RuntimeEffect::PersistLayout));
    }

    #[test]
    fn minimize_applies_suspend_policy() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();
        let control_center = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.control-center"),
        );
        let terminal = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.terminal"),
        );

        let control_center_effects = reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::MinimizeWindow {
                window_id: control_center,
            },
        )
        .expect("minimize control center");
        let control_center_window = state
            .windows
            .iter()
            .find(|w| w.id == control_center)
            .unwrap();
        assert!(control_center_window.suspended);
        assert!(
            control_center_effects.contains(&RuntimeEffect::DispatchLifecycle {
                window_id: control_center,
                event: AppLifecycleEvent::Suspended,
            })
        );

        let terminal_effects = reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::MinimizeWindow {
                window_id: terminal,
            },
        )
        .expect("minimize terminal");
        let terminal_window = state.windows.iter().find(|w| w.id == terminal).unwrap();
        assert!(!terminal_window.suspended);
        assert!(
            !terminal_effects.contains(&RuntimeEffect::DispatchLifecycle {
                window_id: terminal,
                event: AppLifecycleEvent::Suspended,
            })
        );
    }

    #[test]
    fn close_flow_emits_closing_then_closed() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();
        let _first = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.control-center"),
        );
        let second = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.settings"),
        );

        let effects = reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::CloseWindow { window_id: second },
        )
        .expect("close focused window");

        let closing_idx = effects
            .iter()
            .position(|effect| {
                matches!(
                    effect,
                    RuntimeEffect::DispatchLifecycle {
                        window_id,
                        event: AppLifecycleEvent::Closing
                    } if *window_id == second
                )
            })
            .expect("closing lifecycle");
        let closed_idx = effects
            .iter()
            .position(|effect| {
                matches!(
                    effect,
                    RuntimeEffect::DispatchLifecycle {
                        window_id,
                        event: AppLifecycleEvent::Closed
                    } if *window_id == second
                )
            })
            .expect("closed lifecycle");

        assert!(closing_idx < closed_idx);
        assert!(!state.windows.iter().any(|w| w.id == second));
        assert!(state.focused_window_id().is_some());
    }

    #[test]
    fn app_bus_commands_emit_window_manager_effects() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();
        let window_id = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.control-center"),
        );
        let payload = serde_json::json!({ "path": "/Projects/demo" });

        let subscribe_effects = reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::HandleAppCommand {
                window_id,
                command: AppCommand::Subscribe {
                    topic: "explorer.refresh".to_string(),
                },
            },
        )
        .expect("subscribe command");
        assert_eq!(
            subscribe_effects,
            vec![RuntimeEffect::SubscribeWindowTopic {
                window_id,
                topic: "explorer.refresh".to_string(),
            }]
        );

        let publish_effects = reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::HandleAppCommand {
                window_id,
                command: AppCommand::PublishEvent {
                    topic: "explorer.refresh".to_string(),
                    payload: payload.clone(),
                    correlation_id: None,
                    reply_to: None,
                },
            },
        )
        .expect("publish command");
        assert_eq!(
            publish_effects,
            vec![RuntimeEffect::PublishTopicEvent {
                source_window_id: window_id,
                topic: "explorer.refresh".to_string(),
                payload,
                correlation_id: None,
                reply_to: None,
            }]
        );
    }

    #[test]
    fn hydrate_snapshot_uses_restored_restore_limit() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();
        let mut snapshot = DesktopState::default().snapshot();
        snapshot.preferences.max_restore_windows = 1;
        snapshot.windows = vec![
            WindowRecord {
                id: WindowId(1),
                app_id: ApplicationId::trusted("system.control-center"),
                title: "Control Center".to_string(),
                icon_id: "home".to_string(),
                rect: WindowRect::default(),
                restore_rect: None,
                z_index: 1,
                is_focused: true,
                minimized: false,
                maximized: false,
                suspended: false,
                flags: WindowFlags::default(),
                persist_key: None,
                app_state: Value::Null,
                launch_params: Value::Null,
                last_lifecycle_event: None,
            },
            WindowRecord {
                id: WindowId(2),
                app_id: ApplicationId::trusted("system.settings"),
                title: "Settings".to_string(),
                icon_id: "settings".to_string(),
                rect: WindowRect::default(),
                restore_rect: None,
                z_index: 2,
                is_focused: false,
                minimized: false,
                maximized: false,
                suspended: false,
                flags: WindowFlags::default(),
                persist_key: None,
                app_state: Value::Null,
                launch_params: Value::Null,
                last_lifecycle_event: None,
            },
        ];
        state.preferences.max_restore_windows = 5;

        reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::HydrateSnapshot {
                snapshot,
                mode: HydrationMode::BootRestore,
                revision: None,
            },
        )
        .expect("hydrate snapshot");

        assert_eq!(state.windows.len(), 1);
        assert_eq!(state.windows[0].id, WindowId(1));
    }

    #[test]
    fn sync_hydration_does_not_replay_mount_lifecycle_effects() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();
        let mut snapshot = DesktopState::default().snapshot();
        snapshot.windows = vec![WindowRecord {
            id: WindowId(1),
            app_id: ApplicationId::trusted("system.control-center"),
            title: "Control Center".to_string(),
            icon_id: "home".to_string(),
            rect: WindowRect::default(),
            restore_rect: None,
            z_index: 1,
            is_focused: true,
            minimized: false,
            maximized: false,
            suspended: false,
            flags: WindowFlags::default(),
            persist_key: None,
            app_state: Value::Null,
            launch_params: Value::Null,
            last_lifecycle_event: Some(AppLifecycleEvent::Focused.token().to_string()),
        }];

        let effects = reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::HydrateSnapshot {
                snapshot,
                mode: HydrationMode::SyncRefresh,
                revision: Some(7),
            },
        )
        .expect("sync hydrate");

        assert!(effects.is_empty());
    }

    #[test]
    fn stale_sync_snapshot_is_ignored() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();
        let first = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.settings"),
        );
        state.layout_revision = Some(12);

        let mut snapshot = DesktopState::default().snapshot();
        snapshot.windows = vec![WindowRecord {
            id: WindowId(9),
            app_id: ApplicationId::trusted("system.control-center"),
            title: "Control Center".to_string(),
            icon_id: "home".to_string(),
            rect: WindowRect::default(),
            restore_rect: None,
            z_index: 1,
            is_focused: true,
            minimized: false,
            maximized: false,
            suspended: false,
            flags: WindowFlags::default(),
            persist_key: None,
            app_state: Value::Null,
            launch_params: Value::Null,
            last_lifecycle_event: None,
        }];

        let effects = reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::HydrateSnapshot {
                snapshot,
                mode: HydrationMode::SyncRefresh,
                revision: Some(11),
            },
        )
        .expect("stale hydrate should not error");

        assert!(effects.is_empty());
        assert_eq!(state.windows.len(), 1);
        assert_eq!(state.windows[0].id, first);
        assert_eq!(state.layout_revision, Some(12));
    }

    #[test]
    fn complete_boot_hydration_augments_without_duplicate_deep_link_open() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();
        let mut snapshot = DesktopState::default().snapshot();
        snapshot.windows = vec![WindowRecord {
            id: WindowId(4),
            app_id: ApplicationId::trusted("system.settings"),
            title: "Settings - Notes roadmap".to_string(),
            icon_id: "settings".to_string(),
            rect: WindowRect::default(),
            restore_rect: None,
            z_index: 1,
            is_focused: true,
            minimized: false,
            maximized: false,
            suspended: false,
            flags: WindowFlags::default(),
            persist_key: Some("notes:roadmap".to_string()),
            app_state: Value::Null,
            launch_params: json!({ "section": "personalize", "note_slug": "roadmap" }),
            last_lifecycle_event: None,
        }];

        reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::CompleteBootHydration {
                snapshot: Some(snapshot),
                snapshot_revision: Some(21),
                theme: None,
                privileged_app_ids: Vec::new(),
                deep_link: Some(DeepLinkState {
                    open: vec![DeepLinkOpenTarget::NotesSlug("roadmap".to_string())],
                }),
            },
        )
        .expect("boot hydration");

        assert_eq!(state.windows.len(), 1);
        assert_eq!(state.layout_revision, Some(21));
        assert!(state.boot_hydrated);
    }

    #[test]
    fn complete_boot_hydration_applies_unsatisfied_deep_link_after_restore() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();
        let snapshot = DesktopState::default().snapshot();

        reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::CompleteBootHydration {
                snapshot: Some(snapshot),
                snapshot_revision: Some(30),
                theme: None,
                privileged_app_ids: Vec::new(),
                deep_link: Some(DeepLinkState {
                    open: vec![DeepLinkOpenTarget::App(ApplicationId::trusted(
                        "system.terminal",
                    ))],
                }),
            },
        )
        .expect("boot hydration");

        assert_eq!(state.windows.len(), 1);
        assert_eq!(
            state.windows[0].app_id,
            ApplicationId::trusted("system.terminal")
        );
        assert!(state.boot_hydrated);
    }

    #[test]
    fn active_modal_blocks_non_modal_focus_transitions() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();
        let parent = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.settings"),
        );
        let other = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.control-center"),
        );
        let mut modal_request = OpenWindowRequest::new(ApplicationId::trusted("system.settings"));
        modal_request.flags.modal_parent = Some(parent);
        reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::OpenWindow(modal_request),
        )
        .expect("open modal");
        let modal_id = state.windows.last().expect("modal window").id;
        assert_eq!(state.active_modal, Some(modal_id));

        let err = reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::FocusWindow { window_id: other },
        )
        .expect_err("modal should block focus");
        assert!(matches!(
            err,
            ReducerError::ModalBlocked { active_modal } if active_modal == modal_id
        ));
    }

    #[test]
    fn opening_modal_child_sets_active_modal() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();
        let parent = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.settings"),
        );
        let mut modal_request = OpenWindowRequest::new(ApplicationId::trusted("system.settings"));
        modal_request.flags.modal_parent = Some(parent);

        reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::OpenWindow(modal_request),
        )
        .expect("open modal child");

        let modal = state.windows.last().expect("modal window");
        assert_eq!(state.active_modal, Some(modal.id));
        assert_eq!(modal.flags.modal_parent, Some(parent));
        assert_eq!(state.focused_window_id(), Some(modal.id));
    }

    #[test]
    fn active_modal_blocks_new_non_modal_window_opens() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();
        let parent = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.settings"),
        );
        let mut modal_request = OpenWindowRequest::new(ApplicationId::trusted("system.settings"));
        modal_request.flags.modal_parent = Some(parent);
        reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::OpenWindow(modal_request),
        )
        .expect("open modal");
        let modal_id = state.active_modal.expect("active modal");

        let err = reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::OpenWindow(OpenWindowRequest::new(ApplicationId::trusted(
                "system.control-center",
            ))),
        )
        .expect_err("modal should block unrelated open");

        assert!(matches!(
            err,
            ReducerError::ModalBlocked { active_modal } if active_modal == modal_id
        ));
    }

    #[test]
    fn active_modal_blocks_new_modal_window_opens() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();
        let parent = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.settings"),
        );
        let mut first_modal = OpenWindowRequest::new(ApplicationId::trusted("system.settings"));
        first_modal.flags.modal_parent = Some(parent);
        reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::OpenWindow(first_modal),
        )
        .expect("open first modal");
        let active_modal = state.active_modal.expect("active modal");

        let mut second_modal = OpenWindowRequest::new(ApplicationId::trusted("system.settings"));
        second_modal.flags.modal_parent = Some(parent);
        let err = reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::OpenWindow(second_modal),
        )
        .expect_err("active modal should block second modal");

        assert!(matches!(
            err,
            ReducerError::ModalBlocked { active_modal: blocked } if blocked == active_modal
        ));
    }

    #[test]
    fn closing_active_modal_clears_active_modal_and_focuses_parent() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();
        let parent = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.settings"),
        );
        let mut modal_request = OpenWindowRequest::new(ApplicationId::trusted("system.settings"));
        modal_request.flags.modal_parent = Some(parent);
        reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::OpenWindow(modal_request),
        )
        .expect("open modal");
        let modal_id = state.active_modal.expect("active modal");

        let effects = reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::CloseWindow {
                window_id: modal_id,
            },
        )
        .expect("close modal");

        assert!(state.active_modal.is_none());
        assert_eq!(state.focused_window_id(), Some(parent));
        assert!(!state.windows.iter().any(|window| window.id == modal_id));
        assert!(effects.contains(&RuntimeEffect::FocusWindowInput(parent)));
    }

    #[test]
    fn closing_modal_parent_while_child_is_active_is_rejected() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();
        let parent = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.settings"),
        );
        let mut modal_request = OpenWindowRequest::new(ApplicationId::trusted("system.settings"));
        modal_request.flags.modal_parent = Some(parent);
        reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::OpenWindow(modal_request),
        )
        .expect("open modal");
        let modal_id = state.active_modal.expect("active modal");

        let err = reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::CloseWindow { window_id: parent },
        )
        .expect_err("modal parent close should be blocked");

        assert!(matches!(
            err,
            ReducerError::ModalBlocked { active_modal } if active_modal == modal_id
        ));
    }

    #[test]
    fn policy_overlay_grants_privileged_command_access() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();
        let terminal = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.terminal"),
        );

        let denied = reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::HandleAppCommand {
                window_id: terminal,
                command: AppCommand::OpenExternalUrl {
                    url: "https://example.com".to_string(),
                },
            },
        )
        .expect_err("terminal should not have external-url access by default");
        assert!(matches!(denied, ReducerError::CapabilityDenied { .. }));

        reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::CompleteBootHydration {
                snapshot: None,
                snapshot_revision: None,
                theme: None,
                privileged_app_ids: vec!["system.terminal".to_string()],
                deep_link: None,
            },
        )
        .expect("hydrate policy overlay");

        let effects = reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::HandleAppCommand {
                window_id: terminal,
                command: AppCommand::OpenExternalUrl {
                    url: "https://example.com".to_string(),
                },
            },
        )
        .expect("overlay should authorize command");
        assert_eq!(
            effects,
            vec![RuntimeEffect::OpenExternalUrl(
                "https://example.com".to_string()
            )]
        );
    }

    #[test]
    fn capability_denial_returns_stable_diagnostic_event() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();
        let terminal = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.terminal"),
        );

        let err = reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::HandleAppCommand {
                window_id: terminal,
                command: AppCommand::OpenExternalUrl {
                    url: "https://example.com".to_string(),
                },
            },
        )
        .expect_err("terminal should not have external-url access by default");

        match err {
            ReducerError::CapabilityDenied {
                app_id,
                capability,
                window_id,
                diagnostic_event,
            } => {
                assert_eq!(app_id, "system.terminal");
                assert_eq!(capability, AppCapability::ExternalUrl);
                assert_eq!(window_id, terminal);
                assert_eq!(
                    diagnostic_event,
                    Box::new(AppEvent::capability_denied(
                        "system.terminal",
                        AppCapability::ExternalUrl,
                        terminal.0,
                        "open-external-url",
                    ))
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn notes_deeplink_resolves_to_settings_compatibility_route() {
        let request =
            build_open_request_from_deeplink(DeepLinkOpenTarget::NotesSlug("roadmap".to_string()));

        assert_eq!(request.app_id, apps::settings_application_id());
        assert_eq!(request.persist_key.as_deref(), Some("notes:roadmap"));
        assert_eq!(
            request.launch_params,
            json!({ "section": "personalize", "note_slug": "roadmap" })
        );
    }

    #[test]
    fn projects_deeplink_resolves_to_control_center_compatibility_route() {
        let request =
            build_open_request_from_deeplink(DeepLinkOpenTarget::ProjectSlug("alpha".to_string()));

        assert_eq!(
            request.app_id,
            ApplicationId::trusted("system.control-center")
        );
        assert_eq!(request.persist_key.as_deref(), Some("projects:alpha"));
        assert_eq!(
            request.launch_params,
            json!({ "section": "overview", "project_slug": "alpha" })
        );
    }
}
