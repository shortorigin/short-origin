//! Reducer actions, side-effect intents, and transition logic for the desktop runtime.

mod appearance;

use desktop_app_contract::{AppCapability, AppCommand, AppEvent, AppLifecycleEvent, ApplicationId};
use platform_host::{
    WallpaperAssetMetadataPatch, WallpaperAssetRecord, WallpaperCollection, WallpaperConfig,
    WallpaperImportRequest, WallpaperLibrarySnapshot,
};
use serde_json::{json, Value};
use thiserror::Error;

use crate::apps;
use crate::model::{
    DeepLinkOpenTarget, DeepLinkState, DesktopSkin, DesktopSnapshot, DesktopState, DesktopTheme,
    InteractionState, OpenWindowRequest, PointerPosition, ResizeEdge, ResizeSession, WindowId,
    WindowRecord, WindowRect, DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH,
};
use crate::window_manager::{
    focus_window_internal, normalize_window_stack, resize_rect, snap_window_to_viewport_edge,
    MIN_WINDOW_HEIGHT, MIN_WINDOW_WIDTH,
};
use appearance::desktop_skin_from_id;

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
    /// Set the active desktop skin preset.
    SetSkin {
        /// New typed skin id.
        skin: DesktopSkin,
    },
    /// Replace the committed desktop wallpaper configuration.
    SetCurrentWallpaper {
        /// Wallpaper configuration to commit.
        config: WallpaperConfig,
    },
    /// Start previewing a wallpaper configuration.
    PreviewWallpaper {
        /// Wallpaper configuration to preview.
        config: WallpaperConfig,
    },
    /// Commit the active wallpaper preview.
    ApplyWallpaperPreview,
    /// Clear the active wallpaper preview.
    ClearWallpaperPreview,
    /// Hydrate theme state independently from layout restore.
    HydrateTheme {
        /// Persisted theme payload.
        theme: DesktopTheme,
    },
    /// Hydrate wallpaper state independently from layout restore.
    HydrateWallpaper {
        /// Persisted wallpaper payload.
        wallpaper: WallpaperConfig,
    },
    /// Replace the wallpaper library snapshot.
    WallpaperLibraryLoaded {
        /// Imported wallpaper library snapshot loaded from the host.
        snapshot: WallpaperLibrarySnapshot,
    },
    /// Upsert one imported wallpaper asset inside the merged wallpaper library.
    WallpaperAssetUpdated {
        /// Updated imported wallpaper asset metadata.
        asset: WallpaperAssetRecord,
    },
    /// Upsert one wallpaper collection inside the merged wallpaper library.
    WallpaperCollectionUpdated {
        /// Updated or newly created collection metadata.
        collection: WallpaperCollection,
    },
    /// Remove one wallpaper collection from the merged wallpaper library.
    WallpaperCollectionDeleted {
        /// Deleted collection identifier.
        collection_id: String,
    },
    /// Remove one imported wallpaper asset from the merged wallpaper library.
    WallpaperAssetDeleted {
        /// Deleted imported wallpaper asset identifier.
        asset_id: String,
        /// Current managed library usage in bytes after deletion.
        used_bytes: u64,
    },
    /// Toggle high-contrast rendering.
    SetHighContrast {
        /// Whether high contrast is enabled.
        enabled: bool,
    },
    /// Toggle reduced-motion rendering.
    SetReducedMotion {
        /// Whether reduced motion is enabled.
        enabled: bool,
    },
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
    /// Hydrate runtime state from a persisted snapshot.
    HydrateSnapshot {
        /// Snapshot payload to restore.
        snapshot: DesktopSnapshot,
    },
    /// Apply URL-derived deep-link instructions.
    ApplyDeepLink {
        /// Parsed deep-link payload.
        deep_link: DeepLinkState,
    },
    /// Marks asynchronous boot hydration as complete for the current runtime session.
    BootHydrationComplete,
}

