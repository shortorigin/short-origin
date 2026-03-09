# Plugin Application Model

Origin applications are governed plugin modules composed on top of the platform substrate. The
first version is static and build-time registered: plugin manifests are committed in-repo and wired
into the shared shell/runtime through typed contracts.

## Plugin Identity

- Every plugin has a stable `plugin_id`.
- The current v1 model is one plugin module per UI application entry. For built-in applications,
  `plugin_id` and `app_id` are the same canonical dotted identifier.
- Plugin manifests are canonical contract artifacts. The v1 schema lives at
  `schemas/contracts/v1/plugin-module-v1.json`.

## Lifecycle

1. A plugin manifest declares identity, entrypoint, requested capabilities, runtime targets, and
   contract dependencies.
2. Build-time validation parses the manifest and rejects missing or incompatible fields.
3. The shell runtime registers the plugin into the app catalog.
4. At runtime, the shell mounts the plugin through typed context and capability injection.
5. The runtime may grant, withhold, or partially satisfy requested capabilities depending on host
   availability and policy.

## UI Registration and Route Contribution

- Plugins contribute UI through a declared `ui.entry` and a list of `ui.routes`.
- Route contributions are declarative; plugin modules do not patch the core shell router through
  private imports.
- Launcher and desktop visibility are declared in the manifest and consumed by the runtime registry.

## Capability and Contract Model

Plugin manifests declare:

- requested UI/runtime capabilities;
- required platform contracts;
- service contract dependencies;
- workflow contract dependencies;
- host requirements;
- runtime targets (`pwa`, `tauri`);
- permissions and launcher/window defaults.

Plugins may only use contracts already published by `schemas/` and platform SDK/runtime surfaces.
They must not import private shell, service, workflow, or infrastructure internals.

## Permissions and Host Requirements

- Host requirements describe which capabilities must be present or may be optional.
- Runtime targets describe where the plugin is supported without changing its platform identity.
- Tauri-only capabilities are additive enhancements over the same plugin surface, not a separate
  desktop-only application fork.

## Non-Goals for V1

- no remote or dynamic plugin loading;
- no ungoverned runtime discovery outside build-time registration;
- no bypass path around the platform shell, SDK, or `schemas/` contracts.
