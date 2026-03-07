//! Root route components and browser-first shell boot for the site shell.

use desktop_runtime::{
    current_browser_e2e_config, use_desktop_runtime, BrowserE2eConfig, DesktopAction,
    DesktopProvider, DesktopShell,
};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use platform_host_web::build_host_services;

use crate::browser_navigation::{current_browser_route, BrowserRoute};

const DESKTOP_THEME_CSS: &str = concat!(
    include_str!("generated/tokens.css"),
    include_str!("generated/tailwind.css"),
    include_str!("styles/primitives.css"),
    include_str!("styles/components.css"),
    include_str!("styles/shell.css"),
    include_str!("styles/a11y.css"),
);

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
                <Routes>
                    <Route path="" view=DesktopEntry />
                    <Route path="/notes/:slug" view=CanonicalNoteRoute />
                    <Route path="/projects/:slug" view=CanonicalProjectRoute />
                </Routes>
            </main>
        </Router>
    }
}

#[component]
/// Default route that mounts the desktop runtime provider and shell.
pub fn DesktopEntry() -> impl IntoView {
    let host_services = build_host_services();
    if let Some(browser_e2e) = current_browser_e2e_config() {
        provide_context::<BrowserE2eConfig>(browser_e2e);
    }
    view! {
        <DesktopProvider host_services>
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

    create_effect(move |_| {
        if let Some(BrowserRoute::Shell(Some(deep_link))) = current_browser_route() {
            if !deep_link.open.is_empty() {
                runtime.dispatch_action(DesktopAction::ApplyDeepLink { deep_link });
            }
        }
    });

    ().into_view()
}

#[component]
fn BrowserRuntimeEnhancements() -> impl IntoView {
    create_effect(move |_| {
        crate::pwa::register_service_worker();
    });

    ().into_view()
}

#[component]
fn BrowserShellSync() -> impl IntoView {
    let _runtime = use_desktop_runtime();

    #[cfg(target_arch = "wasm32")]
    {
        use desktop_runtime::{load_durable_boot_snapshot, load_theme, load_wallpaper};
        use wasm_bindgen::{closure::Closure, JsCast};

        let host = _runtime.host.get_value();
        create_effect(move |_| {
            if !platform_host_web::broadcast_channel_supported() {
                return;
            }

            let Ok(channel) = web_sys::BroadcastChannel::new("origin-os-shell-sync") else {
                return;
            };
            let runtime = _runtime.clone();
            let host = host.clone();

            let callback = Closure::<dyn FnMut(web_sys::MessageEvent)>::wrap(Box::new(
                move |event: web_sys::MessageEvent| {
                    let Some(message) = event.data().as_string() else {
                        return;
                    };

                    match message.as_str() {
                        "theme-changed" => {
                            let runtime = runtime.clone();
                            let host = host.clone();
                            leptos::spawn_local(async move {
                                if let Some(theme) = load_theme(&host).await {
                                    runtime.dispatch_action(DesktopAction::HydrateTheme { theme });
                                }
                            });
                        }
                        "wallpaper-changed" => {
                            let runtime = runtime.clone();
                            let host = host.clone();
                            leptos::spawn_local(async move {
                                if let Some(wallpaper) = load_wallpaper(&host).await {
                                    runtime.dispatch_action(DesktopAction::HydrateWallpaper {
                                        wallpaper,
                                    });
                                }
                            });
                        }
                        "layout-changed" => {
                            let runtime = runtime.clone();
                            let host = host.clone();
                            leptos::spawn_local(async move {
                                if let Some(snapshot) = load_durable_boot_snapshot(&host).await {
                                    runtime.dispatch_action(DesktopAction::HydrateSnapshot {
                                        snapshot,
                                    });
                                }
                            });
                        }
                        _ => {}
                    }
                },
            ));

            channel.set_onmessage(Some(callback.as_ref().unchecked_ref()));

            on_cleanup(move || {
                channel.set_onmessage(None);
                channel.close();
                drop(callback);
            });
        });
    }

    ().into_view()
}

#[component]
fn CanonicalNoteRoute() -> impl IntoView {
    let params = use_params_map();
    let slug = move || {
        params
            .with(|map| map.get("slug").cloned())
            .unwrap_or_else(|| "unknown".to_string())
    };

    view! {
        <section class="canonical-content canonical-note">
            <h1>"Note"</h1>
            <p>{move || format!("Slug: {}", slug())}</p>
            <p>"Browser-native note route with shell compatibility open intents."</p>
            <A href=move || format!("/?open=notes:{}", slug())>"Open in Shell"</A>
        </section>
    }
}

#[component]
fn CanonicalProjectRoute() -> impl IntoView {
    let params = use_params_map();
    let slug = move || {
        params
            .with(|map| map.get("slug").cloned())
            .unwrap_or_else(|| "unknown".to_string())
    };

    view! {
        <section class="canonical-content canonical-project">
            <h1>"Project"</h1>
            <p>{move || format!("Slug: {}", slug())}</p>
            <p>"Browser-native project route with shell compatibility open intents."</p>
            <A href=move || format!("/?open=projects:{}", slug())>"Open in Shell"</A>
        </section>
    }
}
