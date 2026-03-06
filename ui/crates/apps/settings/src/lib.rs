//! Built-in System Settings desktop app for wallpaper, theme, and accessibility preferences.
//!
//! The app consumes the injected v2 service surface from [`desktop_app_contract::AppServices`]
//! so wallpaper and theme configuration stay synchronized with the desktop runtime.
//! It also exposes the current host capability posture so UI flows can distinguish browser-first
//! constraints from future desktop-native capabilities.

#![warn(missing_docs, rustdoc::broken_intra_doc_links)]

use desktop_app_contract::AppServices;
use leptos::*;
use platform_host::{
    WallpaperAnimationPolicy, WallpaperAssetRecord, WallpaperCollection, WallpaperConfig,
    WallpaperDisplayMode, WallpaperMediaKind, WallpaperPosition, WallpaperSelection,
    WallpaperSourceKind,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use system_ui::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum SettingsSection {
    Personalize,
    Appearance,
    Accessibility,
}

impl SettingsSection {
    fn label(self) -> &'static str {
        match self {
            Self::Personalize => "Personalize",
            Self::Appearance => "Appearance",
            Self::Accessibility => "Accessibility",
        }
    }

    fn from_launch_param(raw: &str) -> Option<Self> {
        match raw.trim() {
            "personalize" => Some(Self::Personalize),
            "appearance" => Some(Self::Appearance),
            "accessibility" => Some(Self::Accessibility),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum WallpaperFlowStep {
    Source,
    Framing,
    Review,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SettingsAppState {
    active_section: SettingsSection,
    wallpaper_step: WallpaperFlowStep,
    wallpaper_library_open: bool,
    appearance_advanced_open: bool,
}

impl Default for SettingsAppState {
    fn default() -> Self {
        Self {
            active_section: SettingsSection::Personalize,
            wallpaper_step: WallpaperFlowStep::Source,
            wallpaper_library_open: false,
            appearance_advanced_open: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SkinPreset {
    id: &'static str,
    label: &'static str,
    note: &'static str,
}

const SKIN_PRESETS: [SkinPreset; 4] = [
    SkinPreset {
        id: "soft-neumorphic",
        label: "Soft Neumorphic",
        note: "Disciplined low-contrast shell depth with tactile surfaces.",
    },
    SkinPreset {
        id: "modern-adaptive",
        label: "Modern Adaptive",
        note: "Fluent-inspired surfaces with restrained motion and sharper hierarchy.",
    },
    SkinPreset {
        id: "classic-xp",
        label: "Classic XP",
        note: "Glossy nostalgic shell styling with stronger contrast edges.",
    },
    SkinPreset {
        id: "classic-95",
        label: "Classic 95",
        note: "Retro square geometry with compact chrome and minimal depth.",
    },
];

fn wallpaper_step_status(active: WallpaperFlowStep, step: WallpaperFlowStep) -> StepStatus {
    match (active, step) {
        (WallpaperFlowStep::Source, WallpaperFlowStep::Source)
        | (WallpaperFlowStep::Framing, WallpaperFlowStep::Framing)
        | (WallpaperFlowStep::Review, WallpaperFlowStep::Review) => StepStatus::Current,
        (WallpaperFlowStep::Framing, WallpaperFlowStep::Source)
        | (WallpaperFlowStep::Review, WallpaperFlowStep::Source | WallpaperFlowStep::Framing) => {
            StepStatus::Complete
        }
        _ => StepStatus::Pending,
    }
}

#[component]
/// Settings app window contents.
pub fn SettingsApp(
    /// Legacy launch params, retained for compatibility.
    launch_params: Value,
    /// Manager-restored app state payload.
    restored_state: Option<Value>,
    /// Injected desktop services bundle.
    services: Option<AppServices>,
) -> impl IntoView {
    let services = services.expect("settings requires app services");
    let settings_state = create_rw_signal(SettingsAppState::default());
    let search = create_rw_signal(String::new());
    let selected_asset_id = create_rw_signal(String::new());
    let rename_value = create_rw_signal(String::new());
    let tags_value = create_rw_signal(String::new());
    let new_collection_name = create_rw_signal(String::new());

    if let Some(restored_state) = restored_state {
        if let Ok(restored) = serde_json::from_value::<SettingsAppState>(restored_state) {
            settings_state.set(restored);
        }
    }

    if let Some(section) = launch_params
        .get("section")
        .and_then(Value::as_str)
        .and_then(SettingsSection::from_launch_param)
    {
        settings_state.update(|state| state.active_section = section);
    }

    create_effect(move |_| {
        if let Ok(serialized) = serde_json::to_value(settings_state.get()) {
            services.state.persist_window_state(serialized);
        }
    });

    let active_wallpaper = Signal::derive(move || {
        services
            .wallpaper
            .preview
            .get()
            .unwrap_or_else(|| services.wallpaper.current.get())
    });
    let wallpaper_library = Signal::derive(move || services.wallpaper.library.get());
    let selected_asset = Signal::derive(move || {
        wallpaper_library
            .get()
            .assets
            .into_iter()
            .find(|asset| asset.asset_id == selected_asset_id.get())
    });
    let filtered_assets = Signal::derive(move || {
        let query = search.get().trim().to_ascii_lowercase();
        wallpaper_library
            .get()
            .assets
            .into_iter()
            .filter(|asset| {
                if query.is_empty() {
                    return true;
                }
                asset.display_name.to_ascii_lowercase().contains(&query)
                    || asset
                        .tags
                        .iter()
                        .any(|tag| tag.to_ascii_lowercase().contains(&query))
            })
            .collect::<Vec<_>>()
    });
    let theme_skin_id = Signal::derive({
        let services = services.clone();
        move || services.theme.skin_id.get()
    });
    let theme_high_contrast = Signal::derive({
        let services = services.clone();
        move || services.theme.high_contrast.get()
    });
    let theme_reduced_motion = Signal::derive({
        let services = services.clone();
        move || services.theme.reduced_motion.get()
    });

    create_effect(move |_| {
        let library = wallpaper_library.get();
        if selected_asset_id.get_untracked().is_empty() {
            if let Some(asset) = library.assets.first() {
                selected_asset_id.set(asset.asset_id.clone());
                rename_value.set(asset.display_name.clone());
                tags_value.set(asset.tags.join(", "));
            }
            return;
        }

        if let Some(asset) = library
            .assets
            .iter()
            .find(|asset| asset.asset_id == selected_asset_id.get())
        {
            rename_value.set(asset.display_name.clone());
            tags_value.set(asset.tags.join(", "));
        }
    });

    let preview_asset = move |asset: &WallpaperAssetRecord| {
        selected_asset_id.set(asset.asset_id.clone());
        rename_value.set(asset.display_name.clone());
        tags_value.set(asset.tags.join(", "));
        services
            .wallpaper
            .preview(asset_to_config(asset, &active_wallpaper.get_untracked()));
    };
    let preview_mode = move |display_mode: WallpaperDisplayMode| {
        let mut config = active_wallpaper.get_untracked();
        config.display_mode = display_mode;
        services.wallpaper.preview(config);
    };
    let preview_position = move |position: WallpaperPosition| {
        let mut config = active_wallpaper.get_untracked();
        config.position = position;
        services.wallpaper.preview(config);
    };
    let apply_preview = move |_| {
        services.wallpaper.apply_preview();
        settings_state.update(|state| state.wallpaper_step = WallpaperFlowStep::Source);
    };
    let revert_preview = move |_| {
        services.wallpaper.clear_preview();
        settings_state.update(|state| state.wallpaper_step = WallpaperFlowStep::Source);
    };
    let import_wallpaper = move |_| {
        services.wallpaper.import_from_picker(Default::default());
        settings_state.update(|state| state.wallpaper_library_open = true);
    };
    let save_rename = move |_| {
        if let Some(asset) = selected_asset.get_untracked() {
            services
                .wallpaper
                .rename_asset(asset.asset_id, rename_value.get_untracked());
        }
    };
    let save_tags = move |_| {
        if let Some(asset) = selected_asset.get_untracked() {
            let tags = tags_value
                .get_untracked()
                .split(',')
                .map(str::trim)
                .filter(|tag| !tag.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>();
            services.wallpaper.set_tags(asset.asset_id, tags);
        }
    };
    let toggle_favorite = move |_| {
        if let Some(asset) = selected_asset.get_untracked() {
            services
                .wallpaper
                .set_favorite(asset.asset_id, !asset.favorite);
        }
    };
    let delete_asset = move |_| {
        if let Some(asset) = selected_asset.get_untracked() {
            services.wallpaper.delete_asset(asset.asset_id);
            selected_asset_id.set(String::new());
        }
    };
    let create_collection = move |_| {
        let name = new_collection_name.get_untracked();
        if !name.trim().is_empty() {
            services.wallpaper.create_collection(name.trim());
            new_collection_name.set(String::new());
        }
    };

    view! {
        <AppShell>
            <MenuBar role="tablist" aria_label="Settings sections">
                <For
                    each=move || {
                        [
                            SettingsSection::Personalize,
                            SettingsSection::Appearance,
                            SettingsSection::Accessibility,
                        ]
                    }
                    key=|section| *section as u8
                    let:section
                >
                    <Button
                        variant=ButtonVariant::Quiet
                        selected=Signal::derive(move || settings_state.get().active_section == section)
                        role="tab"
                        on_click=Callback::new(move |_| {
                            settings_state.update(|state| state.active_section = section);
                        })
                    >
                        {section.label()}
                    </Button>
                </For>
            </MenuBar>

            <Show when=move || settings_state.get().active_section == SettingsSection::Personalize fallback=|| ()>
                <Surface
                    variant=SurfaceVariant::Muted
                    elevation=Elevation::Inset
                >
                    <StepFlow>
                        <StepFlowHeader
                            title="Personalize your desktop"
                            description="Choose a wallpaper, refine the framing, then review before applying."
                        />

                        <StepFlowStep
                            title="Choose a source"
                            description="Browse the wallpaper library or import a new asset."
                            status=Signal::derive(move || {
                                wallpaper_step_status(
                                    settings_state.get().wallpaper_step,
                                    WallpaperFlowStep::Source,
                                )
                            })
                        >
                            <Panel variant=SurfaceVariant::Standard>
                                <Cluster justify=LayoutJustify::Between>
                                    <Cluster>
                                        <Button
                                            variant=ButtonVariant::Primary
                                            on_click=Callback::new(import_wallpaper)
                                        >
                                            "Import"
                                        </Button>
                                        <Button
                                            variant=ButtonVariant::Quiet
                                            on_click=Callback::new(move |_| {
                                                settings_state.update(|state| {
                                                    state.wallpaper_library_open =
                                                        !state.wallpaper_library_open
                                                });
                                            })
                                        >
                                            {move || if settings_state.get().wallpaper_library_open {
                                                "Hide Library Tools"
                                            } else {
                                                "Show Library Tools"
                                            }}
                                        </Button>
                                    </Cluster>
                                    <TextField
                                        input_type="search"
                                        placeholder="Search wallpapers"
                                        value=Signal::derive(move || search.get())
                                        on_input=Callback::new(move |ev| {
                                            search.set(event_target_value(&ev));
                                        })
                                    />
                                </Cluster>

                                <div>
                                    <WallpaperPreview config=active_wallpaper />
                                </div>

                                <div>
                                    <For
                                        each=move || filtered_assets.get()
                                        key=|asset| asset.asset_id.clone()
                                        let:asset
                                    >
                                        <WallpaperLibraryItem
                                            asset=asset
                                            selected_asset_id=selected_asset_id
                                            on_preview=Callback::new(move |asset| preview_asset(&asset))
                                        />
                                    </For>
                                </div>
                            </Panel>

                            <DisclosurePanel
                                title="Library maintenance"
                                description="Rename assets, update tags, favorites, and collection membership only when needed."
                                expanded=Signal::derive(move || settings_state.get().wallpaper_library_open)
                                on_toggle=Callback::new(move |_| {
                                    settings_state.update(|state| {
                                        state.wallpaper_library_open = !state.wallpaper_library_open
                                    });
                                })
                            >
                                <Show when=move || selected_asset.get().is_some() fallback=|| {
                                    view! { <Text tone=TextTone::Secondary>"Select a wallpaper to manage its metadata."</Text> }
                                }>
                                    <Grid>
                                        <label>
                                            <Text role=TextRole::Label>"Name"</Text>
                                            <TextField
                                                value=Signal::derive(move || rename_value.get())
                                                on_input=Callback::new(move |ev| {
                                                    rename_value.set(event_target_value(&ev));
                                                })
                                            />
                                        </label>
                                        <Button on_click=Callback::new(save_rename)>"Rename"</Button>

                                        <label>
                                            <Text role=TextRole::Label>"Tags"</Text>
                                            <TextField
                                                placeholder="comma, separated, tags"
                                                value=Signal::derive(move || tags_value.get())
                                                on_input=Callback::new(move |ev| {
                                                    tags_value.set(event_target_value(&ev));
                                                })
                                            />
                                        </label>
                                        <Button on_click=Callback::new(save_tags)>"Save Tags"</Button>
                                    </Grid>

                                    <Cluster>
                                        <Button
                                            variant=ButtonVariant::Quiet
                                            on_click=Callback::new(toggle_favorite)
                                        >
                                            {move || if selected_asset.get().map(|asset| asset.favorite).unwrap_or(false) {
                                                "Remove Favorite"
                                            } else {
                                                "Mark Favorite"
                                            }}
                                        </Button>
                                        <Show
                                            when=move || {
                                                selected_asset
                                                    .get()
                                                    .map(|asset| {
                                                        asset.source_kind == WallpaperSourceKind::Imported
                                                    })
                                                    .unwrap_or(false)
                                            }
                                            fallback=|| ()
                                        >
                                            <Button
                                                variant=ButtonVariant::Danger
                                                on_click=Callback::new(delete_asset)
                                            >
                                                "Delete Imported Asset"
                                            </Button>
                                        </Show>
                                    </Cluster>

                                    <Heading role=TextRole::Title>
                                        "Collections"
                                    </Heading>
                                    <div>
                                        <For
                                            each=move || wallpaper_library.get().collections
                                            key=|collection| collection.collection_id.clone()
                                            let:collection
                                        >
                                            <WallpaperCollectionItem
                                                collection=collection
                                                selected_asset=selected_asset
                                                on_toggle=Callback::new(move |collection_id: String| {
                                                    if let Some(asset) = selected_asset.get_untracked() {
                                                        let mut collection_ids = asset.collection_ids.clone();
                                                        if collection_ids.contains(&collection_id)
                                                        {
                                                            collection_ids.retain(|id| *id != collection_id);
                                                        } else {
                                                            collection_ids.push(collection_id);
                                                        }
                                                        services.wallpaper.set_collections(
                                                            asset.asset_id,
                                                            collection_ids,
                                                        );
                                                    }
                                                })
                                            />
                                        </For>
                                    </div>
                                    <Cluster>
                                        <TextField
                                            placeholder="New collection"
                                            value=Signal::derive(move || new_collection_name.get())
                                            on_input=Callback::new(move |ev| {
                                                new_collection_name.set(event_target_value(&ev));
                                            })
                                        />
                                        <Button on_click=Callback::new(create_collection)>
                                            "Create Collection"
                                        </Button>
                                    </Cluster>
                                </Show>
                            </DisclosurePanel>

                            <StepFlowActions>
                                <span></span>
                                <Button
                                    variant=ButtonVariant::Primary
                                    disabled=Signal::derive(move || selected_asset.get().is_none())
                                    on_click=Callback::new(move |_| {
                                        settings_state.update(|state| {
                                            state.wallpaper_step = WallpaperFlowStep::Framing;
                                        });
                                    })
                                >
                                    "Next: Framing"
                                </Button>
                            </StepFlowActions>
                        </StepFlowStep>

                        <StepFlowStep
                            title="Adjust framing"
                            description="Set the display mode and placement before review."
                            status=Signal::derive(move || {
                                wallpaper_step_status(
                                    settings_state.get().wallpaper_step,
                                    WallpaperFlowStep::Framing,
                                )
                            })
                        >
                            <Panel variant=SurfaceVariant::Standard>
                                <Heading role=TextRole::Title>
                                    "Display Mode"
                                </Heading>
                                <div>
                                    <For
                                        each=move || wallpaper_display_modes()
                                        key=|mode| *mode as u8
                                        let:mode
                                    >
                                        <Button
                                            variant=ButtonVariant::Quiet
                                            selected=Signal::derive(move || active_wallpaper.get().display_mode == mode)
                                            on_click=Callback::new(move |_| preview_mode(mode))
                                        >
                                            {wallpaper_display_mode_label(mode)}
                                        </Button>
                                    </For>
                                </div>

                                <Heading role=TextRole::Title>
                                    "Position"
                                </Heading>
                                <div>
                                    <For
                                        each=move || wallpaper_positions()
                                        key=|position| *position as u8
                                        let:position
                                    >
                                        <Button
                                            variant=ButtonVariant::Quiet
                                            selected=Signal::derive(move || active_wallpaper.get().position == position)
                                            on_click=Callback::new(move |_| preview_position(position))
                                        >
                                            {wallpaper_position_label(position)}
                                        </Button>
                                    </For>
                                </div>
                            </Panel>

                            <StepFlowActions>
                                <Button
                                    variant=ButtonVariant::Quiet
                                    on_click=Callback::new(move |_| {
                                        settings_state.update(|state| {
                                            state.wallpaper_step = WallpaperFlowStep::Source;
                                        });
                                    })
                                >
                                    "Back"
                                </Button>
                                <Button
                                    variant=ButtonVariant::Primary
                                    on_click=Callback::new(move |_| {
                                        settings_state.update(|state| {
                                            state.wallpaper_step = WallpaperFlowStep::Review;
                                        });
                                    })
                                >
                                    "Next: Review"
                                </Button>
                            </StepFlowActions>
                        </StepFlowStep>

                        <StepFlowStep
                            title="Review and apply"
                            description="Confirm the selected wallpaper and commit the staged preview when ready."
                            status=Signal::derive(move || {
                                wallpaper_step_status(
                                    settings_state.get().wallpaper_step,
                                    WallpaperFlowStep::Review,
                                )
                            })
                        >
                            <Panel variant=SurfaceVariant::Standard>
                                <div>
                                    <WallpaperPreview config=active_wallpaper />
                                </div>
                                <Show when=move || selected_asset.get().is_some() fallback=|| ()>
                                    <Stack gap=LayoutGap::Sm>
                                        <Text role=TextRole::Label>"Selected wallpaper"</Text>
                                        <Text>{move || selected_asset.get().map(|asset| asset.display_name).unwrap_or_default()}</Text>
                                        <Text tone=TextTone::Secondary>
                                            {move || {
                                                let config = active_wallpaper.get();
                                                format!(
                                                    "{} / {}",
                                                    wallpaper_display_mode_label(config.display_mode),
                                                    wallpaper_position_label(config.position),
                                                )
                                            }}
                                        </Text>
                                    </Stack>
                                </Show>
                            </Panel>

                            <StepFlowActions>
                                <Cluster>
                                    <Button
                                        variant=ButtonVariant::Quiet
                                        on_click=Callback::new(move |_| {
                                            settings_state.update(|state| {
                                                state.wallpaper_step = WallpaperFlowStep::Framing;
                                            });
                                        })
                                    >
                                        "Back"
                                    </Button>
                                    <Button
                                        variant=ButtonVariant::Quiet
                                        disabled=Signal::derive(move || services.wallpaper.preview.get().is_none())
                                        on_click=Callback::new(revert_preview)
                                    >
                                        "Cancel Draft"
                                    </Button>
                                </Cluster>
                                <Button
                                    variant=ButtonVariant::Primary
                                    disabled=Signal::derive(move || services.wallpaper.preview.get().is_none())
                                    on_click=Callback::new(apply_preview)
                                >
                                    "Apply Wallpaper"
                                </Button>
                            </StepFlowActions>
                        </StepFlowStep>
                    </StepFlow>
                </Surface>
            </Show>

            <Show when=move || settings_state.get().active_section == SettingsSection::Appearance fallback=|| ()>
                <Surface
                    variant=SurfaceVariant::Muted
                    elevation=Elevation::Inset
                >
                    <Stack gap=LayoutGap::Lg>
                        <Panel variant=SurfaceVariant::Standard>
                            <Heading role=TextRole::Title>"Choose a shell skin"</Heading>
                            <Text tone=TextTone::Secondary>
                                "Use the curated shell presets below. Advanced tuning stays tucked away unless you need it."
                            </Text>
                            <div>
                                <For
                                    each=move || SKIN_PRESETS.into_iter()
                                    key=|preset| preset.id
                                    let:preset
                                >
                                    <Button
                                        variant=ButtonVariant::Quiet
                                        selected=Signal::derive(move || theme_skin_id.get() == preset.id)
                                        on_click=Callback::new(move |_| services.theme.set_skin(preset.id))
                                    >
                                        <span>{preset.label}</span>
                                        <span>{preset.note}</span>
                                    </Button>
                                </For>
                            </div>
                        </Panel>

                        <DisclosurePanel
                            title="Advanced appearance details"
                            description="Keep the shell calm by default. Open this only when you need to inspect the current skin state."
                            expanded=Signal::derive(move || settings_state.get().appearance_advanced_open)
                            on_toggle=Callback::new(move |_| {
                                settings_state.update(|state| {
                                    state.appearance_advanced_open = !state.appearance_advanced_open
                                });
                            })
                        >
                            <Cluster>
                                <Text role=TextRole::Label>"Active skin"</Text>
                                <Text>{move || theme_skin_id.get()}</Text>
                            </Cluster>
                        </DisclosurePanel>
                    </Stack>
                </Surface>
            </Show>

            <Show when=move || settings_state.get().active_section == SettingsSection::Accessibility fallback=|| ()>
                <Surface
                    variant=SurfaceVariant::Muted
                    elevation=Elevation::Inset
                >
                    <Stack gap=LayoutGap::Md>
                        <Panel variant=SurfaceVariant::Standard>
                            <Heading role=TextRole::Title>"Visibility"</Heading>
                            <ToggleRow
                                title="High contrast"
                                description="Increase separation between borders, text, and focus states."
                                checked=theme_high_contrast
                            >
                                <CheckboxField
                                    aria_label="High contrast"
                                    checked=theme_high_contrast
                                    on_change=Callback::new(move |ev| {
                                        services.theme.set_high_contrast(event_target_checked(&ev))
                                    })
                                />
                            </ToggleRow>
                        </Panel>

                        <Panel variant=SurfaceVariant::Standard>
                            <Heading role=TextRole::Title>"Motion"</Heading>
                            <ToggleRow
                                title="Reduced motion"
                                description="Replace animated wallpaper playback and shorten non-essential transitions."
                                checked=theme_reduced_motion
                            >
                                <CheckboxField
                                    aria_label="Reduced motion"
                                    checked=theme_reduced_motion
                                    on_change=Callback::new(move |ev| {
                                        services.theme.set_reduced_motion(event_target_checked(&ev))
                                    })
                                />
                            </ToggleRow>
                        </Panel>
                    </Stack>
                </Surface>
            </Show>

            <StatusBar>
                <StatusBarItem>{move || format!("Skin: {}", theme_skin_id.get())}</StatusBarItem>
                <StatusBarItem>
                    {move || {
                        let config = active_wallpaper.get();
                        match config.selection {
                            WallpaperSelection::BuiltIn { wallpaper_id } => {
                                format!("Wallpaper: {wallpaper_id}")
                            }
                            WallpaperSelection::Imported { asset_id } => {
                                format!("Wallpaper: {asset_id}")
                            }
                        }
                    }}
                </StatusBarItem>
                <StatusBarItem>{move || format!("Library assets: {}", wallpaper_library.get().assets.len())}</StatusBarItem>
            </StatusBar>
        </AppShell>
    }
}

fn asset_to_config(asset: &WallpaperAssetRecord, current: &WallpaperConfig) -> WallpaperConfig {
    let animation = match asset.media_kind {
        WallpaperMediaKind::AnimatedImage | WallpaperMediaKind::Video => {
            WallpaperAnimationPolicy::LoopMuted
        }
        _ => WallpaperAnimationPolicy::None,
    };
    WallpaperConfig {
        selection: match asset.source_kind {
            WallpaperSourceKind::BuiltIn => WallpaperSelection::BuiltIn {
                wallpaper_id: asset.asset_id.clone(),
            },
            WallpaperSourceKind::Imported => WallpaperSelection::Imported {
                asset_id: asset.asset_id.clone(),
            },
        },
        display_mode: current.display_mode,
        position: current.position,
        animation,
    }
}

fn wallpaper_display_modes() -> [WallpaperDisplayMode; 5] {
    [
        WallpaperDisplayMode::Fill,
        WallpaperDisplayMode::Fit,
        WallpaperDisplayMode::Stretch,
        WallpaperDisplayMode::Tile,
        WallpaperDisplayMode::Center,
    ]
}

fn wallpaper_display_mode_label(mode: WallpaperDisplayMode) -> &'static str {
    match mode {
        WallpaperDisplayMode::Fill => "Fill",
        WallpaperDisplayMode::Fit => "Fit",
        WallpaperDisplayMode::Stretch => "Stretch",
        WallpaperDisplayMode::Tile => "Tile",
        WallpaperDisplayMode::Center => "Center",
    }
}

fn wallpaper_positions() -> [WallpaperPosition; 9] {
    [
        WallpaperPosition::TopLeft,
        WallpaperPosition::Top,
        WallpaperPosition::TopRight,
        WallpaperPosition::Left,
        WallpaperPosition::Center,
        WallpaperPosition::Right,
        WallpaperPosition::BottomLeft,
        WallpaperPosition::Bottom,
        WallpaperPosition::BottomRight,
    ]
}

fn wallpaper_position_label(position: WallpaperPosition) -> &'static str {
    match position {
        WallpaperPosition::TopLeft => "Top left",
        WallpaperPosition::Top => "Top",
        WallpaperPosition::TopRight => "Top right",
        WallpaperPosition::Left => "Left",
        WallpaperPosition::Center => "Center",
        WallpaperPosition::Right => "Right",
        WallpaperPosition::BottomLeft => "Bottom left",
        WallpaperPosition::Bottom => "Bottom",
        WallpaperPosition::BottomRight => "Bottom right",
    }
}

fn asset_label_prefix(asset: &WallpaperAssetRecord) -> &'static str {
    match asset.source_kind {
        WallpaperSourceKind::BuiltIn => "Built-in",
        WallpaperSourceKind::Imported => "Imported",
    }
}

#[component]
fn WallpaperLibraryItem(
    asset: WallpaperAssetRecord,
    selected_asset_id: RwSignal<String>,
    on_preview: Callback<WallpaperAssetRecord>,
) -> impl IntoView {
    let asset_id = asset.asset_id.clone();
    let asset_for_click = asset.clone();
    let display_name = asset.display_name.clone();
    let meta = format!(
        "{}{}{}",
        asset_label_prefix(&asset),
        if asset.favorite { " | favorite" } else { "" },
        if asset.tags.is_empty() {
            String::new()
        } else {
            format!(" | {}", asset.tags.join(", "))
        }
    );

    view! {
        <Button
            variant=ButtonVariant::Quiet
            selected=Signal::derive(move || selected_asset_id.get() == asset_id)
            on_click=Callback::new(move |_| on_preview.call(asset_for_click.clone()))
        >
            <span>
                <WallpaperThumb asset=asset.clone() />
            </span>
            <span>
                <span>{display_name}</span>
                <span>{meta}</span>
            </span>
        </Button>
    }
}

#[component]
fn WallpaperCollectionItem(
    collection: WallpaperCollection,
    selected_asset: Signal<Option<WallpaperAssetRecord>>,
    on_toggle: Callback<String>,
) -> impl IntoView {
    let collection_id = collection.collection_id.clone();

    view! {
        <Button
            variant=ButtonVariant::Quiet
            selected=Signal::derive(move || {
                selected_asset
                    .get()
                    .map(|asset| asset.collection_ids.contains(&collection_id))
                    .unwrap_or(false)
            })
            on_click=Callback::new(move |_| on_toggle.call(collection.collection_id.clone()))
        >
            <span>{collection.display_name}</span>
        </Button>
    }
}

#[component]
fn WallpaperThumb(asset: WallpaperAssetRecord) -> impl IntoView {
    view! {
        <img src=asset.poster_url.unwrap_or(asset.primary_url) alt=asset.display_name />
    }
}

#[component]
fn WallpaperPreview(config: Signal<WallpaperConfig>) -> impl IntoView {
    view! {
        <div>
            {move || match config.get().selection {
                WallpaperSelection::BuiltIn { wallpaper_id } => view! {
                    <span>{format!("Built-in: {wallpaper_id}")}</span>
                }
                .into_view(),
                WallpaperSelection::Imported { asset_id } => view! {
                    <span>{format!("Imported: {asset_id}")}</span>
                }
                .into_view(),
            }}
            <small>
                {move || format!(
                    "{} / {}",
                    wallpaper_display_mode_label(config.get().display_mode),
                    wallpaper_position_label(config.get().position)
                )}
            </small>
        </div>
    }
}
