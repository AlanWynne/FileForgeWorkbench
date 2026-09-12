//! Glob and brace-expansion matching for EditorConfig section patterns.
//!
//! Self-contained str/byte matching engine used by `matches_pattern`.

/// Match a file path against an EditorConfig glob pattern.
///
/// `pattern` is the glob from the section header (e.g., `*.rs`, `lib/**/*.rs`).
/// `filename` is the relative path of the file from the .editorconfig directory.
///
/// EditorConfig matching rules:
/// - Patterns without `/` are matched only against the file's basename
/// - Patterns with `/` are matched against the full relative path
/// - Matching is case-sensitive
///
/// Supported glob features:
/// - `*` -- matches any string of characters except `/`
/// - `**` -- matches any string of characters including `/`
/// - `?` -- matches any single character except `/`
/// - `[abc]` -- character class
/// - `[!abc]` or `[^abc]` -- negated character class
/// - `{s1,s2,s3}` -- brace expansion (matches any of the alternatives)
/// - `{num1..num2}` -- integer range (matches any integer in the range)
pub fn matches_pattern(pattern: &str, filename: &str) -> bool {
    // Determine whether to match against basename only or full path.
    // If pattern contains a `/`, match against the full relative path.
    // Otherwise, match only against the filename's basename.
    let target = if pattern.contains('/') {
        filename
    } else {
        // Extract basename (last component after final `/`)
        filename.rsplit('/').next().unwrap_or(filename)
    };

    // Expand braces first, then match each expanded pattern
    let expanded = expand_braces(pattern);
    expanded.iter().any(|p| glob_match(p, target))
}

/// Expand brace expressions in a pattern into multiple alternatives.
///
/// Handles:
/// - `{s1,s2,s3}` -- alternatives
/// - `{num1..num2}` -- integer ranges
///
/// Nested braces are not supported by the EditorConfig spec.
fn expand_braces(pattern: &str) -> Vec<String> {
    // Find the first `{` that has a matching `}`
    let Some(open) = pattern.find('{') else {
        return vec![pattern.to_string()];
    };

    // Find the matching closing brace (not nested)
    let after_open = &pattern[open + 1..];
    let Some(close_offset) = find_matching_close_brace(after_open) else {
        // No matching close brace -- treat literal
        return vec![pattern.to_string()];
    };

    let close = open + 1 + close_offset;
    let prefix = &pattern[..open];
    let inner = &pattern[open + 1..close];
    let suffix = &pattern[close + 1..];

    // Check for integer range pattern: {num..num}
    if let Some((start, end)) = parse_integer_range(inner) {
        let range_start = start.min(end);
        let range_end = start.max(end);
        let mut results = Vec::new();
        for i in range_start..=range_end {
            let expanded_suffix = expand_braces(suffix);
            for s in &expanded_suffix {
                results.push(format!("{prefix}{i}{s}"));
            }
        }
        return results;
    }

    // Otherwise, split by comma for alternatives
    let alternatives = split_brace_alternatives(inner);
    let mut results = Vec::new();
    for alt in &alternatives {
        let combined = format!("{prefix}{alt}{suffix}");
        let expanded = expand_braces(&combined);
        results.extend(expanded);
    }
    results
}

