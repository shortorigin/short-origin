use std::fs;
use std::path::{Path, PathBuf};

use lattice_config::{
    LatticeConfigV1, finance_service_component_binding_with_artifact,
    rollout_target_for_environment, treasury_disbursement_component_binding_with_artifact,
};

const DEFAULT_REGISTRY: &str = "ghcr.io/shortorigin";
const PENDING_DIGEST: &str = "sha256:pending";
const LATTICE_NAME: &str = "institutional-lattice";

pub fn run(args: Vec<String>) -> Result<(), String> {
    match args.split_first() {
        Some((command, rest)) if command == "render-components" => render_components(rest),
        Some((command, rest)) if command == "render-manifest" => render_manifest(rest),
        Some((command, _)) => Err(format!("unknown delivery xtask command `{command}`")),
        None => Err(help()),
    }
}

fn render_components(args: &[String]) -> Result<(), String> {
    let mut environment = None;
    let mut output_dir = None;
    let mut registry = DEFAULT_REGISTRY.to_string();
    let mut tag = None;
    let mut index = 0usize;

    while index < args.len() {
        match args[index].as_str() {
            "--environment" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --environment".to_owned());
                };
                environment = Some(parse_environment(value)?);
                index += 2;
            }
            "--output-dir" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --output-dir".to_owned());
                };
                output_dir = Some(PathBuf::from(value));
                index += 2;
            }
            "--registry" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --registry".to_owned());
                };
                registry = normalize_registry(value);
                index += 2;
            }
            "--tag" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --tag".to_owned());
                };
                tag = Some(value.clone());
                index += 2;
            }
            other => return Err(format!("unknown render-components argument `{other}`")),
        }
    }

    let environment = environment.ok_or_else(|| "missing --environment".to_owned())?;
    let output_dir = output_dir.ok_or_else(|| "missing --output-dir".to_owned())?;
    let tag = tag.ok_or_else(|| "missing --tag".to_owned())?;
    fs::create_dir_all(&output_dir).map_err(|error| {
        format!(
            "failed to create output directory `{}`: {error}",
            output_dir.display()
        )
    })?;

    write_json(
        &output_dir.join("finance-service.json"),
        &finance_service_component_binding_with_artifact(
            format!("{registry}/finance-service:{tag}"),
            PENDING_DIGEST,
            environment,
        ),
    )?;
    write_json(
        &output_dir.join("treasury-disbursement.json"),
        &treasury_disbursement_component_binding_with_artifact(
            format!("{registry}/treasury-disbursement:{tag}"),
            PENDING_DIGEST,
            environment,
        ),
    )?;
    Ok(())
}

fn render_manifest(args: &[String]) -> Result<(), String> {
    let mut environment = None;
    let mut finance_ref = None;
    let mut finance_digest = None;
    let mut treasury_ref = None;
    let mut treasury_digest = None;
    let mut output = None;
    let mut index = 0usize;

    while index < args.len() {
        match args[index].as_str() {
            "--environment" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --environment".to_owned());
                };
                environment = Some(parse_environment(value)?);
                index += 2;
            }
            "--finance-ref" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --finance-ref".to_owned());
                };
                finance_ref = Some(value.clone());
                index += 2;
            }
            "--finance-digest" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --finance-digest".to_owned());
                };
                finance_digest = Some(value.clone());
                index += 2;
            }
            "--treasury-ref" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --treasury-ref".to_owned());
                };
                treasury_ref = Some(value.clone());
                index += 2;
            }
            "--treasury-digest" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --treasury-digest".to_owned());
                };
                treasury_digest = Some(value.clone());
                index += 2;
            }
            "--output" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --output".to_owned());
                };
                output = Some(PathBuf::from(value));
                index += 2;
            }
            other => return Err(format!("unknown render-manifest argument `{other}`")),
        }
    }

    let environment = environment.ok_or_else(|| "missing --environment".to_owned())?;
    let output = output.ok_or_else(|| "missing --output".to_owned())?;
    let manifest = LatticeConfigV1 {
        lattice_name: LATTICE_NAME.to_string(),
        rollout: rollout_target_for_environment(environment),
        components: vec![
            finance_service_component_binding_with_artifact(
                finance_ref.ok_or_else(|| "missing --finance-ref".to_owned())?,
                finance_digest.ok_or_else(|| "missing --finance-digest".to_owned())?,
                environment,
            ),
            treasury_disbursement_component_binding_with_artifact(
                treasury_ref.ok_or_else(|| "missing --treasury-ref".to_owned())?,
                treasury_digest.ok_or_else(|| "missing --treasury-digest".to_owned())?,
                environment,
            ),
        ],
    };
    write_json(&output, &manifest)
}

fn parse_environment(value: &str) -> Result<&str, String> {
    match value {
        "dev" | "stage" | "prod" => Ok(value),
        other => Err(format!(
            "unsupported environment `{other}`; expected one of dev, stage, prod"
        )),
    }
}

fn normalize_registry(value: &str) -> String {
    value.trim_end_matches('/').to_string()
}

fn write_json(path: &Path, value: &impl serde::Serialize) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create parent directory `{}`: {error}",
                parent.display()
            )
        })?;
    }
    let json = serde_json::to_string_pretty(value)
        .map_err(|error| format!("failed to serialize JSON for `{}`: {error}", path.display()))?;
    fs::write(path, format!("{json}\n"))
        .map_err(|error| format!("failed to write `{}`: {error}", path.display()))
}

fn help() -> String {
    "usage: cargo xtask delivery <render-components|render-manifest> ...".to_owned()
}

#[cfg(test)]
mod tests {
    use super::{normalize_registry, parse_environment};
    use lattice_config::rollout_target_for_environment;

    #[test]
    fn parse_environment_accepts_all_delivery_targets() {
        assert_eq!(parse_environment("dev").expect("dev"), "dev");
        assert_eq!(parse_environment("stage").expect("stage"), "stage");
        assert_eq!(parse_environment("prod").expect("prod"), "prod");
    }

    #[test]
    fn normalize_registry_trims_trailing_slash() {
        assert_eq!(
            normalize_registry("ghcr.io/shortorigin/"),
            "ghcr.io/shortorigin"
        );
    }

    #[test]
    fn rollout_target_tracks_environment_name() {
        let target = rollout_target_for_environment("stage");
        assert_eq!(target.environment, "stage");
        assert_eq!(target.namespace, "stage");
        assert_eq!(target.policy_group, "origin-stage");
    }
}
