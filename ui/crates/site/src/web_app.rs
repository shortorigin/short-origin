//! Root route components and browser-first shell boot for the site shell.

use desktop_runtime::{
    BrowserE2eConfig, DesktopAction, DesktopProvider, DesktopShell, current_browser_e2e_config,
    use_desktop_runtime,
};
use leptos::prelude::*;
#[cfg(target_arch = "wasm32")]
use leptos::task::spawn_local;
use leptos_meta::*;
use leptos_router::components::{A, Route, Router, Routes};
use leptos_router::hooks::use_params_map;
use leptos_router::path;
use platform_host_web::build_host_services;
#[cfg(target_arch = "wasm32")]
use platform_host_web::{
    ShellSyncKind, decode_shell_sync_event, shell_sync_sender_id, should_apply_shell_sync_event,
};

use crate::browser_navigation::{BrowserRoute, current_browser_route};

const DESKTOP_THEME_CSS: &str = concat!(
    include_str!("generated/tokens.css"),
    include_str!("generated/tailwind.css"),
    include_str!("styles/primitives.css"),
    include_str!("styles/components.css"),
    include_str!("styles/shell.css"),
    include_str!("styles/a11y.css"),
);

fn note_shell_compatibility_href(slug: &str) -> String {
    format!("/?open=notes:{slug}")
}

fn project_shell_compatibility_href(slug: &str) -> String {
    format!("/?open=projects:{slug}")
}

#[component]
/// Root application component that configures metadata, routes, and the desktop shell entrypoint.
pub fn SiteApp() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="Origin OS" />
        <Meta
            name="description"
            content="Origin OS is a Leptos shell for wasmCloud-managed platform operations."
        />
        <style id="desktop-theme-css">{DESKTOP_THEME_CSS}</style>

        <Router>
            <main class="site-root">
                <Routes fallback=|| view! { <p>"Not found."</p> }>
                    <Route path=path!("") view=DesktopEntry />
                    <Route path=path!("/notes/:slug") view=CanonicalNoteRoute />
                    <Route path=path!("/projects/:slug") view=CanonicalProjectRoute />
                </Routes>
            </main>
        </Router>
    }
}

#[component]
/// Default route that mounts the desktop runtime provider and shell.
pub fn DesktopEntry() -> impl IntoView {
    let host_services = build_host_services();
    let initial_deep_link = match current_browser_route() {
        Some(BrowserRoute::Shell(Some(deep_link))) if !deep_link.open.is_empty() => Some(deep_link),
        _ => None,
    };
    if let Some(browser_e2e) = current_browser_e2e_config() {
        provide_context::<BrowserE2eConfig>(browser_e2e);
    }
    view! {
        <DesktopProvider host_services initial_deep_link=initial_deep_link>
            <DesktopUrlBoot />
            <BrowserRuntimeEnhancements />
            <BrowserShellSync />
            <DesktopShell />
        </DesktopProvider>
    }
}

#[component]
fn DesktopUrlBoot() -> impl IntoView {
    let runtime = use_desktop_runtime();

    Effect::new(move |_| {
        if !runtime.state.get().boot_hydrated {
            return;
        }
        if let Some(BrowserRoute::Shell(Some(deep_link))) = current_browser_route()
            && !deep_link.open.is_empty()
        {
            runtime.dispatch_action(DesktopAction::ApplyDeepLink { deep_link });
        }
    });

    ().into_view()
}

#[component]
fn BrowserRuntimeEnhancements() -> impl IntoView {
    Effect::new(move |_| {
        crate::pwa::register_service_worker();
    });

    ().into_view()
}

