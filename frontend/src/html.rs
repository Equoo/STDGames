//! Game descriptions come back as HTML (the CDN API's `description`/`short_description` fields
//! — the Svelte original just dumped them into the DOM via `{@html ...}`). Slint's `Text` has no
//! rich-text/HTML rendering, so this strips tags down to plain text instead of showing the raw
//! markup verbatim, while still keeping paragraph/list structure as line breaks.

/// Converts a (possibly empty) HTML string to plain text. Unknown/malformed markup degrades
/// gracefully — worst case, stray `<`/`>` get dropped, which just yields slightly heavier
/// whitespace collapsing rather than a broken string.
pub fn to_text(html: &str) -> String {
    if html.is_empty() {
        return String::new();
    }

    let mut out = String::with_capacity(html.len());
    let mut chars = html.chars();
    let mut tag = String::new();

    while let Some(c) = chars.next() {
        if c != '<' {
            out.push(c);
            continue;
        }
        tag.clear();
        for c in chars.by_ref() {
            if c == '>' {
                break;
            }
            tag.push(c);
        }
        let raw = tag.trim();
        let is_closing = raw.starts_with('/');
        let tag_name = raw
            .trim_start_matches('/')
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_end_matches('/')
            .to_lowercase();
        match tag_name.as_str() {
            "br" => out.push('\n'),
            // Only the opening tag gets a bullet — matching on both would double up (`</li>`
            // would otherwise re-match "li" too since it's compared with the leading `/` gone).
            "li" if !is_closing => out.push_str("\n• "),
            "p" | "div" | "ul" | "ol" | "h1" | "h2" | "h3" | "h4" if !out.is_empty() && !out.ends_with('\n') => {
                out.push('\n');
            }
            _ => {}
        }
    }

    let decoded = decode_entities(&out);
    let mut lines: Vec<&str> = Vec::new();
    for line in decoded.lines().map(str::trim) {
        if line.is_empty() && lines.last().is_some_and(|l| l.is_empty()) {
            continue;
        }
        lines.push(line);
    }
    lines.join("\n").trim().to_string()
}

fn decode_entities(s: &str) -> String {
    s.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
}

fn collapse_whitespace(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_was_space = false;
    for c in s.chars() {
        if c.is_whitespace() {
            if !last_was_space {
                out.push(' ');
            }
            last_was_space = true;
        } else {
            out.push(c);
            last_was_space = false;
        }
    }
    out
}

/// One renderable unit of a parsed description — see `DescriptionBlock` in `ui/model.slint`,
/// which this maps onto directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Heading(String),
    Paragraph { bold_prefix: String, text: String },
    ListItem { bold_prefix: String, text: String },
    Image { src: String },
}

#[derive(Default)]
struct Current {
    bold_prefix: String,
    text: String,
    in_bold: bool,
    is_heading: bool,
    is_list_item: bool,
}

impl Current {
    /// A `<strong>`/`<b>` run only becomes `bold_prefix` while nothing else has been written to
    /// the block yet — this is what makes Steam's common `<strong>Label</strong> - text...` show
    /// up as a bold lead-in rather than losing the emphasis entirely. A second bold run later in
    /// the same block (rare) just gets appended to `text` like any other inline tag.
    fn push(&mut self, c: char) {
        if self.in_bold && self.text.trim().is_empty() {
            self.bold_prefix.push(c);
        } else {
            self.text.push(c);
        }
    }
}

/// Parses Steam store-page HTML (the CDN API's `description` field) into a sequence of
/// renderable blocks, since Slint's `Text` can't render HTML directly (no rich-text spans, no
/// embedded images) — see the doc comment on `DescriptionBlock` in `ui/model.slint`.
#[allow(unused_assignments)] // `flush!()`'s final reset of `cur` before returning is a no-op, not a bug
pub fn parse_blocks(html: &str) -> Vec<Block> {
    if html.is_empty() {
        return Vec::new();
    }

    let mut blocks = Vec::new();
    let mut cur = Current::default();
    let mut chars = html.chars();
    let mut tag = String::new();

    macro_rules! flush {
        () => {{
            let bold_prefix = decode_entities(collapse_whitespace(cur.bold_prefix.trim()).trim());
            let text = decode_entities(collapse_whitespace(cur.text.trim()).trim());
            if !bold_prefix.is_empty() || !text.is_empty() {
                blocks.push(if cur.is_heading {
                    Block::Heading(if bold_prefix.is_empty() {
                        text
                    } else if text.is_empty() {
                        bold_prefix
                    } else {
                        format!("{bold_prefix} {text}")
                    })
                } else if cur.is_list_item {
                    Block::ListItem { bold_prefix, text }
                } else {
                    Block::Paragraph { bold_prefix, text }
                });
            }
            cur = Current::default();
        }};
    }

    while let Some(c) = chars.next() {
        if c != '<' {
            cur.push(c);
            continue;
        }
        tag.clear();
        for c in chars.by_ref() {
            if c == '>' {
                break;
            }
            tag.push(c);
        }
        let raw = tag.trim();
        let is_closing = raw.starts_with('/');
        let tag_name = raw
            .trim_start_matches('/')
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_end_matches('/')
            .to_lowercase();

        match tag_name.as_str() {
            "br" | "p" | "div" | "ul" | "ol" => flush!(),
            "h1" | "h2" | "h3" | "h4" => {
                flush!();
                if !is_closing {
                    cur.is_heading = true;
                }
            }
            "li" => {
                flush!();
                if !is_closing {
                    cur.is_list_item = true;
                }
            }
            "strong" | "b" => cur.in_bold = !is_closing,
            "img" => {
                if let Some(src) = extract_attr(raw, "src") {
                    flush!();
                    blocks.push(Block::Image { src });
                }
            }
            _ => {}
        }
    }
    flush!();

    blocks
}

