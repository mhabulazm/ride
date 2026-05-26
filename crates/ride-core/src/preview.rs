use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewStyle {
    Normal,
    Heading(u8),
    Bold,
    Italic,
    Code,
    Link,
    ListItem,
    BlockQuote,
    Rule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewSpan {
    pub text: String,
    pub style: PreviewStyle,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PreviewLine {
    pub spans: Vec<PreviewSpan>,
}

fn heading_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn inline_style(heading: Option<u8>, bold: usize, italic: usize, in_link: bool) -> PreviewStyle {
    if let Some(l) = heading {
        PreviewStyle::Heading(l)
    } else if in_link {
        PreviewStyle::Link
    } else if bold > 0 {
        PreviewStyle::Bold
    } else if italic > 0 {
        PreviewStyle::Italic
    } else {
        PreviewStyle::Normal
    }
}

/// Render Markdown source into a UI-agnostic model of styled terminal lines.
pub fn render_markdown(source: &str) -> Vec<PreviewLine> {
    let mut lines: Vec<PreviewLine> = Vec::new();
    let mut cur: Vec<PreviewSpan> = Vec::new();

    let mut bold = 0usize;
    let mut italic = 0usize;
    let mut in_link = false;
    let mut heading: Option<u8> = None;
    let mut in_code_block = false;
    let mut list_stack: Vec<Option<u64>> = Vec::new();

    let flush = |cur: &mut Vec<PreviewSpan>, lines: &mut Vec<PreviewLine>| {
        if !cur.is_empty() {
            lines.push(PreviewLine { spans: std::mem::take(cur) });
        }
    };

    for ev in Parser::new(source) {
        match ev {
            Event::Start(Tag::Heading { level, .. }) => {
                let l = heading_u8(level);
                heading = Some(l);
                cur.push(PreviewSpan {
                    text: format!("{} ", "#".repeat(l as usize)),
                    style: PreviewStyle::Heading(l),
                });
            }
            Event::Start(Tag::Emphasis) => italic += 1,
            Event::Start(Tag::Strong) => bold += 1,
            Event::Start(Tag::BlockQuote(_)) => {
                cur.push(PreviewSpan { text: "▌ ".to_string(), style: PreviewStyle::BlockQuote });
            }
            Event::Start(Tag::CodeBlock(_)) => in_code_block = true,
            Event::Start(Tag::List(start)) => list_stack.push(start),
            Event::Start(Tag::Item) => {
                let indent = "  ".repeat(list_stack.len().saturating_sub(1));
                let marker = match list_stack.last().copied().flatten() {
                    Some(n) => format!("{}. ", n),
                    None => "• ".to_string(),
                };
                cur.push(PreviewSpan {
                    text: format!("{}{}", indent, marker),
                    style: PreviewStyle::ListItem,
                });
            }
            Event::Start(Tag::Link { .. }) => in_link = true,

            Event::End(TagEnd::Heading(_)) => {
                heading = None;
                flush(&mut cur, &mut lines);
            }
            Event::End(TagEnd::Paragraph) => flush(&mut cur, &mut lines),
            Event::End(TagEnd::Emphasis) => italic = italic.saturating_sub(1),
            Event::End(TagEnd::Strong) => bold = bold.saturating_sub(1),
            Event::End(TagEnd::BlockQuote(_)) => flush(&mut cur, &mut lines),
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                flush(&mut cur, &mut lines);
            }
            Event::End(TagEnd::List(_)) => {
                list_stack.pop();
            }
            Event::End(TagEnd::Item) => flush(&mut cur, &mut lines),
            Event::End(TagEnd::Link) => in_link = false,

            Event::Text(t) => {
                if in_code_block {
                    let parts: Vec<&str> = t.split('\n').collect();
                    for (k, part) in parts.iter().enumerate() {
                        if k > 0 {
                            flush(&mut cur, &mut lines);
                        }
                        if !part.is_empty() {
                            cur.push(PreviewSpan {
                                text: part.to_string(),
                                style: PreviewStyle::Code,
                            });
                        }
                    }
                } else {
                    let style = inline_style(heading, bold, italic, in_link);
                    cur.push(PreviewSpan { text: t.to_string(), style });
                }
            }
            Event::Code(t) => {
                cur.push(PreviewSpan { text: t.to_string(), style: PreviewStyle::Code });
            }
            Event::SoftBreak | Event::HardBreak => {
                cur.push(PreviewSpan { text: " ".to_string(), style: PreviewStyle::Normal });
            }
            Event::Rule => {
                flush(&mut cur, &mut lines);
                cur.push(PreviewSpan { text: "─".repeat(40), style: PreviewStyle::Rule });
                flush(&mut cur, &mut lines);
            }
            _ => {}
        }
    }
    flush(&mut cur, &mut lines);
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_of(lines: &[PreviewLine]) -> String {
        lines
            .iter()
            .map(|l| l.spans.iter().map(|s| s.text.as_str()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn test_heading_styled() {
        let lines = render_markdown("# Title");
        assert!(lines[0].spans.iter().any(|s| s.style == PreviewStyle::Heading(1)));
        assert!(text_of(&lines).contains("Title"));
    }

    #[test]
    fn test_bold_and_italic() {
        let lines = render_markdown("**b** and *i*");
        assert!(lines.iter().flat_map(|l| &l.spans).any(|s| s.style == PreviewStyle::Bold));
        assert!(lines.iter().flat_map(|l| &l.spans).any(|s| s.style == PreviewStyle::Italic));
    }

    #[test]
    fn test_unordered_list_marker() {
        let lines = render_markdown("- one\n- two");
        assert!(text_of(&lines).contains("• one"));
        assert!(text_of(&lines).contains("• two"));
    }

    #[test]
    fn test_ordered_list_marker() {
        let lines = render_markdown("1. first");
        assert!(text_of(&lines).contains("1. first"));
    }

    #[test]
    fn test_code_block() {
        let lines = render_markdown("```\nlet x = 1;\n```");
        assert!(lines.iter().flat_map(|l| &l.spans).any(|s| s.style == PreviewStyle::Code));
        assert!(text_of(&lines).contains("let x = 1;"));
    }

    #[test]
    fn test_blockquote() {
        let lines = render_markdown("> quoted");
        assert!(lines.iter().flat_map(|l| &l.spans).any(|s| s.style == PreviewStyle::BlockQuote));
        assert!(text_of(&lines).contains("quoted"));
    }

    #[test]
    fn test_link_text_preserved() {
        let lines = render_markdown("[click](http://example.com)");
        assert!(lines.iter().flat_map(|l| &l.spans).any(|s| s.style == PreviewStyle::Link));
        assert!(text_of(&lines).contains("click"));
    }

    #[test]
    fn test_thematic_break() {
        let lines = render_markdown("a\n\n---\n\nb");
        assert!(lines.iter().flat_map(|l| &l.spans).any(|s| s.style == PreviewStyle::Rule));
    }
}
