use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiLinkRef {
    pub target: String,
    pub alias: Option<String>,
}

fn pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"\[\[([^\]|]+)(?:\|([^\]]+))?\]\]").expect("wiki-link regex is valid")
    })
}

pub fn extract_wiki_links(content: &str) -> Vec<WikiLinkRef> {
    pattern()
        .captures_iter(content)
        .map(|caps| WikiLinkRef {
            target: caps
                .get(1)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default(),
            alias: caps.get(2).map(|m| m.as_str().trim().to_string()),
        })
        .filter(|link| !link.target.is_empty())
        .collect()
}

pub fn strip_wiki_link_syntax(text: &str) -> String {
    pattern()
        .replace_all(text, |caps: &regex::Captures| {
            caps.get(2)
                .or_else(|| caps.get(1))
                .map(|m| m.as_str().to_string())
                .unwrap_or_default()
        })
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_simple_links() {
        let content = "See [[Project Alpha]] and [[Project Beta]] for details.";
        let links = extract_wiki_links(content);
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].target, "Project Alpha");
        assert_eq!(links[0].alias, None);
        assert_eq!(links[1].target, "Project Beta");
    }

    #[test]
    fn extracts_aliased_links() {
        let content = "Read the [[Research/Findings|latest findings]] first.";
        let links = extract_wiki_links(content);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "Research/Findings");
        assert_eq!(links[0].alias, Some("latest findings".to_string()));
    }

    #[test]
    fn ignores_empty_or_malformed_brackets() {
        let content = "Not a link: [single] or [[]] or just text.";
        let links = extract_wiki_links(content);
        assert!(links.is_empty());
    }

    #[test]
    fn counts_duplicate_links_separately() {
        let content = "[[Same Note]] mentioned twice: [[Same Note]].";
        let links = extract_wiki_links(content);
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].target, links[1].target);
    }

    #[test]
    fn handles_links_at_start_and_end_of_content() {
        let content = "[[First]] middle text [[Last]]";
        let links = extract_wiki_links(content);
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].target, "First");
        assert_eq!(links[1].target, "Last");
    }

    #[test]
    fn strip_wiki_link_syntax_uses_alias_when_present() {
        let text = "Read the [[Research/Findings|latest findings]] first.";
        assert_eq!(
            strip_wiki_link_syntax(text),
            "Read the latest findings first."
        );
    }

    #[test]
    fn strip_wiki_link_syntax_uses_target_when_no_alias() {
        let text = "See [[Project Alpha]] for details.";
        assert_eq!(
            strip_wiki_link_syntax(text),
            "See Project Alpha for details."
        );
    }

    #[test]
    fn strip_wiki_link_syntax_handles_multiple_links() {
        let text = "[[A]] and [[B|second]] and [[C]]";
        assert_eq!(strip_wiki_link_syntax(text), "A and second and C");
    }

    #[test]
    fn strip_wiki_link_syntax_leaves_plain_text_untouched() {
        let text = "No links here at all.";
        assert_eq!(strip_wiki_link_syntax(text), text);
    }
}
