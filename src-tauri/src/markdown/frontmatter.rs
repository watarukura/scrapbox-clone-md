use serde_json::Value as JsonValue;

pub struct ParsedMarkdown {
    pub frontmatter_json: Option<String>,
    pub body: String,
}

pub fn parse_frontmatter(content: &str) -> ParsedMarkdown {
    if !content.starts_with("---") {
        return ParsedMarkdown {
            frontmatter_json: None,
            body: content.to_string(),
        };
    }

    let rest = &content[3..];
    if let Some(end) = rest.find("\n---") {
        let yaml_str = &rest[..end].trim();
        let body = rest[end + 4..].trim_start_matches('\n').to_string();

        let frontmatter_json = serde_yaml::from_str::<serde_yaml::Value>(yaml_str)
            .ok()
            .and_then(|v| {
                let json: JsonValue = serde_json::to_value(v).ok()?;
                serde_json::to_string(&json).ok()
            });

        ParsedMarkdown {
            frontmatter_json,
            body,
        }
    } else {
        ParsedMarkdown {
            frontmatter_json: None,
            body: content.to_string(),
        }
    }
}

pub fn extract_title(frontmatter_json: &Option<String>, body: &str, file_name: &str) -> String {
    // 1. frontmatter.title
    if let Some(json_str) = frontmatter_json {
        if let Ok(val) = serde_json::from_str::<JsonValue>(json_str) {
            if let Some(title) = val.get("title").and_then(|t| t.as_str()) {
                if !title.is_empty() {
                    return title.to_string();
                }
            }
        }
    }

    // 2. first H1
    for line in body.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("# ") {
            let title = rest.trim();
            if !title.is_empty() {
                return title.to_string();
            }
        }
    }

    // 3. filename without extension
    file_name
        .strip_suffix(".md")
        .unwrap_or(file_name)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_frontmatter_with_valid_yaml() {
        let content = "---\ntitle: Hello\ntags:\n  - rust\n---\n# Body\nSome text";
        let result = parse_frontmatter(content);
        assert!(result.frontmatter_json.is_some());
        let json: serde_json::Value =
            serde_json::from_str(result.frontmatter_json.as_ref().unwrap()).unwrap();
        assert_eq!(json["title"], "Hello");
        assert_eq!(result.body, "# Body\nSome text");
    }

    #[test]
    fn parse_frontmatter_without_frontmatter() {
        let content = "# Just a heading\nSome text";
        let result = parse_frontmatter(content);
        assert!(result.frontmatter_json.is_none());
        assert_eq!(result.body, content);
    }

    #[test]
    fn parse_frontmatter_with_unclosed_delimiter() {
        let content = "---\ntitle: Hello\nNo closing delimiter";
        let result = parse_frontmatter(content);
        assert!(result.frontmatter_json.is_none());
        assert_eq!(result.body, content);
    }

    #[test]
    fn parse_frontmatter_empty_content() {
        let result = parse_frontmatter("");
        assert!(result.frontmatter_json.is_none());
        assert_eq!(result.body, "");
    }

    #[test]
    fn parse_frontmatter_empty_yaml_block() {
        let content = "---\n---\nBody here";
        let result = parse_frontmatter(content);
        assert!(result.frontmatter_json.is_some());
        assert_eq!(result.body, "Body here");
    }

    #[test]
    fn extract_title_from_frontmatter() {
        let fm = Some(r#"{"title":"My Title"}"#.to_string());
        assert_eq!(extract_title(&fm, "# Heading", "file.md"), "My Title");
    }

    #[test]
    fn extract_title_from_h1_when_no_frontmatter_title() {
        let fm = Some(r#"{"tags":["a"]}"#.to_string());
        assert_eq!(extract_title(&fm, "# First Heading\ntext", "file.md"), "First Heading");
    }

    #[test]
    fn extract_title_from_h1_when_no_frontmatter() {
        assert_eq!(extract_title(&None, "# My Heading\ntext", "file.md"), "My Heading");
    }

    #[test]
    fn extract_title_falls_back_to_filename() {
        assert_eq!(extract_title(&None, "No heading here", "my-note.md"), "my-note");
    }

    #[test]
    fn extract_title_filename_without_md_extension() {
        assert_eq!(extract_title(&None, "No heading", "readme.txt"), "readme.txt");
    }

    #[test]
    fn extract_title_skips_empty_frontmatter_title() {
        let fm = Some(r#"{"title":""}"#.to_string());
        assert_eq!(extract_title(&fm, "# Fallback", "file.md"), "Fallback");
    }

    #[test]
    fn extract_title_skips_empty_h1() {
        assert_eq!(extract_title(&None, "# \nsome text", "note.md"), "note");
    }
}