/// Extracts `attr="value"` (or `attr='value'`/unquoted `attr=value`, as Steam's markup uses for
/// `width`/`height`) from a raw `<tag ...>` string with the surrounding angle brackets removed.
fn extract_attr(raw_tag: &str, attr: &str) -> Option<String> {
    let needle = format!("{attr}=");
    let idx = raw_tag.find(&needle)?;
    let rest = &raw_tag[idx + needle.len()..];
    match rest.chars().next()? {
        quote @ ('"' | '\'') => {
            let end = rest[1..].find(quote)?;
            Some(rest[1..1 + end].to_string())
        }
        _ => {
            let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
            Some(rest[..end].to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_blocks, to_text, Block};

    #[test]
    fn empty_stays_empty() {
        assert_eq!(to_text(""), "");
    }

    #[test]
    fn strips_tags_and_decodes_entities() {
        assert_eq!(to_text("<b>Bold</b> &amp; <i>italic</i>"), "Bold & italic");
    }

    #[test]
    fn paragraphs_become_blank_line_separated() {
        assert_eq!(to_text("<p>First</p><p>Second</p>"), "First\nSecond");
    }

    #[test]
    fn br_becomes_newline() {
        assert_eq!(to_text("Line one<br>Line two<br/>Line three"), "Line one\nLine two\nLine three");
    }

    #[test]
    fn list_items_become_bullets() {
        assert_eq!(to_text("<ul><li>One</li><li>Two</li></ul>"), "• One\n• Two");
    }

    #[test]
    fn collapses_runs_of_blank_lines() {
        assert_eq!(to_text("<p>A</p><p></p><p></p><p>B</p>"), "A\nB");
    }

    #[test]
    fn parse_blocks_empty_stays_empty() {
        assert_eq!(parse_blocks(""), vec![]);
    }

    #[test]
    fn parse_blocks_heading_and_paragraph() {
        assert_eq!(
            parse_blocks("<h2 class=\"bb_tag\">A Heading</h2>Body text.<br><br>Second paragraph."),
            vec![
                Block::Heading("A Heading".to_string()),
                Block::Paragraph { bold_prefix: String::new(), text: "Body text.".to_string() },
                Block::Paragraph { bold_prefix: String::new(), text: "Second paragraph.".to_string() },
            ]
        );
    }

    #[test]
    fn parse_blocks_leading_bold_becomes_prefix() {
        assert_eq!(
            parse_blocks("<strong>La Troupe Grimm</strong> - Allumez la lanterne.<br><br>Rest."),
            vec![
                Block::Paragraph {
                    bold_prefix: "La Troupe Grimm".to_string(),
                    text: "- Allumez la lanterne.".to_string(),
                },
                Block::Paragraph { bold_prefix: String::new(), text: "Rest.".to_string() },
            ]
        );
    }

    #[test]
    fn parse_blocks_list_items_keep_bold_prefix() {
        assert_eq!(
            parse_blocks("<ul><li><strong>One</strong> first</li><li>Two</li></ul>"),
            vec![
                Block::ListItem { bold_prefix: "One".to_string(), text: "first".to_string() },
                Block::ListItem { bold_prefix: String::new(), text: "Two".to_string() },
            ]
        );
    }

    #[test]
    fn parse_blocks_extracts_image_src() {
        assert_eq!(
            parse_blocks("<span class=\"bb_img_ctn\"><img class=\"bb_img\" src=\"https://x/a.jpg\" width=630 height=122 /></span>"),
            vec![Block::Image { src: "https://x/a.jpg".to_string() }]
        );
    }

    #[test]
    fn parse_blocks_mid_paragraph_bold_is_not_a_prefix() {
        assert_eq!(
            parse_blocks("Some text <strong>bold</strong> more text."),
            vec![Block::Paragraph { bold_prefix: String::new(), text: "Some text bold more text.".to_string() }]
        );
    }
}