#[derive(Debug, Clone, PartialEq)]
/// Side-effect intents emitted by [`reduce_desktop`] for the shell runtime to execute.
pub enum RuntimeEffect {
    /// Persist the current desktop layout snapshot.
    PersistLayout,
    /// Persist theme changes.
    PersistTheme,
    /// Persist wallpaper changes.
    PersistWallpaper,
    /// Persist terminal history changes.
    PersistTerminalHistory,
    /// Move focus into the newly focused window's primary input.
    FocusWindowInput(WindowId),
    /// Parse and open deep-link targets in the UI layer.
    ParseAndOpenDeepLink(DeepLinkState),
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
    /// Load the wallpaper library snapshot from the host.
    LoadWallpaperLibrary,
    /// Import a wallpaper through the host picker flow.
    ImportWallpaperFromPicker {
        /// Import request payload.
        request: WallpaperImportRequest,
    },
    /// Update managed wallpaper metadata through the host service.
    UpdateWallpaperAssetMetadata {
        /// Managed asset identifier.
        asset_id: String,
        /// Metadata patch payload.
        patch: WallpaperAssetMetadataPatch,
    },
    /// Create a wallpaper collection.
    CreateWallpaperCollection {
        /// New collection label.
        display_name: String,
    },
    /// Rename a wallpaper collection.
    RenameWallpaperCollection {
        /// Collection identifier.
        collection_id: String,
        /// Updated collection label.
        display_name: String,
    },
    /// Delete a wallpaper collection.
    DeleteWallpaperCollection {
        /// Collection identifier.
        collection_id: String,
    },
    /// Delete a wallpaper asset.
    DeleteWallpaperAsset {
        /// Managed asset identifier.
        asset_id: String,
    },
    /// Emit a host notification request.
    Notify {
        /// Notification title.
        title: String,
        /// Notification body.
        body: String,
    },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Reducer errors for invalid actions (for example, referencing a missing window).
pub enum ReducerError {
    /// The target window id was not found in the current state.
    #[error("window not found")]
    WindowNotFound,
    /// A wallpaper configuration violated runtime constraints.
    #[error("invalid wallpaper configuration: {0}")]
    InvalidWallpaperConfig(String),
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
            let descriptor = apps::app_descriptor_by_id(&app_id);

