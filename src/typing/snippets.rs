use std::path::{Path, PathBuf};

/// Code snippets shipped with the binary, used as a typing source and as the
/// fallback when no per-user snippet files exist.
pub const SNIPPETS: &[(&str, &str)] = &[
    (
        "rust",
        "fn greet(name: &str) -> String {\n    format!(\"hello, {name}\")\n}\n\nfn main() {\n    let who = \"world\";\n    println!(\"{}\", greet(who));\n}\n",
    ),
    (
        "python",
        "def fib(n: int) -> int:\n    if n < 2:\n        return n\n    return fib(n - 1) + fib(n - 2)\n\nfor i in range(10):\n    print(fib(i))\n",
    ),
    (
        "javascript",
        "function debounce(fn, wait) {\n  let timer = null;\n  return function (...args) {\n    clearTimeout(timer);\n    timer = setTimeout(() => fn(...args), wait);\n  };\n}\n",
    ),
];

fn assets_snippets_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/snippets")
}

fn read_snippet(dir: &Path, name: &str) -> Option<String> {
    std::fs::read_to_string(dir.join(format!("{name}.txt"))).ok()
}

/// Every selectable snippet: embedded defaults first, then any per-user files
/// found under `assets/snippets/`.
pub fn available_snippets() -> Vec<String> {
    let mut names: Vec<String> = SNIPPETS
        .iter()
        .map(|(name, _)| (*name).to_owned())
        .collect();
    let dir = assets_snippets_dir();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return names;
    };
    let mut found: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "txt"))
        .filter_map(|path| {
            let stem = path.file_stem()?.to_string_lossy().into_owned();
            Some(stem)
        })
        .filter(|stem| !names.iter().any(|name| name.eq_ignore_ascii_case(stem)))
        .collect();
    found.sort();
    names.extend(found);
    names
}

/// The snippet text for `name`: a per-user file wins, otherwise the embedded
/// default of the same name, otherwise the first embedded snippet.
pub fn snippet_source(name: &str) -> String {
    let dir = assets_snippets_dir();
    if let Some(content) = read_snippet(&dir, name) {
        return content;
    }
    SNIPPETS
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name))
        .map_or_else(
            || SNIPPETS[0].1.to_owned(),
            |(_, content)| (*content).to_owned(),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_snippets_are_present_and_multi_line() {
        assert_eq!(SNIPPETS.len(), 3);
        for (_, content) in SNIPPETS {
            assert!(content.lines().count() >= 5, "snippet too short");
            assert!(content.contains('\n'));
        }
    }

    #[test]
    fn snippet_source_reads_by_name() {
        let rust = snippet_source("Rust"); // case-insensitive lookup
        assert!(rust.starts_with("fn greet"));
        assert!(rust.contains("println!"));

        let python = snippet_source("python");
        assert!(python.contains("def fib"));

        let javascript = snippet_source("javascript");
        assert!(javascript.contains("debounce"));
    }

    #[test]
    fn unknown_name_falls_back_to_first_snippet() {
        assert_eq!(snippet_source("nope"), SNIPPETS[0].1);
    }

    #[test]
    fn asset_files_are_listed_after_embedded() {
        let names = available_snippets();
        assert_eq!(names.first().map(String::as_str), Some("rust"));
        assert!(names.iter().any(|name| name == "javascript"));
    }
}
