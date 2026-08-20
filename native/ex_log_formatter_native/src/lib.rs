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

#[derive(NifStruct, Clone)]
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

#[derive(NifStruct, Clone)]
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
static LOGFMT_RE: OnceLock<Regex> = OnceLock::new();
static GENERAL_LOG_RE: OnceLock<Regex> = OnceLock::new();
static ERROR_KEYWORDS_RE: OnceLock<Regex> = OnceLock::new();
static STRICT_ERROR_KEYWORDS_RE: OnceLock<Regex> = OnceLock::new();

fn get_sub_highlight_re() -> &'static Regex {
    SUB_HIGHLIGHT_RE.get_or_init(|| {
        Regex::new(r#"(?i)(?P<uuid>\b[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}\b)|(?P<hash>\b0x[0-9a-fA-F]+\b|\b[0-9a-fA-F]{40}\b)|(?P<mac>\b(?:[0-9a-fA-F]{2}:){5}[0-9a-fA-F]{2}\b)|(?P<url>\bhttps?://[^\s]+)|(?P<ip_bracket>\[[0-9a-fA-F:]+\](?::\d{1,5})?)|(?P<ip_v4>\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}(?::\d{1,5}|/(?:[0-9]|[12][0-9]|3[0-2]))?\b)|(?P<ip_v6>\b(?:[0-9a-fA-F]{1,4}:)+(?::[0-9a-fA-F]{1,4})+(?:/(?:[0-9]|[1-9][0-9]|1[0-1][0-9]|12[0-8]))?\b|\b(?:[0-9a-fA-F]{1,4}:){1,7}:(?:/(?:[0-9]|[1-9][0-9]|1[0-1][0-9]|12[0-8]))?\b|(?:^|[\s"'\x28])::(?:[0-9a-fA-F]{1,4}:){0,5}[0-9a-fA-F]{1,4}(?:/(?:[0-9]|[1-9][0-9]|1[0-1][0-9]|12[0-8]))?\b|(?:^|[\s"'\x28])::(?:/(?:[0-9]|[1-9][0-9]|1[0-1][0-9]|12[0-8]))?)|(?P<path>(?:^|[\s"'\x28])/(?:[a-zA-Z0-9._-]+/)*[a-zA-Z0-9._-]*)|(?P<mem>\b\d+(?:\.\d+)?(?:MB|GB|KB|TB|B|MiB|GiB|KiB)\b)|(?P<duration>\b\d+(?:\.\d+)?(?:µs|us|ms|s|min|ns|m)\b)|(?P<method>\b(?:GET|POST|PUT|DELETE|PATCH|OPTIONS|HEAD)\b)|(?P<status>\b[1-5]\d{2}\b)"#).unwrap()
    })
}

fn get_logfmt_re() -> &'static Regex {
    LOGFMT_RE.get_or_init(|| {
        Regex::new(r#"(?P<key>[a-zA-Z0-9_.-]+)=(?:"(?P<qval>[^"]*)"|(?P<uval>[^\s]+))"#).unwrap()
    })
}

fn get_general_log_re() -> &'static Regex {
    GENERAL_LOG_RE.get_or_init(|| {
        Regex::new(r#"(?i)^(?:(?P<ts>\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:?\d{2})?|\d{2}:\d{2}:\d{2}(?:\.\d+)?)\s)?(?:(?:\[(?P<bracket_lvl>[a-zA-Z0-9_-]+)\]|(?P<colon_lvl>\b(?:info|warn|warning|error|err|debug|fatal|trace|critical|crit|emerg|emergency|stderr|fail|failure|severe|notice)\b):|(?P<bare_lvl>\b(?:info|warn|warning|error|err|debug|fatal|trace|critical|crit|emerg|emergency|stderr|fail|failure|severe|notice)\b))\s?)?(?P<msg>.*)$"#).unwrap()
    })
}

fn get_error_keywords_re() -> &'static Regex {
    ERROR_KEYWORDS_RE.get_or_init(|| {
        Regex::new(r#"(?i)\b(?:panic|fatal|exception|crash|runtimeerror|compileerror|unhandled|traceback|backtrace)\b|\*\*\s*\(|\bcaused by:|\b\s*at\s+[a-zA-Z0-9_.$]+\("#).unwrap()
    })
}

