use colored::Colorize;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

#[derive(Clone, Copy)]
pub enum Accent {
    Blue,
    Cyan,
    Yellow,
    Magenta,
}

impl Accent {
    fn paint(self, s: &str) -> String {
        match self {
            Accent::Blue => s.bright_blue().to_string(),
            Accent::Cyan => s.bright_cyan().to_string(),
            Accent::Yellow => s.bright_yellow().to_string(),
            Accent::Magenta => s.bright_magenta().to_string(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Section<'a> {
    pub title: &'a str,
    pub accent: Accent,
    pub rows: &'a [(&'a str, &'a str)],
}

fn w(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

fn truncate_display(s: &str, max_width: usize) -> String {
    if w(s) <= max_width {
        return s.to_string();
    }
    if max_width == 0 {
        return String::new();
    }
    if max_width == 1 {
        return "…".to_string();
    }

    let mut out = String::new();
    let mut used = 0;
    for ch in s.chars() {
        let cw = UnicodeWidthChar::width(ch).unwrap_or(0);
        if used + cw > max_width - 1 {
            break;
        }
        out.push(ch);
        used += cw;
    }
    out.push('…');
    out
}

fn pad_right_display(s: &str, width: usize) -> String {
    let clipped = truncate_display(s, width);
    let cur = w(&clipped);
    if cur < width {
        format!("{clipped}{}", " ".repeat(width - cur))
    } else {
        clipped
    }
}

fn take_prefix_by_width(s: &str, max_width: usize) -> (&str, &str) {
    if max_width == 0 || s.is_empty() {
        return ("", s);
    }

    let mut used = 0;
    let mut split_at = 0;
    for (idx, ch) in s.char_indices() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if used + ch_width > max_width {
            break;
        }
        used += ch_width;
        split_at = idx + ch.len_utf8();
    }

    if split_at == 0 {
        let first = s.chars().next().map(char::len_utf8).unwrap_or(0);
        (&s[..first], &s[first..])
    } else {
        (&s[..split_at], &s[split_at..])
    }
}

fn wrap_word(word: &str, max_width: usize, out: &mut Vec<String>) {
    let mut rest = word;
    while !rest.is_empty() {
        let (head, tail) = take_prefix_by_width(rest, max_width);
        out.push(head.to_string());
        rest = tail;
    }
}

fn wrap_paragraph(s: &str, max_width: usize, out: &mut Vec<String>) {
    if s.is_empty() {
        out.push(String::new());
        return;
    }

    let mut current = String::new();
    let mut current_width = 0;

    for word in s.split_whitespace() {
        let word_width = w(word);
        if current.is_empty() {
            if word_width <= max_width {
                current.push_str(word);
                current_width = word_width;
            } else {
                wrap_word(word, max_width, out);
            }
            continue;
        }

        if current_width + 1 + word_width <= max_width {
            current.push(' ');
            current.push_str(word);
            current_width += 1 + word_width;
            continue;
        }

        out.push(current);
        current = String::new();
        current_width = 0;

        if word_width <= max_width {
            current.push_str(word);
            current_width = word_width;
        } else {
            wrap_word(word, max_width, out);
        }
    }

    if !current.is_empty() {
        out.push(current);
    }
}

fn wrap_display(s: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![String::new()];
    }

    let mut lines = Vec::new();
    for paragraph in s.split('\n') {
        wrap_paragraph(paragraph, max_width, &mut lines);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

fn section_line(title: &str, total_width: usize, left: char, right: char) -> String {
    let inner = total_width.saturating_sub(2);
    let label = truncate_display(&format!(" {} ", title), inner);
    let lw = w(&label);

    let left_fill = (inner - lw) / 2;
    let right_fill = inner - lw - left_fill;

    format!(
        "{left}{}{label}{}{right}",
        "─".repeat(left_fill),
        "─".repeat(right_fill)
    )
}

fn bottom_line(total_width: usize) -> String {
    format!("╰{}╯", "─".repeat(total_width.saturating_sub(2)))
}

fn row_lines(accent: Accent, key: &str, value: &str, key_w: usize, val_w: usize) -> Vec<String> {
    let left = accent.paint("│");
    let mid = accent.paint("│");
    let right = accent.paint("│");

    let key_lines = wrap_display(key, key_w);
    let value_lines = wrap_display(value, val_w);
    let line_count = key_lines.len().max(value_lines.len());

    let mut lines = Vec::with_capacity(line_count);
    for idx in 0..line_count {
        let key_part = key_lines.get(idx).map(String::as_str).unwrap_or("");
        let value_part = value_lines.get(idx).map(String::as_str).unwrap_or("");
        let key = accent.paint(&pad_right_display(key_part, key_w));
        let val = pad_right_display(value_part, val_w);
        lines.push(format!("{left} {key} {mid} {val} {right}"));
    }

    lines
}

pub fn render_dashboard(sections: &[Section], total_width: usize, key_width_hint: usize) -> String {
    let max_key = sections
        .iter()
        .flat_map(|s| s.rows.iter())
        .map(|(k, _)| w(k))
        .max()
        .unwrap_or(0);

    let key_w = key_width_hint.max(max_key);
    let min_total = key_w + 8 + 8; // value at least 8 columns
    let total_width = total_width.max(min_total);
    let val_w = total_width - key_w - 7; // because: "│ {key} │ {val} │"

    let mut out = Vec::new();

    // Top section (title in top border)
    let first = sections[0];
    out.push(
        first
            .accent
            .paint(&section_line(first.title, total_width, '╭', '╮')),
    );
    for (k, v) in first.rows {
        out.extend(row_lines(first.accent, k, v, key_w, val_w));
    }

    // Middle sections (title separators)
    for section in &sections[1..] {
        out.push(
            section
                .accent
                .paint(&section_line(section.title, total_width, '├', '┤')),
        );
        for (k, v) in section.rows {
            out.extend(row_lines(section.accent, k, v, key_w, val_w));
        }
    }

    // Bottom border
    let last_accent = sections[sections.len() - 1].accent;
    out.push(last_accent.paint(&bottom_line(total_width)));

    out.join("\n")
}
