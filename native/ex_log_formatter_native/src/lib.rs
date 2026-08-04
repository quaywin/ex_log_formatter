use rustler::{Atom, NifStruct};
use serde_json::Value;
use regex::Regex;
use std::sync::OnceLock;

mod atoms {
    rustler::atoms! {
        white,
        green,
        red,
        yellow,
        cyan,
        magenta,
        dark_gray,
        blue,
        bold,
    }
}

#[derive(NifStruct)]
#[module = "ExRatatui.Style"]
pub struct Style {
    pub fg: Option<Atom>,
    pub bg: Option<Atom>,
    pub modifiers: Vec<Atom>,
}

impl Style {
    pub fn new(fg: Atom) -> Self {
        Style {
            fg: Some(fg),
            bg: None,
            modifiers: Vec::new(),
        }
    }

    pub fn with_bold(fg: Atom) -> Self {
        Style {
            fg: Some(fg),
            bg: None,
            modifiers: vec![atoms::bold()],
        }
    }
}

#[derive(NifStruct)]
#[module = "ExRatatui.Text.Span"]
pub struct Span {
    pub content: String,
    pub style: Style,
}

impl Span {
    pub fn new(content: impl Into<String>, fg: Atom) -> Self {
        Span {
            content: content.into(),
            style: Style::new(fg),
        }
    }

    pub fn bold(content: impl Into<String>, fg: Atom) -> Self {
        Span {
            content: content.into(),
            style: Style::with_bold(fg),
        }
    }
}

static SUB_HIGHLIGHT_RE: OnceLock<Regex> = OnceLock::new();

fn get_sub_highlight_re() -> &'static Regex {
    SUB_HIGHLIGHT_RE.get_or_init(|| {
        Regex::new(r#"(?i)(?P<uuid>\b[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}\b)|(?P<hash>\b0x[0-9a-fA-F]+\b|\b[0-9a-fA-F]{40}\b)|(?P<url>\bhttps?://[^\s]+)|(?P<ip_bracket>\[[0-9a-fA-F:]+\](?::\d{1,5})?)|(?P<ip_v4>\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}(?::\d{1,5}|/(?:[0-9]|[12][0-9]|3[0-2]))?\b)|(?P<ip_v6>\b(?:[0-9a-fA-F]{1,4}:)+(?::[0-9a-fA-F]{1,4})+(?:/(?:[0-9]|[1-9][0-9]|1[0-1][0-9]|12[0-8]))?\b|\b(?:[0-9a-fA-F]{1,4}:){1,7}:(?:/(?:[0-9]|[1-9][0-9]|1[0-1][0-9]|12[0-8]))?\b|(?:^|[\s"'\x28])::(?:[0-9a-fA-F]{1,4}:){0,5}[0-9a-fA-F]{1,4}(?:/(?:[0-9]|[1-9][0-9]|1[0-1][0-9]|12[0-8]))?\b|(?:^|[\s"'\x28])::(?:/(?:[0-9]|[1-9][0-9]|1[0-1][0-9]|12[0-8]))?)|(?P<path>(?:^|[\s"'\x28])/(?:[a-zA-Z0-9._-]+/)*[a-zA-Z0-9._-]*)|(?P<duration>\b\d+(?:\.\d+)?(?:µs|us|ms|s|min|ns)\b)|(?P<method>\b(?:GET|POST|PUT|DELETE|PATCH|OPTIONS|HEAD)\b)|(?P<status>\b[1-5]\d{2}\b)"#).unwrap()
    })
}

fn do_sanitize_log(line: &str) -> String {
    let bytes = strip_ansi_escapes::strip(line);
    String::from_utf8_lossy(&bytes).to_string()
}

#[rustler::nif]
pub fn sanitize_log(line: String) -> String {
    do_sanitize_log(&line)
}

#[rustler::nif]
pub fn process_chunk_native(chunk: String, buffer: String, max_len: usize) -> (Vec<String>, String) {
    let combined = format!("{}{}", buffer, chunk);
    let mut complete_lines = Vec::new();
    let mut last_idx = 0;

    for (idx, &byte) in combined.as_bytes().iter().enumerate() {
        if byte == b'\n' {
            let mut line = combined[last_idx..idx].trim_end_matches('\r').to_string();
            if line.len() > max_len {
                line.truncate(max_len);
                line.push_str("... [truncated]");
            }
            complete_lines.push(line);
            last_idx = idx + 1;
        }
    }

    let remaining = combined[last_idx..].to_string();
    (complete_lines, remaining)
}

fn trim_trailing_punct(s: &str) -> (&str, &str) {
    let mut end = s.len();
    while end > 0 {
        let last_char = s[..end].chars().last().unwrap();
        if matches!(last_char, '"' | '\'' | ',' | ';' | ')' | ']' | '>' | '}' | '.') {
            end -= last_char.len_utf8();
        } else {
            break;
        }
    }
    (&s[..end], &s[end..])
}

