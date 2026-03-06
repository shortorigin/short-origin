use desktop_tauri::explorer::ScopedExplorerFs;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("{prefix}_{}_{}", process::id(), nanos));
    fs::create_dir_all(&path).expect("create temp dir");
    path
}

#[test]
fn scoped_fs_normalizes_and_stays_within_root() {
    let root = temp_dir("explorer_scoped_root");
    let fscope = ScopedExplorerFs::from_root(&root).expect("init scoped fs");

    fscope.create_dir("/notes").expect("create notes dir");
    fscope
        .write_text_file("/docs/../notes/readme.txt", "hello")
        .expect("write in root");
    let read = fscope
        .read_text_file("/notes/readme.txt")
        .expect("read in root");
    assert_eq!(read.path, "/notes/readme.txt");
    assert_eq!(read.text, "hello");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn scoped_fs_rejects_root_delete_and_root_write() {
    let root = temp_dir("explorer_scoped_root_reject_root");
    let fscope = ScopedExplorerFs::from_root(&root).expect("init scoped fs");

    let delete_err = fscope
        .delete("/", true)
        .expect_err("root delete should be rejected");
    assert_eq!(delete_err, "cannot delete explorer root");

    let write_err = fscope
        .write_text_file("/", "x")
        .expect_err("root write should be rejected");
    assert_eq!(write_err, "cannot write to explorer root");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn scoped_fs_error_contract_for_invalid_or_unsupported_operations() {
    let root = temp_dir("explorer_scoped_error_contract");
    let fscope = ScopedExplorerFs::from_root(&root).expect("init scoped fs");

    fscope
        .write_text_file("/file.txt", "x")
        .expect("create baseline file");
    fscope.create_dir("/dir").expect("create baseline dir");

    let cases = [
        (
            "delete_root",
            fscope.delete("/", true).err(),
            "cannot delete explorer root",
        ),
        (
            "write_root",
            fscope.write_text_file("/", "x").err(),
            "cannot write to explorer root",
        ),
        (
            "list_non_dir",
            fscope.list_dir("/file.txt").err(),
            "path `/file.txt` is not a directory",
        ),
        (
            "read_non_file",
            fscope.read_text_file("/dir").err(),
            "path `/dir` is not a file",
        ),
    ];

    for (label, got, expected) in cases {
        let got = got.unwrap_or_else(|| panic!("{label} should fail"));
        assert_eq!(got, expected, "{label} mismatch");
    }

    let _ = fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn scoped_fs_rejects_symlink_escape_on_existing_paths() {
    use std::os::unix::fs::symlink;

    let root = temp_dir("explorer_scoped_symlink_existing_root");
    let outside = temp_dir("explorer_scoped_symlink_existing_outside");
    let outside_file = outside.join("outside.txt");
    fs::write(&outside_file, "outside").expect("write outside file");

    let link_path = root.join("escape.txt");
    symlink(&outside_file, &link_path).expect("create file symlink");

    let fscope = ScopedExplorerFs::from_root(&root).expect("init scoped fs");
    let err = fscope
        .stat("/escape.txt")
        .expect_err("symlink escape should fail");
    assert!(
        err.contains("outside scoped explorer root"),
        "unexpected error: {err}"
    );

    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(outside);
}

#[cfg(unix)]
#[test]
fn scoped_fs_rejects_symlink_escape_on_parent_for_writes() {
    use std::os::unix::fs::symlink;

    let root = temp_dir("explorer_scoped_symlink_parent_root");
    let outside = temp_dir("explorer_scoped_symlink_parent_outside");

    let link_dir = root.join("linked");
    symlink(&outside, &link_dir).expect("create dir symlink");

    let fscope = ScopedExplorerFs::from_root(&root).expect("init scoped fs");
    let err = fscope
        .write_text_file("/linked/new.txt", "x")
        .expect_err("symlink-parent escape should fail");
    assert!(
        err.contains("parent resolves outside scoped explorer root"),
        "unexpected error: {err}"
    );

    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(outside);
}