#[component]
fn BrowserShellSync() -> impl IntoView {
    #[cfg(target_arch = "wasm32")]
    let runtime = use_desktop_runtime();

    #[cfg(target_arch = "wasm32")]
    {
        use desktop_runtime::{HydrationMode, load_durable_boot_snapshot, load_theme};
        use wasm_bindgen::{JsCast, closure::Closure};

        enum BrowserShellSyncBinding {
            Active {
                channel: web_sys::BroadcastChannel,
                callback: Closure<dyn FnMut(web_sys::MessageEvent)>,
            },
            Inactive,
        }

        impl Drop for BrowserShellSyncBinding {
            fn drop(&mut self) {
                if let Self::Active { channel, callback } = self {
                    channel.set_onmessage(None);
                    channel.close();
                    let _ = callback;
                }
            }
        }

        let host = runtime.host.get_value();
        Effect::new(move |binding: Option<BrowserShellSyncBinding>| {
            drop(binding);
            if !platform_host_web::broadcast_channel_supported() {
                return BrowserShellSyncBinding::Inactive;
            }

            let Ok(channel) = web_sys::BroadcastChannel::new("origin-os-shell-sync") else {
                return BrowserShellSyncBinding::Inactive;
            };
            let runtime = runtime.clone();
            let host = host.clone();
            let sender_id = shell_sync_sender_id();

            let callback = Closure::<dyn FnMut(web_sys::MessageEvent)>::wrap(Box::new(
                move |event: web_sys::MessageEvent| {
                    let Some(message) = event.data().as_string() else {
                        return;
                    };
                    let Some(sync_event) = decode_shell_sync_event(&message) else {
                        return;
                    };
                    if !runtime.state.get_untracked().boot_hydrated {
                        return;
                    }

                    match sync_event.kind {
                        ShellSyncKind::Theme => {
                            if !should_apply_shell_sync_event(
                                &sync_event,
                                &sender_id,
                                runtime.state.get_untracked().theme_revision,
                            ) {
                                return;
                            }
                            let runtime = runtime.clone();
                            let host = host.clone();
                            spawn_local(async move {
                                if let Some(theme) = load_theme(&host).await {
                                    runtime.dispatch_action(DesktopAction::HydrateTheme {
                                        theme,
                                        revision: Some(sync_event.revision),
                                    });
                                }
                            });
                        }
                        ShellSyncKind::Layout => {
                            if !should_apply_shell_sync_event(
                                &sync_event,
                                &sender_id,
                                runtime.state.get_untracked().layout_revision,
                            ) {
                                return;
                            }
                            let runtime = runtime.clone();
                            let host = host.clone();
                            spawn_local(async move {
                                if let Some(snapshot) = load_durable_boot_snapshot(&host).await {
                                    runtime.dispatch_action(DesktopAction::HydrateSnapshot {
                                        snapshot,
                                        mode: HydrationMode::SyncRefresh,
                                        revision: Some(sync_event.revision),
                                    });
                                }
                            });
                        }
                    }
                },
            ));

            channel.set_onmessage(Some(callback.as_ref().unchecked_ref()));

            BrowserShellSyncBinding::Active { channel, callback }
        });
    }

    ().into_view()
}

#[component]
fn CanonicalNoteRoute() -> impl IntoView {
    let params = use_params_map();
    let slug = move || {
        params
            .with(|map| map.get("slug"))
            .unwrap_or_else(|| "unknown".to_string())
    };

    view! {
        <section class="canonical-content canonical-note">
            <h1>"Note"</h1>
            <p>{move || format!("Slug: {}", slug())}</p>
            <p>"Browser-native note compatibility route that opens the shell's Settings-based compatibility route."</p>
            <A href=move || note_shell_compatibility_href(&slug())>"Open in Shell"</A>
        </section>
    }
}

#[component]
fn CanonicalProjectRoute() -> impl IntoView {
    let params = use_params_map();
    let slug = move || {
        params
            .with(|map| map.get("slug"))
            .unwrap_or_else(|| "unknown".to_string())
    };

    view! {
        <section class="canonical-content canonical-project">
            <h1>"Project"</h1>
            <p>{move || format!("Slug: {}", slug())}</p>
            <p>"Browser-native project compatibility route that opens the shell's Control Center compatibility route."</p>
            <A href=move || project_shell_compatibility_href(&slug())>"Open in Shell"</A>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_shell_open_link_uses_compatibility_route() {
        assert_eq!(
            note_shell_compatibility_href("roadmap"),
            "/?open=notes:roadmap"
        );
    }

    #[test]
    fn project_shell_open_link_uses_compatibility_route() {
        assert_eq!(
            project_shell_compatibility_href("alpha"),
            "/?open=projects:alpha"
        );
    }
}
