//! DEC-029 / SECURITY gate: the desktop crate MUST NOT contain a license (or update)
//! signing private key, nor any hardcoded 32-byte Ed25519 private-key literal.
//!
//! This runs as an integration test (and is suitable for CI). It greps the Rust
//! source tree for forbidden patterns and fails the build if any are found.

use std::path::Path;

fn src_dir() -> std::path::PathBuf {
    // tests/ is a sibling of src/ in the crate root.
    let manifest = env!("CARGO_MANIFEST_DIR");
    Path::new(manifest).join("src")
}

#[test]
fn no_private_signing_key_in_desktop() {
    let dir = src_dir();
    assert!(dir.exists(), "src dir missing: {:?}", dir);

    let patterns = [
        "mock_server_private_key",
        "let key_bytes: [u8; 32] =",
        "let mock_token",
    ];

    // Use git grep if available (repo root), else fall back to a recursive ripgrep-free walk.
    for pat in patterns {
        let found = grep_fallback(&dir, pat);
        assert!(
            !found,
            "FORBIDDEN pattern found in desktop crate source: '{}'. \
             DEC-029 forbids private signing keys / Postgres mock paths in the desktop app.",
            pat
        );
    }
}

/// Recursive, dependency-free grep over .rs files.
fn grep_fallback(dir: &Path, needle: &str) -> bool {
    let mut stack = vec![dir.to_path_buf()];
    while let Some(cur) = stack.pop() {
        let entries = match std::fs::read_dir(&cur) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Skip target/ build artifacts.
                if path.file_name().map(|n| n == "target").unwrap_or(false) {
                    continue;
                }
                stack.push(path);
            } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if content.contains(needle) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

#[test]
fn no_32_byte_ed25519_key_literal() {
    // Heuristic: a suspected private-key literal in src/ is forbidden by DEC-029.
    // Scan only src/ so this test file itself (which mentions the words) is excluded.
    let dir = src_dir();
    let found = grep_fallback(&dir, "signing_key_bytes")
        || grep_fallback(&dir, "private_key_bytes")
        || grep_fallback(&dir, "let key_bytes: [u8; 32] = [");
    assert!(
        !found,
        "Suspected 32-byte private-key literal found in desktop src/. \
         DEC-029: private signing key must never live in the desktop crate."
    );
}
