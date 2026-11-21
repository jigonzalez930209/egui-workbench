//! The Lucide icon set, embedded.
//!
//! Lucide is ISC licensed; see `assets/lucide-icons/LICENSE`. The SVGs use
//! `currentColor`; the loader replaces it with white so the text-tint pipeline in
//! [`crate::Icons`] works unchanged.

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets/lucide-icons"]
struct Assets;

/// The sorted names of every bundled Lucide icon.
pub fn names() -> Vec<String> {
    let mut names: Vec<String> = Assets::iter()
        .filter_map(|path| path.strip_suffix(".svg").map(str::to_owned))
        .collect();
    names.sort();
    names
}

/// Icons matching `query`, best match first.
///
/// Exact substring matches rank before subsequence matches, so `tarsh` still finds
/// `trash-2` while `trash` ranks it first. An empty query returns every name.
pub fn search(query: &str) -> Vec<String> {
    let query = query.to_lowercase();
    let mut matches: Vec<(usize, String)> = names()
        .into_iter()
        .filter_map(|name| score(&query, &name).map(|score| (score, name)))
        .collect();
    matches.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    matches.into_iter().map(|(_, name)| name).collect()
}

fn score(query: &str, name: &str) -> Option<usize> {
    if query.is_empty() {
        return Some(0);
    }
    if let Some(position) = name.find(query) {
        return Some(position);
    }
    let mut chars = name.chars();
    let mut skips = 0;
    let mut subsequence = true;
    for query_char in query.chars() {
        loop {
            let Some(name_char) = chars.next() else {
                subsequence = false;
                break;
            };
            if name_char == query_char {
                break;
            }
            skips += 1;
        }
        if !subsequence {
            break;
        }
    }
    if subsequence {
        return Some(1000 + skips);
    }
    let limit = (query.chars().count() / 3).max(2);
    let distance = name
        .split('-')
        .map(|part| levenshtein(query, part))
        .min()
        .unwrap_or(usize::MAX);
    (distance <= limit).then_some(2000 + distance)
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    let mut current = vec![0; b.len() + 1];
    for (i, a_char) in a.iter().enumerate() {
        current[0] = i + 1;
        for (j, b_char) in b.iter().enumerate() {
            let cost = usize::from(a_char != b_char);
            current[j + 1] = (previous[j] + cost).min(previous[j + 1] + 1).min(current[j] + 1);
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[b.len()]
}

/// The SVG bytes of one icon, with `currentColor` made white.
pub(crate) fn svg(name: &str) -> Option<Vec<u8>> {
    let file = Assets::get(&format!("{name}.svg"))?;
    let text = String::from_utf8_lossy(&file.data);
    Some(text.replace("currentColor", "#ffffff").into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_ranks_exact_substrings_and_survives_typos() {
        let trash = search("trash");
        assert_eq!(trash.first().map(String::as_str), Some("trash"));
        assert!(trash.iter().any(|name| name == "trash-2"));
        assert!(search("trsh").iter().any(|name| name == "trash-2"));
        assert!(search("hreat").iter().any(|name| name == "heart"));
        assert!(search("zzzz-not-an-icon").is_empty());
        assert_eq!(search("").len(), names().len());
    }
}
