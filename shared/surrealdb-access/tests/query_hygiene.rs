use std::fs;
use std::path::{Path, PathBuf};

fn collect_rust_files(root: &Path, output: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.ends_with("target") || path.ends_with("node_modules") || path.ends_with(".git")
            {
                continue;
            }
            collect_rust_files(&path, output);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            output.push(path);
        }
    }
}

#[test]
fn repo_avoids_direct_surreal_query_calls_outside_shared_access_layer() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
        .to_path_buf();
    let allowed_root = repo_root.join("shared").join("surrealdb-access");
    let mut files = Vec::new();
    collect_rust_files(&repo_root, &mut files);

    let offenders = files
        .into_iter()
        .filter(|path| !path.starts_with(&allowed_root))
        .filter_map(|path| {
            let raw = fs::read_to_string(&path).ok()?;
            raw.contains(".query(").then_some(path)
        })
        .collect::<Vec<_>>();

    assert!(
        offenders.is_empty(),
        "direct SurrealDB query calls must stay inside shared/surrealdb-access: {offenders:?}"
    );
}
