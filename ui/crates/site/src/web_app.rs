//! Root route components and URL deep-link bootstrap for the site shell.

#[cfg(any(test, target_arch = "wasm32"))]
use desktop_app_contract::ApplicationId;
#[cfg(any(test, target_arch = "wasm32"))]
use desktop_runtime::DeepLinkOpenTarget;
use desktop_runtime::{
    current_browser_e2e_config, use_desktop_runtime, BrowserE2eConfig, DeepLinkState,
    DesktopAction, DesktopProvider, DesktopShell,
};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use platform_host_web::build_host_services;

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
            <DesktopShell />
        </DesktopProvider>
    }
}

#[component]
fn DesktopUrlBoot() -> impl IntoView {
    let runtime = use_desktop_runtime();

    create_effect(move |_| {
        if let Some(deep_link) = current_url_deep_link() {
            if !deep_link.open.is_empty() {
                runtime.dispatch_action(DesktopAction::ApplyDeepLink { deep_link });
            }
        }
    });

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
            <p>"Canonical SSR route placeholder. Final version renders prebuilt HTML here."</p>
            <A href=move || format!("/?open=notes:{}", slug())>"Open in Desktop"</A>
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
            <p>"Canonical SSR route placeholder. Final version renders project metadata/details."</p>
            <A href=move || format!("/?open=projects:{}", slug())>"Open in Desktop"</A>
        </section>
    }
}

fn current_url_deep_link() -> Option<DeepLinkState> {
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window()?;
        let location = window.location();
        let search = location.search().ok()?;
        let hash = location.hash().ok().unwrap_or_default();
        let parsed = parse_deep_link_from_parts(&search, &hash);
        if parsed.open.is_empty() {
            None
        } else {
            Some(parsed)
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

#[cfg(any(test, target_arch = "wasm32"))]
fn parse_deep_link_from_parts(search: &str, hash: &str) -> DeepLinkState {
    let mut open = Vec::new();
    open.extend(parse_open_values_from_query_like(
        search.trim_start_matches('?'),
    ));

    let hash_trimmed = hash.trim_start_matches('#');
    let hash_trimmed = hash_trimmed.trim_start_matches('/');

    if let Some(path_value) = hash_trimmed.strip_prefix("open/") {
        if let Some(target) = parse_open_target(path_value) {
            open.push(target);
        }
    } else {
        open.extend(parse_open_values_from_query_like(hash_trimmed));
    }

    DeepLinkState { open }
}

#[cfg(any(test, target_arch = "wasm32"))]
fn parse_open_values_from_query_like(query: &str) -> Vec<DeepLinkOpenTarget> {
    if query.is_empty() {
        return Vec::new();
    }

    query
        .split('&')
        .filter_map(|pair| {
            let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
            (key == "open").then_some(value)
        })
        .flat_map(|value| value.split(','))
        .filter_map(parse_open_target)
        .collect()
}

#[cfg(any(test, target_arch = "wasm32"))]
fn parse_open_target(raw: &str) -> Option<DeepLinkOpenTarget> {
    let value = raw.trim();
    if value.is_empty() {
        return None;
    }

    let lowered = value.to_ascii_lowercase();
    if lowered.strip_prefix("notes:").is_some() {
        return Some(DeepLinkOpenTarget::NotesSlug(
            value[6..].to_string().trim().to_string(),
        ));
    }
    if lowered.strip_prefix("projects:").is_some() {
        return Some(DeepLinkOpenTarget::ProjectSlug(
            value[9..].to_string().trim().to_string(),
        ));
    }
    if let Some(rest) = lowered.strip_prefix("app:") {
        return parse_app_id(rest).map(DeepLinkOpenTarget::App);
    }

    parse_app_id(&lowered).map(DeepLinkOpenTarget::App)
}

#[cfg(any(test, target_arch = "wasm32"))]
fn parse_app_id(raw: &str) -> Option<ApplicationId> {
    ApplicationId::new(raw.trim()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_query_open_targets() {
        let parsed = parse_deep_link_from_parts("?open=notes:hello-world&open=system.terminal", "");
        assert_eq!(
            parsed.open,
            vec![
                DeepLinkOpenTarget::NotesSlug("hello-world".to_string()),
                DeepLinkOpenTarget::App(ApplicationId::trusted("system.terminal")),
            ]
        );
    }

    #[test]
    fn parses_hash_query_style_open_targets() {
        let parsed =
            parse_deep_link_from_parts("", "#open=projects:alpha&open=system.control-center");
        assert_eq!(
            parsed.open,
            vec![
                DeepLinkOpenTarget::ProjectSlug("alpha".to_string()),
                DeepLinkOpenTarget::App(ApplicationId::trusted("system.control-center")),
            ]
        );
    }

    #[test]
    fn parses_hash_path_style_open_target() {
        let parsed = parse_deep_link_from_parts("", "#/open/notes:hello-world");
        assert_eq!(
            parsed.open,
            vec![DeepLinkOpenTarget::NotesSlug("hello-world".to_string())]
        );
    }

    #[test]
    fn supports_comma_separated_query_targets() {
        let parsed =
            parse_deep_link_from_parts("?open=system.settings,system.terminal,projects:beta", "");
        assert_eq!(
            parsed.open,
            vec![
                DeepLinkOpenTarget::App(ApplicationId::trusted("system.settings")),
                DeepLinkOpenTarget::App(ApplicationId::trusted("system.terminal")),
                DeepLinkOpenTarget::ProjectSlug("beta".to_string()),
            ]
        );
    }
}
