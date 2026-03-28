use pulldown_cmark::{Event, Parser, Tag, TagEnd};

pub fn markdown_to_plaintext(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut output = String::new();
    let mut in_code_block = false;

    for event in parser {
        match event {
            Event::Text(text) => {
                output.push_str(&text);
            }
            Event::Code(code) => {
                output.push_str(&code);
            }
            Event::SoftBreak | Event::HardBreak => {
                output.push('\n');
            }
            Event::Start(Tag::CodeBlock(_)) => {
                in_code_block = true;
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                output.push('\n');
            }
            Event::Start(Tag::Paragraph) => {
                if !output.is_empty() && !in_code_block {
                    output.push('\n');
                }
            }
            Event::End(TagEnd::Paragraph) => {
                output.push('\n');
            }
            _ => {}
        }
    }

    output.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_paragraph() {
        assert_eq!(markdown_to_plaintext("Hello world"), "Hello world");
    }

    #[test]
    fn heading_extracted_as_text() {
        assert_eq!(markdown_to_plaintext("# Title"), "Title");
    }

    #[test]
    fn bold_and_italic_stripped() {
        assert_eq!(markdown_to_plaintext("**bold** and *italic*"), "bold and italic");
    }

    #[test]
    fn inline_code_preserved() {
        assert_eq!(markdown_to_plaintext("Use `println!`"), "Use println!");
    }

    #[test]
    fn code_block_content_preserved() {
        let md = "```rust\nfn main() {}\n```";
        let result = markdown_to_plaintext(md);
        assert!(result.contains("fn main() {}"));
    }

    #[test]
    fn multiple_paragraphs() {
        let md = "First paragraph\n\nSecond paragraph";
        let result = markdown_to_plaintext(md);
        assert!(result.contains("First paragraph"));
        assert!(result.contains("Second paragraph"));
    }

    #[test]
    fn link_text_extracted() {
        assert_eq!(markdown_to_plaintext("[click here](http://example.com)"), "click here");
    }

    #[test]
    fn empty_input() {
        assert_eq!(markdown_to_plaintext(""), "");
    }

    #[test]
    fn list_items() {
        let md = "- item1\n- item2";
        let result = markdown_to_plaintext(md);
        assert!(result.contains("item1"));
        assert!(result.contains("item2"));
    }
}