fn do_sub_highlight(text: &str) -> Vec<Span> {
    let re = get_sub_highlight_re();
    let mut spans = Vec::new();
    let mut last_pos = 0;

    for cap in re.captures_iter(text) {
        if let Some(m) = cap.get(0) {
            let mut start = m.start();
            let end = m.end();

            let mut token = m.as_str();

            if cap.name("path").is_some() || cap.name("ip_v6").is_some() {
                if let Some(first_char) = token.chars().next() {
                    if matches!(first_char, ' ' | '\t' | '"' | '\'' | '(') {
                        token = &token[first_char.len_utf8()..];
                        start += first_char.len_utf8();
                    }
                }
            }

            if start > last_pos {
                spans.push(Span::new(&text[last_pos..start], atoms::white()));
            }

            if cap.name("uuid").is_some() || cap.name("hash").is_some() {
                spans.push(Span::new(token, atoms::dark_gray()));
            } else if cap.name("url").is_some() {
                let (clean_url, trailing) = trim_trailing_punct(token);
                spans.push(Span::new(clean_url, atoms::magenta()));
                if !trailing.is_empty() {
                    spans.push(Span::new(trailing, atoms::white()));
                }
            } else if cap.name("path").is_some() {
                let (clean_path, trailing) = trim_trailing_punct(token);
                spans.push(Span::new(clean_path, atoms::cyan()));
                if !trailing.is_empty() {
                    spans.push(Span::new(trailing, atoms::white()));
                }
            } else if cap.name("ip_v4").is_some() || cap.name("ip_v6").is_some() || cap.name("ip_bracket").is_some() {
                spans.push(Span::new(token, atoms::magenta()));
            } else if cap.name("duration").is_some() {
                spans.push(Span::new(token, atoms::cyan()));
            } else if cap.name("method").is_some() {
                spans.push(Span::bold(token, atoms::blue()));
            } else if cap.name("status").is_some() {
                if token.starts_with('2') {
                    spans.push(Span::new(token, atoms::green()));
                } else if token.starts_with('5') {
                    spans.push(Span::bold(token, atoms::red()));
                } else {
                    spans.push(Span::new(token, atoms::yellow()));
                }
            } else {
                spans.push(Span::new(token, atoms::yellow()));
            }

            last_pos = end;
        }
    }

    if last_pos < text.len() {
        spans.push(Span::new(&text[last_pos..], atoms::white()));
    }

    if spans.is_empty() {
        vec![Span::new(text, atoms::white())]
    } else {
        spans
    }
}

#[rustler::nif]
pub fn sub_highlight_native(text: String) -> Vec<Span> {
    do_sub_highlight(&text)
}

#[rustler::nif]
pub fn parse_log_line(line: String) -> (Vec<Span>, bool) {
    if line.is_empty() {
        return (vec![Span::new("", atoms::white())], false);
    }

    // 1. Strip ANSI escapes
    let clean_line = do_sanitize_log(&line);
    let trimmed = clean_line.trim();

    // 2. Try JSON Parse
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
            if let Some(map) = value.as_object() {
                return parse_json_object(map);
            }
        }
    }

    // 3. Fallback: Parse general text line + sub-highlight
    let is_err = trimmed.to_lowercase().contains("error") || trimmed.to_lowercase().contains("err");
    let spans = do_sub_highlight(&clean_line);
    (spans, is_err)
}

fn parse_json_object(map: &serde_json::Map<String, Value>) -> (Vec<Span>, bool) {
    let mut spans = Vec::new();
    let mut is_err = false;

    if let Some(lvl) = map.get("level").or_else(|| map.get("severity")).or_else(|| map.get("lvl")) {
        let lvl_str = lvl.as_str().unwrap_or("").to_lowercase();
        if lvl_str == "error" || lvl_str == "fatal" || lvl_str == "crit" || lvl_str == "err" {
            is_err = true;
        }
    }

    let mut first = true;
    for (k, v) in map {
        let prefix = if first { "" } else { " " };
        first = false;

        spans.push(Span::new(format!("{}{}", prefix, k), atoms::cyan()));

        match v {
            Value::String(s) => {
                spans.push(Span::new(format!("={}", s), atoms::green()));
            }
            Value::Number(n) => {
                spans.push(Span::new(format!("={}", n), atoms::yellow()));
            }
            Value::Bool(b) => {
                spans.push(Span::new(format!("={}", b), atoms::magenta()));
            }
            _ => {
                spans.push(Span::new(format!("={}", v), atoms::dark_gray()));
            }
        }
    }

    (spans, is_err)
}

rustler::init!("Elixir.ExLogFormatter.Native");
