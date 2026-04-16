use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

pub(super) fn idea_template(abstract_text: Option<&str>, content_text: Option<&str>) -> String {
    let mut markdown = String::from("## Abstract\n\n");

    if let Some(text) = abstract_text.filter(|text| !text.is_empty()) {
        markdown.push_str(text);
        if !text.ends_with('\n') {
            markdown.push('\n');
        }
        markdown.push('\n');
    }

    markdown.push_str("## Content\n");

    if let Some(text) = content_text.filter(|text| !text.is_empty()) {
        markdown.push('\n');
        markdown.push_str(text);
        if !text.ends_with('\n') {
            markdown.push('\n');
        }
    }

    markdown
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum IdeaSection {
    Abstract,
    Content,
}

#[derive(Debug, Eq, PartialEq)]
pub(super) struct IdeaDocument {
    pub(super) abstract_text: String,
    pub(super) content_text: String,
}

pub(super) fn parse_idea_document(markdown: &str) -> Result<IdeaDocument, String> {
    let mut current_section = None;
    let mut saw_abstract = false;
    let mut saw_content = false;
    let mut heading_level = None;
    let mut heading_text = String::new();
    let mut abstract_text = String::new();
    let mut content_text = String::new();

    for event in Parser::new(markdown) {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                heading_level = Some(level);
                heading_text.clear();
            }
            Event::End(TagEnd::Heading(_)) => {
                let level = heading_level
                    .take()
                    .ok_or_else(|| "invalid markdown heading state".to_string())?;
                let text = heading_text.trim().to_string();

                match level {
                    HeadingLevel::H1 => {
                        return Err("idea files do not allow level-1 headings".to_string());
                    }
                    HeadingLevel::H2 => match text.as_str() {
                        "Abstract" if !saw_abstract && !saw_content => {
                            saw_abstract = true;
                            current_section = Some(IdeaSection::Abstract);
                        }
                        "Abstract" => {
                            return Err("`Abstract` must come first and appear once".to_string());
                        }
                        "Content" if saw_abstract && !saw_content => {
                            saw_content = true;
                            current_section = Some(IdeaSection::Content);
                        }
                        "Content" if !saw_abstract => {
                            return Err("`Abstract` must come first".to_string());
                        }
                        "Content" => {
                            return Err("`Content` must come second and appear once".to_string());
                        }
                        _ => {
                            return Err(format!("unknown level-2 heading `{text}`"));
                        }
                    },
                    HeadingLevel::H3 | HeadingLevel::H4 | HeadingLevel::H5 | HeadingLevel::H6 => {
                        if current_section != Some(IdeaSection::Content) {
                            return Err(
                                "nested headings are only allowed under `Content`".to_string()
                            );
                        }
                    }
                }
            }
            _ => {
                if heading_level.is_some() {
                    push_event_text(&mut heading_text, &event);
                    continue;
                }

                let Some(section) = current_section else {
                    if event_has_non_whitespace_text(&event) {
                        return Err(
                            "content must belong to either `Abstract` or `Content`".to_string()
                        );
                    }

                    continue;
                };

                match section {
                    IdeaSection::Abstract => push_section_event_text(&mut abstract_text, &event),
                    IdeaSection::Content => push_section_event_text(&mut content_text, &event),
                }
            }
        }
    }

    if !saw_abstract {
        return Err("missing required level-2 heading `Abstract`".to_string());
    }

    if !saw_content {
        return Err("missing required level-2 heading `Content`".to_string());
    }

    Ok(IdeaDocument {
        abstract_text: abstract_text.trim().to_string(),
        content_text: content_text.trim().to_string(),
    })
}

fn push_event_text(buffer: &mut String, event: &Event<'_>) {
    match event {
        Event::Text(text) | Event::Code(text) | Event::Html(text) | Event::InlineHtml(text) => {
            buffer.push_str(text);
        }
        Event::SoftBreak | Event::HardBreak => buffer.push(' '),
        _ => {}
    }
}

fn push_section_event_text(buffer: &mut String, event: &Event<'_>) {
    match event {
        Event::Text(text) | Event::Code(text) | Event::Html(text) | Event::InlineHtml(text) => {
            buffer.push_str(text);
        }
        Event::SoftBreak | Event::HardBreak => buffer.push('\n'),
        Event::End(TagEnd::Paragraph)
        | Event::End(TagEnd::BlockQuote(_))
        | Event::End(TagEnd::CodeBlock)
        | Event::End(TagEnd::Item)
        | Event::End(TagEnd::List(_)) => buffer.push('\n'),
        _ => {}
    }
}

fn event_has_non_whitespace_text(event: &Event<'_>) -> bool {
    match event {
        Event::Text(text) | Event::Code(text) | Event::Html(text) | Event::InlineHtml(text) => {
            !text.trim().is_empty()
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{idea_template, parse_idea_document};

    #[test]
    fn idea_parser_accepts_required_template() {
        let document = parse_idea_document(
            "## Abstract\n\nShort summary of the idea.\n\n## Content\n\nMain idea text.\n\n### Details\n\nMore detail.\n",
        )
        .expect("idea document should parse");

        assert_eq!(document.abstract_text, "Short summary of the idea.");
        assert!(document.content_text.contains("Main idea text."));
    }

    #[test]
    fn idea_template_accepts_empty_sections() {
        assert_eq!(idea_template(None, None), "## Abstract\n\n## Content\n");
    }

    #[test]
    fn idea_template_fills_provided_sections() {
        assert_eq!(
            idea_template(Some("Short summary."), Some("Detailed content.")),
            "## Abstract\n\nShort summary.\n\n## Content\n\nDetailed content.\n"
        );
    }

    #[test]
    fn idea_parser_rejects_extra_level_two_heading() {
        let error = parse_idea_document(
            "## Abstract\n\nShort summary of the idea.\n\n## Content\n\nMain idea text.\n\n## Notes\n\nMore detail.\n",
        )
        .expect_err("extra section should be rejected");

        assert!(error.contains("unknown level-2 heading `Notes`"));
    }

    #[test]
    fn idea_parser_rejects_nested_heading_under_abstract() {
        let error = parse_idea_document(
            "## Abstract\n\nShort summary of the idea.\n\n### Details\n\nMore detail.\n\n## Content\n\nMain idea text.\n",
        )
        .expect_err("nested heading under abstract should be rejected");

        assert!(error.contains("nested headings are only allowed under `Content`"));
    }
}