            if descriptor.single_instance {
                if let Some(window_id) = preferred_window_for_app(state, &app_id) {
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
                        reduce_desktop(
                            state,
                            interaction,
                            DesktopAction::FocusWindow { window_id },
                        )?
                    } else {
                        Vec::new()
                    };
                    effects.extend(nested);
                    return Ok(effects);
                }
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
            state.start_menu_open = false;
            record_window_lifecycle(state, window_id, AppLifecycleEvent::Mounted);
            effects.push(RuntimeEffect::DispatchLifecycle {
                window_id,
                event: AppLifecycleEvent::Mounted,
            });
            emit_focus_transition(previously_focused, Some(window_id), state, &mut effects);
            effects.push(RuntimeEffect::PersistLayout);
            effects.push(RuntimeEffect::FocusWindowInput(window_id));
            if apps::is_dialup_application_id(&req.app_id) && state.theme.audio_enabled {
                effects.push(RuntimeEffect::PlaySound("dialup-open"));
            }
        }
        DesktopAction::CloseWindow { window_id } => {
            let was_focused = state.focused_window_id() == Some(window_id);
            effects.push(RuntimeEffect::DispatchLifecycle {
                window_id,
                event: AppLifecycleEvent::Closing,
            });
            let before_len = state.windows.len();
            state.windows.retain(|w| w.id != window_id);
            if state.windows.len() == before_len {
                return Err(ReducerError::WindowNotFound);
            }
            if state.active_modal == Some(window_id) {
                state.active_modal = None;
            }
            normalize_window_stack(state);
            effects.push(RuntimeEffect::DispatchLifecycle {
                window_id,
                event: AppLifecycleEvent::Closed,
            });
            if was_focused {
                let new_focus = state.focused_window_id();
                emit_focus_transition(Some(window_id), new_focus, state, &mut effects);
            }
            effects.push(RuntimeEffect::PersistLayout);
        }
        DesktopAction::FocusWindow { window_id } => {
            let previous_focus = state.focused_window_id();
            if !focus_window_internal(state, window_id) {
                return Err(ReducerError::WindowNotFound);
            }
            state.start_menu_open = false;
            emit_focus_transition(previous_focus, Some(window_id), state, &mut effects);
            effects.push(RuntimeEffect::FocusWindowInput(window_id));
        }
        DesktopAction::MinimizeWindow { window_id } => {
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
            state.start_menu_open = !state.start_menu_open;
        }
        DesktopAction::CloseStartMenu => {
            state.start_menu_open = false;
        }
        DesktopAction::BeginMove { window_id, pointer } => {
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
            if let Some(required) = command_required_capability(&command) {
                if !command_allowed_for_app(&source_app_id, required) {
                    return Ok(effects);
                }
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
                AppCommand::SetDesktopSkin { skin_id } => {
                    if let Some(skin) = desktop_skin_from_id(&skin_id) {
                        let nested =
                            reduce_desktop(state, interaction, DesktopAction::SetSkin { skin })?;
                        effects.extend(nested);
                    }
                }
                AppCommand::PreviewWallpaper { config } => {
                    let nested = reduce_desktop(
                        state,
                        interaction,
                        DesktopAction::PreviewWallpaper { config },
                    )?;
                    effects.extend(nested);
                }
                AppCommand::ApplyWallpaperPreview => {
                    let nested =
                        reduce_desktop(state, interaction, DesktopAction::ApplyWallpaperPreview)?;
                    effects.extend(nested);
                }
                AppCommand::SetCurrentWallpaper { config } => {
                    let nested = reduce_desktop(
                        state,
                        interaction,
                        DesktopAction::SetCurrentWallpaper { config },
                    )?;
                    effects.extend(nested);
                }
                AppCommand::ClearWallpaperPreview => {
                    let nested =
                        reduce_desktop(state, interaction, DesktopAction::ClearWallpaperPreview)?;
                    effects.extend(nested);
                }
                AppCommand::ImportWallpaperFromPicker { request } => {
                    effects.push(RuntimeEffect::ImportWallpaperFromPicker { request });
                }
                AppCommand::RenameWallpaperAsset {
                    asset_id,
                    display_name,
                } => {
                    effects.push(RuntimeEffect::UpdateWallpaperAssetMetadata {
                        asset_id,
                        patch: WallpaperAssetMetadataPatch {
                            display_name: Some(display_name),
                            ..Default::default()
                        },
                    });
                }
                AppCommand::SetWallpaperFavorite { asset_id, favorite } => {
                    effects.push(RuntimeEffect::UpdateWallpaperAssetMetadata {
                        asset_id,
                        patch: WallpaperAssetMetadataPatch {
                            favorite: Some(favorite),
                            ..Default::default()
                        },
                    });
                }
                AppCommand::SetWallpaperTags { asset_id, tags } => {
                    effects.push(RuntimeEffect::UpdateWallpaperAssetMetadata {
                        asset_id,
                        patch: WallpaperAssetMetadataPatch {
                            tags: Some(tags),
                            ..Default::default()
                        },
                    });
                }
                AppCommand::SetWallpaperCollections {
                    asset_id,
                    collection_ids,
                } => {
                    effects.push(RuntimeEffect::UpdateWallpaperAssetMetadata {
                        asset_id,
                        patch: WallpaperAssetMetadataPatch {
                            collection_ids: Some(collection_ids),
                            ..Default::default()
                        },
                    });
                }
                AppCommand::CreateWallpaperCollection { display_name } => {
                    effects.push(RuntimeEffect::CreateWallpaperCollection { display_name });
                }
                AppCommand::RenameWallpaperCollection {
                    collection_id,
                    display_name,
                } => {
                    effects.push(RuntimeEffect::RenameWallpaperCollection {
                        collection_id,
                        display_name,
                    });
                }
                AppCommand::DeleteWallpaperCollection { collection_id } => {
                    effects.push(RuntimeEffect::DeleteWallpaperCollection { collection_id });
                }
                AppCommand::DeleteWallpaperAsset { asset_id } => {
                    effects.push(RuntimeEffect::DeleteWallpaperAsset { asset_id });
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
                AppCommand::Notify { title, body } => {
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
        DesktopAction::HydrateSnapshot { snapshot } => {
            let max_restore = state.preferences.max_restore_windows;
            let theme = state.theme.clone();
            let wallpaper_config = state.wallpaper.clone();
            let wallpaper_preview = state.wallpaper_preview.clone();
            let wallpaper_library = state.wallpaper_library.clone();
            *state = DesktopState::from_snapshot(snapshot);
            state.theme = theme;
            state.wallpaper = wallpaper_config;
            state.wallpaper_preview = wallpaper_preview;
            state.wallpaper_library = wallpaper_library;
            if state.windows.len() > max_restore {
                state.windows.truncate(max_restore);
            }
            normalize_window_stack(state);
            for window in state.windows.iter_mut() {
                if window.last_lifecycle_event.is_none() {
                    window.last_lifecycle_event =
                        Some(AppLifecycleEvent::Mounted.token().to_string());
                }
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
        DesktopAction::ApplyDeepLink { deep_link } => {
            effects.push(RuntimeEffect::ParseAndOpenDeepLink(deep_link));
        }
        DesktopAction::BootHydrationComplete => {
            state.boot_hydrated = true;
        }
        DesktopAction::SetSkin { .. }
        | DesktopAction::SetCurrentWallpaper { .. }
        | DesktopAction::PreviewWallpaper { .. }
        | DesktopAction::ApplyWallpaperPreview
        | DesktopAction::ClearWallpaperPreview
        | DesktopAction::HydrateTheme { .. }
        | DesktopAction::HydrateWallpaper { .. }
        | DesktopAction::WallpaperLibraryLoaded { .. }
        | DesktopAction::WallpaperAssetUpdated { .. }
        | DesktopAction::WallpaperCollectionUpdated { .. }
        | DesktopAction::WallpaperCollectionDeleted { .. }
        | DesktopAction::WallpaperAssetDeleted { .. }
        | DesktopAction::SetHighContrast { .. }
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
            let mut req = OpenWindowRequest::new(ApplicationId::trusted("system.notepad"));
            req.title = Some(format!("Note - {slug}"));
            req.persist_key = Some(format!("notes:{slug}"));
            req.launch_params = json!({ "slug": slug });
            req
        }
        DeepLinkOpenTarget::ProjectSlug(slug) => {
            let mut req = OpenWindowRequest::new(ApplicationId::trusted("system.explorer"));
            req.title = Some(format!("Project - {slug}"));
            req.persist_key = Some(format!("projects:{slug}"));
            req.launch_params = json!({ "project_slug": slug });
            req
        }
    }
}

fn next_window_id(state: &mut DesktopState) -> WindowId {
    let id = WindowId(state.next_window_id);
    state.next_window_id = state.next_window_id.saturating_add(1);
    id
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
        window.last_lifecycle_event = Some(event.token().to_string());
    }
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

    if let Some(previous) = previous_focus {
        if state.windows.iter().any(|window| window.id == previous) {
            record_window_lifecycle(state, previous, AppLifecycleEvent::Blurred);
            effects.push(RuntimeEffect::DispatchLifecycle {
                window_id: previous,
                event: AppLifecycleEvent::Blurred,
            });
        }
    }

    if let Some(next) = next_focus {
        if state.windows.iter().any(|window| window.id == next) {
            record_window_lifecycle(state, next, AppLifecycleEvent::Focused);
            effects.push(RuntimeEffect::DispatchLifecycle {
                window_id: next,
                event: AppLifecycleEvent::Focused,
            });
        }
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
        AppCommand::SetDesktopSkin { .. }
        | AppCommand::SetDesktopHighContrast { .. }
        | AppCommand::SetDesktopReducedMotion { .. } => Some(AppCapability::Theme),
        AppCommand::PreviewWallpaper { .. }
        | AppCommand::ApplyWallpaperPreview
        | AppCommand::SetCurrentWallpaper { .. }
        | AppCommand::ClearWallpaperPreview
        | AppCommand::ImportWallpaperFromPicker { .. }
        | AppCommand::RenameWallpaperAsset { .. }
        | AppCommand::SetWallpaperFavorite { .. }
        | AppCommand::SetWallpaperTags { .. }
        | AppCommand::SetWallpaperCollections { .. }
        | AppCommand::CreateWallpaperCollection { .. }
        | AppCommand::RenameWallpaperCollection { .. }
        | AppCommand::DeleteWallpaperCollection { .. }
        | AppCommand::DeleteWallpaperAsset { .. } => Some(AppCapability::Wallpaper),
        AppCommand::Notify { .. } => Some(AppCapability::Notifications),
    }
}

fn command_allowed_for_app(app_id: &ApplicationId, required: AppCapability) -> bool {
    if apps::app_is_privileged_by_id(app_id) {
        return true;
    }
    apps::app_requested_capabilities_by_id(app_id).contains(&required)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::model::{InteractionState, OpenWindowRequest};
    use desktop_app_contract::ApplicationId;
    use platform_host::{WallpaperDisplayMode, WallpaperSelection};

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
            ApplicationId::trusted("system.explorer"),
        );
        let second = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.notepad"),
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
            ApplicationId::trusted("system.explorer"),
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
            ApplicationId::trusted("system.explorer"),
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
    fn activate_app_opens_new_window_for_multi_instance_apps() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();

        reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::ActivateApp {
                app_id: ApplicationId::trusted("system.explorer"),
                viewport: None,
            },
        )
        .expect("activate explorer first");
        reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::ActivateApp {
                app_id: ApplicationId::trusted("system.explorer"),
                viewport: None,
            },
        )
        .expect("activate explorer second");