/// Find the position of the matching `}` in a string (not counting nested braces).
fn find_matching_close_brace(s: &str) -> Option<usize> {
    let mut depth = 0;
    for (i, ch) in s.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                if depth == 0 {
                    return Some(i);
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    None
}

/// Split brace content by commas, respecting nested braces.
fn split_brace_alternatives(inner: &str) -> Vec<&str> {
    let mut results = Vec::new();
    let mut depth = 0;
    let mut start = 0;

    for (i, ch) in inner.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => depth -= 1,
            ',' if depth == 0 => {
                results.push(&inner[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    results.push(&inner[start..]);
    results
}

/// Try to parse `inner` as an integer range `num..num`.
fn parse_integer_range(inner: &str) -> Option<(i64, i64)> {
    let parts: Vec<&str> = inner.splitn(2, "..").collect();
    if parts.len() != 2 {
        return None;
    }
    let start = parts[0].trim().parse::<i64>().ok()?;
    let end = parts[1].trim().parse::<i64>().ok()?;
    Some((start, end))
}

/// Match a glob pattern (without braces) against a target string.
///
/// Supports `*`, `**`, `?`, and `[...]` character classes.
fn glob_match(pattern: &str, target: &str) -> bool {
    glob_match_recursive(pattern.as_bytes(), target.as_bytes())
}

/// Recursive glob matching implementation.
fn glob_match_recursive(pattern: &[u8], target: &[u8]) -> bool {
    let mut p = 0;
    let mut t = 0;

    // Track backtracking point for `*`
    let mut star_p: Option<usize> = None;
    let mut star_t: Option<usize> = None;

    while t < target.len() || p < pattern.len() {
        if p < pattern.len() {
            match pattern[p] {
                b'*' => {
                    // Check for `**`
                    if p + 1 < pattern.len() && pattern[p + 1] == b'*' {
                        // `**` matches everything including `/`
                        // Skip the `**`
                        let mut pp = p + 2;
                        // If followed by `/`, skip it too
                        if pp < pattern.len() && pattern[pp] == b'/' {
                            pp += 1;
                        }
                        // Try matching the rest of the pattern at every position
                        // including matching zero characters
                        for tt in t..=target.len() {
                            if glob_match_recursive(&pattern[pp..], &target[tt..]) {
                                return true;
                            }
                        }
                        return false;
                    }
                    // Single `*` -- matches any characters except `/`
                    star_p = Some(p);
                    star_t = Some(t);
                    p += 1;
                    continue;
                }
                b'?' => {
                    if t < target.len() && target[t] != b'/' {
                        p += 1;
                        t += 1;
                        continue;
                    }
                }
                b'[' => {
                    if t < target.len() {
                        if let Some(class_end) = find_class_end(&pattern[p..]) {
                            let class_content = &pattern[p + 1..p + class_end];
                            let ch = target[t];
                            if match_character_class(class_content, ch) {
                                p = p + class_end + 1;
                                t += 1;
                                continue;
                            }
                        }
                    }
                }
                c => {
                    if t < target.len() && target[t] == c {
                        p += 1;
                        t += 1;
                        continue;
                    }
                }
            }
        }

        // No match at current position -- try backtracking to last `*`
        if let (Some(sp), Some(st)) = (star_p, star_t) {
            // `*` cannot match past end of target or match `/`
            if st >= target.len() || target[st] == b'/' {
                return false;
            }
            let new_st = st + 1;
            star_t = Some(new_st);
            p = sp + 1;
            t = new_st;
            continue;
        }

        return false;
    }

    true
}

/// Find the closing `]` of a character class, returning its offset from the start `[`.
fn find_class_end(pattern: &[u8]) -> Option<usize> {
    // pattern[0] == b'['
    let mut i = 1;
    // Allow `]` as first char in class (or after `!`/`^`)
    if i < pattern.len() && (pattern[i] == b'!' || pattern[i] == b'^') {
        i += 1;
    }
    if i < pattern.len() && pattern[i] == b']' {
        i += 1;
    }
    while i < pattern.len() {
        if pattern[i] == b']' {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Match a single character against a character class content (between `[` and `]`).
///
/// Supports negation with `!` or `^` as first character, and ranges like `a-z`.
fn match_character_class(class: &[u8], ch: u8) -> bool {
    let (negated, content) = if !class.is_empty() && (class[0] == b'!' || class[0] == b'^') {
        (true, &class[1..])
    } else {
        (false, class)
    };

    let mut matched = false;
    let mut i = 0;

    while i < content.len() {
        if i + 2 < content.len() && content[i + 1] == b'-' {
            // Range: e.g., `a-z`
            let range_start = content[i];
            let range_end = content[i + 2];
            if ch >= range_start && ch <= range_end {
                matched = true;
            }
            i += 3;
        } else {
            if content[i] == ch {
                matched = true;
            }
            i += 1;
        }
    }

    if negated {
        !matched
    } else {
        matched
    }
}
