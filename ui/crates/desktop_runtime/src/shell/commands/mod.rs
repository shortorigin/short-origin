#![allow(clippy::clone_on_copy)]

use desktop_app_contract::AppCommandRegistration;

use crate::components::DesktopRuntimeContext;

mod apps;
mod config;
mod data;
mod filesystem;
mod inspect;
mod theme;
mod windows;

pub(super) fn builtin_registrations(runtime: DesktopRuntimeContext) -> Vec<AppCommandRegistration> {
    let mut registrations = Vec::new();
    registrations.extend(vec![
        super::help_list_registration(runtime.clone()),
        super::help_show_registration(runtime.clone()),
        super::clear_registration(),
        super::history_list_registration(runtime.clone()),
        super::open_registration(runtime.clone()),
    ]);
    registrations.extend(apps::registrations(runtime.clone()));
    registrations.extend(windows::registrations(runtime.clone()));
    registrations.extend(theme::registrations(runtime.clone()));
    registrations.extend(inspect::registrations(runtime.clone()));
    registrations.extend(filesystem::registrations(runtime.clone()));
    registrations.extend(data::registrations());
    registrations.extend(config::registrations(runtime.clone()));
    registrations
}