fn get_strict_error_keywords_re() -> &'static Regex {
    STRICT_ERROR_KEYWORDS_RE.get_or_init(|| {
        Regex::new(r#"(?i)(?:\b(?:panic|runtimeerror|compileerror)\b:\s*|\b(?:fatal|uncaught|unhandled)\b|\*\*\s*\(|\btraceback \(most recent call last\):|\bcaused by:|\bgoroutine \d+ \[|\bstack backtrace:)"#).unwrap()
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
pub fn parse_docker_log(line: String) -> (Option<String>, String) {
    if let Some((ts, rest)) = line.split_once(' ') {
        let ts_trimmed = ts.trim();
        if ts_trimmed.len() >= 19
            && ts_trimmed.as_bytes()[4] == b'-'
            && ts_trimmed.as_bytes()[7] == b'-'
            && (ts_trimmed.as_bytes()[10] == b'T' || ts_trimmed.as_bytes()[10] == b' ')
        {
            return (Some(ts.to_string()), rest.to_string());
        }
    }
    (None, line)
}

#[rustler::nif]
pub fn wrap_spans(spans: Vec<Span>, width: usize) -> Vec<Vec<Span>> {
    let max_w = width.max(1);
    let mut lines = Vec::new();
    let mut current_line = Vec::new();
    let mut rem_w = max_w;

    for span in spans {
        if span.content.is_empty() {
            current_line.push(span);
            continue;
        }

        let char_count = span.content.chars().count();
        if char_count <= rem_w {
            rem_w -= char_count;
            current_line.push(span);
        } else {
            let mut remaining_str = span.content.as_str();
            while !remaining_str.is_empty() {
                let remaining_chars = remaining_str.chars().count();
                if remaining_chars <= rem_w {
                    current_line.push(Span {
                        content: remaining_str.to_string(),
                        style: span.style.clone(),
                    });
                    rem_w -= remaining_chars;
                    break;
                } else {
                    let mut split_idx = 0;
                    for (i, (byte_idx, _)) in remaining_str.char_indices().enumerate() {
                        if i == rem_w {
                            split_idx = byte_idx;
                            break;
                        }
                    }
                    if split_idx == 0 && rem_w > 0 {
                        split_idx = remaining_str.len();
                    }

                    let (chunk, rest) = remaining_str.split_at(split_idx);
                    current_line.push(Span {
                        content: chunk.to_string(),
                        style: span.style.clone(),
                    });

                    lines.push(std::mem::take(&mut current_line));
                    rem_w = max_w;
                    remaining_str = rest;
                }
            }
        }
    }

    if !current_line.is_empty() || lines.is_empty() {
        lines.push(current_line);
    }

    lines
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

            if cap.name("uuid").is_some() || cap.name("hash").is_some() || cap.name("mac").is_some() {
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
            } else if cap.name("mem").is_some() {
                spans.push(Span::new(token, atoms::yellow()));
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

fn get_level_atom(lvl: &str) -> Atom {
    match lvl.trim().to_lowercase().as_str() {
        "info" | "i" => atoms::green(),
        "warn" | "warning" | "w" => atoms::yellow(),
        "error" | "err" | "fatal" | "critical" | "crit" | "emerg" | "emergency" | "stderr"
        | "fail" | "failure" | "panic" | "severe" | "50" | "60" | "0" | "1" | "2" | "3" | "e" | "f" => atoms::red(),
        "debug" | "trace" | "d" | "t" => atoms::magenta(),
        _ => atoms::cyan(),
    }
}

fn is_error_level(lvl: &str) -> bool {
    let l = lvl.trim().to_lowercase();
    matches!(
        l.as_str(),
        "error" | "err" | "fatal" | "critical" | "crit" | "emerg" | "emergency" | "stderr"
        | "fail" | "failure" | "panic" | "severe" | "e" | "f" | "50" | "60" | "0" | "1" | "2" | "3"
    )
}

fn is_non_error_level(lvl: &str) -> bool {
    let l = lvl.trim().to_lowercase();
    matches!(
        l.as_str(),
        "info" | "warn" | "warning" | "debug" | "trace" | "notice" | "30" | "20" | "10" | "4" | "5" | "6" | "7" | "i" | "w" | "d"
    )
}

fn is_5xx_status_str(status: &str) -> bool {
    let s = status.trim();
    s.len() == 3 && s.starts_with('5') && s.chars().all(|c| c.is_ascii_digit())
}

fn parse_json_object(map: &serde_json::Map<String, Value>) -> (Vec<Span>, bool) {
    let mut spans = Vec::new();

    let mut level_str = None;
    for k in &["level", "severity", "lvl", "log.level", "s", "severity_number"] {
        if let Some(v) = map.get(*k) {
            match v {
                Value::String(s) => { level_str = Some(s.to_string()); break; },
                Value::Number(n) => {
                    let num = n.as_u64().unwrap_or(0);
                    if num >= 17 && *k == "severity_number" {
                        level_str = Some("ERROR".to_string());
                    } else {
                        level_str = Some(n.to_string());
                    }
                    break;
                },
                _ => {}
            }
        }
    }

    let has_truthy_error = ["error", "err", "exception", "stack", "stacktrace", "backtrace", "error_message", "error_details", "exc_info", "cause"]
        .iter()
        .any(|k| {
            if let Some(v) = map.get(*k) {
                match v {
                    Value::Null => false,
                    Value::Bool(b) => *b,
                    Value::String(s) => !s.is_empty(),
                    Value::Array(a) => !a.is_empty(),
                    Value::Object(o) => !o.is_empty(),
                    Value::Number(_) => true,
                }
            } else {
                false
            }
        });

    let has_5xx_status = map.get("status").map_or(false, |v| {
        match v {
            Value::Number(n) => n.as_u64().map_or(false, |code| code >= 500 && code < 600),
            Value::String(s) => is_5xx_status_str(s),
            _ => false,
        }
    });

    let is_err = match &level_str {
        Some(lvl) if is_error_level(lvl) => true,
        Some(lvl) if is_non_error_level(lvl) => has_truthy_error || has_5xx_status,
        _ => has_truthy_error || has_5xx_status,
    };

    let mut first = true;
    for (k, v) in map {
        let prefix = if first { "" } else { " " };
        first = false;

        let key_span = Span::new(format!("{}{}", prefix, format!("{}=", k)), atoms::cyan());
        spans.push(key_span);

        let k_lower = k.to_lowercase();
        if matches!(k_lower.as_str(), "level" | "severity" | "lvl" | "log.level" | "s") {
            let str_val = match v {
                Value::String(s) => s.to_string(),
                Value::Number(n) => n.to_string(),
                _ => v.to_string(),
            };
            let lvl_atom = get_level_atom(&str_val);
            let formatted_lvl = format!("[{}]", str_val.to_uppercase());
            spans.push(Span::bold(formatted_lvl, lvl_atom));
        } else if matches!(k_lower.as_str(), "timestamp" | "time" | "ts" | "@timestamp" | "datetime") {
            let str_val = match v {
                Value::String(s) => s.to_string(),
                _ => v.to_string(),
            };
            spans.push(Span::new(str_val, atoms::dark_gray()));
        } else if matches!(k_lower.as_str(), "message" | "msg" | "log" | "message_text" | "event" | "detail" | "details" | "description" | "reason" | "text" | "body" | "payload" | "summary" | "info") {
            let str_val = match v {
                Value::String(s) => s.to_string(),
                _ => v.to_string(),
            };
            spans.extend(do_sub_highlight(&str_val));
        } else {
            match v {
                Value::String(s) => {
                    spans.extend(do_sub_highlight(s));
                }
                Value::Number(n) => {
                    spans.push(Span::new(n.to_string(), atoms::yellow()));
                }
                Value::Bool(b) => {
                    spans.push(Span::new(b.to_string(), atoms::magenta()));
                }
                Value::Null => {
                    spans.push(Span::new("null", atoms::dark_gray()));
                }
                _ => {
                    spans.push(Span::new(v.to_string(), atoms::dark_gray()));
                }
            }
        }
    }

    (spans, is_err)
}

fn try_parse_logfmt(line: &str) -> Option<(Vec<Span>, bool)> {
    if !line.contains('=') || line.contains("=>") || line.contains("%{") {
        return None;
    }

    let re = get_logfmt_re();
    let mut matches = Vec::new();

    for cap in re.captures_iter(line) {
        if let Some(m) = cap.get(0) {
            let key = cap.name("key").map(|k| k.as_str()).unwrap_or("");
            let qval = cap.name("qval").map(|v| v.as_str());
            let uval = cap.name("uval").map(|v| v.as_str());
            let clean_val = qval.or(uval).unwrap_or("");
            let raw_val = &line[m.start() + key.len() + 1..m.end()];
            matches.push((key, clean_val, raw_val, m.start(), m.end()));
        }
    }

    if matches.is_empty() {
        return None;
    }

    let has_known_key = matches.iter().any(|(k, _, _, _, _)| {
        matches!(k.to_lowercase().as_str(), "level" | "lvl" | "severity" | "msg" | "message" | "ts" | "time" | "status" | "err" | "error")
    });

    if !has_known_key && matches.len() < 2 {
        return None;
    }

    let mut level_str = None;
    for (k, clean_val, _, _, _) in &matches {
        if matches!(k.to_lowercase().as_str(), "level" | "lvl" | "severity") {
            level_str = Some(*clean_val);
            break;
        }
    }

    let has_truthy_error = matches.iter().any(|(k, clean_val, _, _, _)| {
        let kl = k.to_lowercase();
        (kl == "error" || kl == "err" || kl == "exception" || kl == "stack" || kl == "stacktrace")
            && !clean_val.is_empty() && *clean_val != "false" && *clean_val != "nil" && *clean_val != "null" && *clean_val != "\"\""
    });

    let has_5xx_status = matches.iter().any(|(k, clean_val, _, _, _)| {
        k.to_lowercase() == "status" && is_5xx_status_str(clean_val)
    });

    let is_err = match level_str {
        Some(lvl) if is_error_level(lvl) => true,
        Some(lvl) if is_non_error_level(lvl) => has_truthy_error || has_5xx_status,
        _ => has_truthy_error || has_5xx_status,
    };

    let mut spans = Vec::new();
    let mut last_pos = 0;

    for (key, clean_val, raw_val, start, end) in matches {
        if start > last_pos {
            spans.push(Span::new(&line[last_pos..start], atoms::white()));
        }

        let key_span = Span::new(format!("{}=", key), atoms::cyan());
        spans.push(key_span);

        let k_lower = key.to_lowercase();
        if matches!(k_lower.as_str(), "level" | "lvl" | "severity") {
            let lvl_atom = get_level_atom(clean_val);
            spans.push(Span::bold(format!("[{}]", clean_val.to_uppercase()), lvl_atom));
        } else if matches!(k_lower.as_str(), "ts" | "time" | "timestamp") {
            spans.push(Span::new(clean_val, atoms::dark_gray()));
        } else if matches!(k_lower.as_str(), "msg" | "message") {
            spans.extend(do_sub_highlight(clean_val));
        } else {
            spans.extend(do_sub_highlight(raw_val));
        }

        last_pos = end;
    }

    if last_pos < line.len() {
        spans.push(Span::new(&line[last_pos..], atoms::white()));
    }

    Some((spans, is_err))
}

fn parse_general_log(line: &str) -> (Vec<Span>, bool) {
    let re = get_general_log_re();
    if let Some(cap) = re.captures(line) {
        let ts = cap.name("ts").map(|m| m.as_str()).unwrap_or("");
        let bracket_lvl = cap.name("bracket_lvl").map(|m| m.as_str()).unwrap_or("");
        let colon_lvl = cap.name("colon_lvl").map(|m| m.as_str()).unwrap_or("");
        let bare_lvl = cap.name("bare_lvl").map(|m| m.as_str()).unwrap_or("");
        let msg = cap.name("msg").map(|m| m.as_str()).unwrap_or("");

        let level_str = if !bracket_lvl.is_empty() {
            Some(bracket_lvl)
        } else if !colon_lvl.is_empty() {
            Some(colon_lvl)
        } else if !bare_lvl.is_empty() {
            Some(bare_lvl)
        } else {
            None
        };

        let is_err = match level_str {
            Some(lvl) if is_error_level(lvl) => true,
            Some(lvl) if is_non_error_level(lvl) => get_strict_error_keywords_re().is_match(line) || line.contains(" 500 ") || line.contains(" 502 ") || line.contains(" 503 "),
            _ => get_error_keywords_re().is_match(line) || line.contains(" 500 ") || line.contains("HTTP 5"),
        };

        let mut spans = Vec::new();
        if !ts.is_empty() {
            spans.push(Span::new(format!("{} ", ts), atoms::dark_gray()));
        }

        if !bracket_lvl.is_empty() {
            let lvl_atom = get_level_atom(bracket_lvl);
            spans.push(Span::bold(format!("[{}] ", bracket_lvl), lvl_atom));
        } else if !colon_lvl.is_empty() {
            let lvl_atom = get_level_atom(colon_lvl);
            spans.push(Span::bold(format!("{}: ", colon_lvl), lvl_atom));
        } else if !bare_lvl.is_empty() {
            let lvl_atom = get_level_atom(bare_lvl);
            spans.push(Span::bold(format!("{} ", bare_lvl), lvl_atom));
        }

        spans.extend(do_sub_highlight(msg));
        (spans, is_err)
    } else {
        let is_err = get_error_keywords_re().is_match(line);
        (do_sub_highlight(line), is_err)
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

    let clean_line = do_sanitize_log(&line);
    let trimmed = clean_line.trim();

    // 1. Try JSON
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
            if let Some(map) = value.as_object() {
                return parse_json_object(map);
            }
        }
    }

    // Prefix JSON (e.g. "2026-08-04T10:15:00Z stdout F {"level":"info",...}")
    if trimmed.contains('{') && trimmed.ends_with('}') {
        if let Some(idx) = trimmed.find('{') {
            let prefix = &trimmed[..idx];
            let json_part = &trimmed[idx..];
            if let Ok(value) = serde_json::from_str::<Value>(json_part) {
                if let Some(map) = value.as_object() {
                    let (mut prefix_spans, p_err) = parse_general_log(prefix);
                    let (json_spans, j_err) = parse_json_object(map);
                    prefix_spans.extend(json_spans);
                    return (prefix_spans, p_err || j_err);
                }
            }
        }
    }

    // 2. Try Logfmt
    if let Some((spans, is_err)) = try_parse_logfmt(&clean_line) {
        return (spans, is_err);
    }

    // 3. General Regex Log + Sub-highlight
    parse_general_log(&clean_line)
}

rustler::init!("Elixir.ExLogFormatter.Native");
