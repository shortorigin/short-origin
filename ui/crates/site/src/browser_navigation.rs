//! Browser-first routing and deep-link parsing helpers.

#[cfg(any(test, target_arch = "wasm32"))]
use desktop_app_contract::ApplicationId;
#[cfg(any(test, target_arch = "wasm32"))]
use desktop_runtime::DeepLinkOpenTarget;
use desktop_runtime::DeepLinkState;

#[cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserRoute {
    Shell(Option<DeepLinkState>),
    Note { slug: String },
    Project { slug: String },
}

pub fn current_browser_route() -> Option<BrowserRoute> {
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window()?;
        let location = window.location();
        let pathname = location.pathname().ok()?;
        let search = location.search().ok().unwrap_or_default();
        let hash = location.hash().ok().unwrap_or_default();
        Some(detect_browser_route(&pathname, &search, &hash))
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

#[cfg(any(test, target_arch = "wasm32"))]
pub fn detect_browser_route(pathname: &str, search: &str, hash: &str) -> BrowserRoute {
    let normalized_path = pathname.trim().trim_matches('/');
    let segments = normalized_path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    match segments.as_slice() {
        ["notes", slug] => BrowserRoute::Note {
            slug: slug.to_string(),
        },
        ["projects", slug] => BrowserRoute::Project {
            slug: slug.to_string(),
        },
        _ => {
            let deep_link = parse_deep_link_from_parts(search, hash);
            if deep_link.open.is_empty() {
                BrowserRoute::Shell(None)
            } else {
                BrowserRoute::Shell(Some(deep_link))
            }
        }
    }
}

#[cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]
#[cfg(not(any(test, target_arch = "wasm32")))]
pub fn detect_browser_route(_pathname: &str, _search: &str, _hash: &str) -> BrowserRoute {
    BrowserRoute::Shell(None)
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
    fn parses_note_route() {
        assert_eq!(
            detect_browser_route("/notes/roadmap", "", ""),
            BrowserRoute::Note {
                slug: "roadmap".to_string()
            }
        );
    }

    #[test]
    fn parses_project_route() {
        assert_eq!(
            detect_browser_route("/projects/alpha", "", ""),
            BrowserRoute::Project {
                slug: "alpha".to_string()
            }
        );
    }

    #[test]
    fn parses_shell_route_without_open_targets() {
        assert_eq!(detect_browser_route("/", "", ""), BrowserRoute::Shell(None));
    }

    #[test]
    fn parses_query_open_targets() {
        let parsed = detect_browser_route("/", "?open=notes:hello-world&open=system.terminal", "");
        assert_eq!(
            parsed,
            BrowserRoute::Shell(Some(DeepLinkState {
                open: vec![
                    DeepLinkOpenTarget::NotesSlug("hello-world".to_string()),
                    DeepLinkOpenTarget::App(ApplicationId::trusted("system.terminal")),
                ]
            }))
        );
    }

    #[test]
    fn parses_hash_query_style_open_targets() {
        let parsed =
            detect_browser_route("/", "", "#open=projects:alpha&open=system.control-center");
        assert_eq!(
            parsed,
            BrowserRoute::Shell(Some(DeepLinkState {
                open: vec![
                    DeepLinkOpenTarget::ProjectSlug("alpha".to_string()),
                    DeepLinkOpenTarget::App(ApplicationId::trusted("system.control-center")),
                ]
            }))
        );
    }

    #[test]
    fn parses_hash_path_style_open_target() {
        let parsed = detect_browser_route("/", "", "#/open/notes:hello-world");
        assert_eq!(
            parsed,
            BrowserRoute::Shell(Some(DeepLinkState {
                open: vec![DeepLinkOpenTarget::NotesSlug("hello-world".to_string())]
            }))
        );
    }

    #[test]
    fn supports_comma_separated_query_targets() {
        let parsed = detect_browser_route(
            "/",
            "?open=system.settings,system.terminal,projects:beta",
            "",
        );
        assert_eq!(
            parsed,
            BrowserRoute::Shell(Some(DeepLinkState {
                open: vec![
                    DeepLinkOpenTarget::App(ApplicationId::trusted("system.settings")),
                    DeepLinkOpenTarget::App(ApplicationId::trusted("system.terminal")),
                    DeepLinkOpenTarget::ProjectSlug("beta".to_string()),
                ]
            }))
        );
    }
}