        assert_eq!(state.windows.len(), 2);
        assert!(state
            .windows
            .iter()
            .all(|w| w.app_id == ApplicationId::trusted("system.explorer")));
    }

    #[test]
    fn activate_settings_uses_default_open_request_without_theme_launch_params() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();
        state.theme.skin = DesktopSkin::ClassicXp;
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
            ApplicationId::trusted("system.explorer"),
        );
        let second = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.calculator"),
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
            ApplicationId::trusted("system.explorer"),
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
    fn set_skin_updates_theme_and_persists() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();

        let effects = reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::SetSkin {
                skin: DesktopSkin::Classic95,
            },
        )
        .expect("set skin");

        assert_eq!(state.theme.skin, DesktopSkin::Classic95);
        assert_eq!(effects, vec![RuntimeEffect::PersistTheme]);
    }

    #[test]
    fn preview_and_apply_wallpaper_are_independent_of_theme() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();
        let next = WallpaperConfig {
            selection: WallpaperSelection::BuiltIn {
                wallpaper_id: "sunset-lake".to_string(),
            },
            display_mode: WallpaperDisplayMode::Fit,
            ..WallpaperConfig::default()
        };

        let effects = reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::PreviewWallpaper {
                config: next.clone(),
            },
        )
        .expect("preview wallpaper");
        assert!(effects.is_empty());
        assert_eq!(state.wallpaper_preview, Some(next.clone()));
        assert_eq!(state.theme.skin, DesktopSkin::SoftNeumorphic);

        let effects = reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::ApplyWallpaperPreview,
        )
        .expect("apply wallpaper preview");
        assert_eq!(state.wallpaper, next);
        assert!(state.wallpaper_preview.is_none());
        assert_eq!(effects, vec![RuntimeEffect::PersistWallpaper]);
        assert_eq!(state.theme.skin, DesktopSkin::SoftNeumorphic);
    }

    #[test]
    fn tile_mode_rejects_animated_wallpapers() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();

        let err = reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::SetCurrentWallpaper {
                config: WallpaperConfig {
                    selection: WallpaperSelection::BuiltIn {
                        wallpaper_id: "aurora-flow".to_string(),
                    },
                    display_mode: WallpaperDisplayMode::Tile,
                    ..WallpaperConfig::default()
                },
            },
        )
        .expect_err("animated wallpaper tile mode should fail");

        assert!(matches!(err, ReducerError::InvalidWallpaperConfig(_)));
    }

    #[test]
    fn handle_app_command_persist_state_updates_window_record_and_persists() {
        let mut state = DesktopState::default();
        let mut interaction = InteractionState::default();
        let window_id = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.explorer"),
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
            ApplicationId::trusted("system.notepad"),
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
        let explorer = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.explorer"),
        );
        let terminal = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.terminal"),
        );

        let explorer_effects = reduce_desktop(
            &mut state,
            &mut interaction,
            DesktopAction::MinimizeWindow {
                window_id: explorer,
            },
        )
        .expect("minimize explorer");
        let explorer_window = state.windows.iter().find(|w| w.id == explorer).unwrap();
        assert!(explorer_window.suspended);
        assert!(
            explorer_effects.contains(&RuntimeEffect::DispatchLifecycle {
                window_id: explorer,
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
            ApplicationId::trusted("system.explorer"),
        );
        let second = open(
            &mut state,
            &mut interaction,
            ApplicationId::trusted("system.notepad"),
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
            ApplicationId::trusted("system.explorer"),
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
}
