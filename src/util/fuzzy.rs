use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};

/// Fuzzy filter items by query, returning matched indices in order of relevance.
pub fn fuzzy_filter_indices<T>(
    query: &str,
    items: &[T],
    name_of: impl Fn(&T) -> &str,
) -> Vec<usize> {
    if query.is_empty() {
        return (0..items.len()).collect();
    }

    let config = Config::DEFAULT;
    let mut matcher = Matcher::new(config);
    let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);

    let mut results: Vec<(usize, u32)> = items
        .iter()
        .enumerate()
        .filter_map(|(idx, item)| {
            let name = name_of(item);
            let haystack = Utf32Str::Ascii(name.as_bytes());
            let score = pattern.score(haystack, &mut matcher)?;
            Some((idx, score))
        })
        .collect();

    // Sort by score (descending), then by original index (ascending)
    results.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    results.into_iter().map(|(idx, _)| idx).collect()
}

/// Wrap-around selection movement helper.
/// Returns the new index after moving by `delta`.
#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
pub const fn move_selection(current: usize, len: usize, delta: isize) -> usize {
    if len == 0 {
        return 0;
    }

    let current_signed = current as isize;
    let len_signed = len as isize;
    let new_pos = (current_signed + delta) % len_signed;
    ((new_pos + len_signed) % len_signed) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_filter_all_empty_query() {
        let items = vec!["foo", "bar", "baz"];
        let result = fuzzy_filter_indices("", &items, |s| s);
        assert_eq!(result, vec![0, 1, 2]);
    }

    #[test]
    fn test_fuzzy_filter_matching() {
        let items = vec!["main.rs", "lib.rs", "test.rs"];
        let result = fuzzy_filter_indices("main", &items, |s| s);
        assert!(result.contains(&0));
    }

    #[test]
    fn test_move_selection_forward() {
        assert_eq!(move_selection(0, 3, 1), 1);
        assert_eq!(move_selection(1, 3, 1), 2);
    }

    #[test]
    fn test_move_selection_wrap_forward() {
        assert_eq!(move_selection(2, 3, 1), 0);
    }

    #[test]
    fn test_move_selection_backward() {
        assert_eq!(move_selection(1, 3, -1), 0);
    }

    #[test]
    fn test_move_selection_wrap_backward() {
        assert_eq!(move_selection(0, 3, -1), 2);
    }

    #[test]
    fn test_move_selection_empty() {
        assert_eq!(move_selection(0, 0, 1), 0);
    }
}
