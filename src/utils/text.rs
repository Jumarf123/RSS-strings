use std::collections::HashSet;

pub const MAX_STRINGS: usize = 50_000;

pub fn parse_input_lines(input: &str, limit: usize) -> Vec<String> {
    let limit = limit.min(MAX_STRINGS);
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let candidate = if let Some((_, rhs)) = trimmed.split_once(":::") {
            let rhs = rhs.trim();
            if rhs.is_empty() {
                continue;
            }
            rhs
        } else {
            trimmed
        };
        let key = candidate.to_ascii_lowercase();
        if seen.insert(key) {
            out.push(candidate.to_string());
        }
        if out.len() >= limit {
            break;
        }
    }
    out
}

pub fn sample_input() -> String {
    SAMPLE_STRINGS.join("\n")
}

static SAMPLE_STRINGS: &[&str] = &[
    "password",
    "token",
    "session",
    "user",
    "debug",
    "api_key",
    "secret",
    "auth",
    "login",
    "refresh",
    "bearer",
    "cookie",
    "csrf",
    "jwt",
    "client_id",
    "client_secret",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedup_and_trim() {
        let src = "foo \n Foo\nbar\n\nbaz  ";
        let parsed = parse_input_lines(src, 10);
        assert_eq!(parsed, vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn respects_limit() {
        let src = (0..60_000)
            .map(|i| format!("item{i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let parsed = parse_input_lines(&src, 50_000);
        assert_eq!(parsed.len(), 50_000);
        assert_eq!(parsed[0], "item0");
    }

    #[test]
    fn extracts_after_marker() {
        let src = "shadowbypasspublic:::2026/01/06:02:05:58\nshadowbypassprivate:::2026/01/01:21:25:56\nclub44:::2026/01/04:13:35:48\nshitbypass:::!2026/01/14:02:37:40\n";
        let parsed = parse_input_lines(src, 10);
        assert_eq!(
            parsed,
            vec![
                "2026/01/06:02:05:58",
                "2026/01/01:21:25:56",
                "2026/01/04:13:35:48",
                "!2026/01/14:02:37:40",
            ]
        );
    }
}
