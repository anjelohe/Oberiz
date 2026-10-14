//! Cardigann's text selectors must constrain matches, never be discarded.
use scraper::{ElementRef, Html, Selector};

pub struct TextSelector {
    alternatives: Vec<Vec<Part>>,
    candidates: Selector,
    native: bool,
}
struct Part {
    css: Selector,
    conditions: Vec<Condition>,
    relation: char,
}
enum Condition {
    Contains(String),
    Has(Box<TextSelector>),
    Not(Box<TextSelector>),
}

fn pseudo_start(raw: &str) -> Option<usize> {
    let mut depth = 0usize;
    let mut quote = None;
    let mut escaped = false;
    for (index, c) in raw.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' {
            escaped = true;
            continue;
        }
        if let Some(q) = quote {
            if c == q {
                quote = None;
            }
            continue;
        }
        if matches!(c, '\'' | '"') {
            quote = Some(c);
            continue;
        }
        match c {
            '[' | '(' => depth += 1,
            ']' | ')' => depth = depth.saturating_sub(1),
            ':' if depth == 0 => return Some(index),
            _ => {}
        }
    }
    None
}

// Separators inside attributes, quoted strings and nested functions are literal.
fn split(raw: &str, separators: impl Fn(char) -> bool) -> Option<Vec<(String, char)>> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    let mut quote = None;
    let mut escaped = false;
    for (index, c) in raw.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' {
            escaped = true;
            continue;
        }
        if let Some(q) = quote {
            if c == q {
                quote = None;
            }
            continue;
        }
        if matches!(c, '\'' | '"') {
            quote = Some(c);
            continue;
        }
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => {
                depth = depth.checked_sub(1)?;
            }
            _ if depth == 0 && separators(c) => {
                out.push((raw[start..index].to_owned(), c));
                start = index + c.len_utf8();
            }
            _ => {}
        }
    }
    if quote.is_some() || depth != 0 || escaped {
        return None;
    }
    out.push((raw[start..].to_owned(), '\0'));
    Some(out)
}

impl TextSelector {
    pub fn parse(raw: &str) -> Option<Self> {
        Self::parse_depth(raw, 0)
    }
    fn parse_depth(raw: &str, depth: usize) -> Option<Self> {
        if depth > 8 || raw.len() > 16384 {
            return None;
        }
        if let Ok(css) = Selector::parse(raw) {
            return Some(Self {
                alternatives: vec![vec![Part {
                    css: css.clone(),
                    conditions: vec![],
                    relation: ' ',
                }]],
                candidates: css,
                native: true,
            });
        }
        let mut alternatives = Vec::new();
        for (alternative, _) in split(raw, |c| c == ',')? {
            let mut parts = Vec::new();
            let mut relation = ' ';
            for (compound, separator) in split(&alternative, |c| {
                c.is_whitespace() || matches!(c, '>' | '+' | '~')
            })? {
                if !compound.is_empty() {
                    if parts.is_empty() && relation != ' ' {
                        return None;
                    }
                    let mut css = String::new();
                    let mut conditions = Vec::new();
                    let mut remaining = compound.as_str();
                    while let Some(index) = pseudo_start(remaining) {
                        css.push_str(&remaining[..index]);
                        remaining = &remaining[index..];
                        let kind = [":contains(", ":has(", ":not("]
                            .into_iter()
                            .find(|prefix| remaining.starts_with(prefix));
                        let Some(kind) = kind else {
                            // Keep native pseudo-classes intact. Nested unsupported functions fail closed.
                            css.push(':');
                            remaining = &remaining[1..];
                            continue;
                        };
                        let args = &remaining[kind.len()..];
                        let mut nesting = 1usize;
                        let mut quote = None;
                        let mut escape = false;
                        let mut end = None;
                        for (i, c) in args.char_indices() {
                            if escape {
                                escape = false;
                                continue;
                            }
                            if c == '\\' {
                                escape = true;
                                continue;
                            }
                            if let Some(q) = quote {
                                if c == q {
                                    quote = None;
                                }
                                continue;
                            }
                            if matches!(c, '\'' | '"') {
                                quote = Some(c);
                                continue;
                            }
                            if c == '(' {
                                nesting += 1;
                            }
                            if c == ')' {
                                nesting -= 1;
                                if nesting == 0 {
                                    end = Some(i);
                                    break;
                                }
                            }
                        }
                        let end = end?;
                        let argument = args[..end].trim();
                        conditions.push(match kind {
                            ":contains(" => {
                                let text = if argument.starts_with(['\'', '"']) {
                                    if argument.len() < 2
                                        || argument.chars().next() != argument.chars().last()
                                    {
                                        return None;
                                    }
                                    &argument[1..argument.len() - 1]
                                } else {
                                    argument
                                };
                                // CSS escapes beyond simple quotes/backslashes are rejected, not broadened.
                                let mut decoded = String::new();
                                let mut chars = text.chars();
                                while let Some(c) = chars.next() {
                                    if c == '\\' {
                                        let next = chars.next()?;
                                        if !matches!(next, '\\' | '\'' | '"' | '(' | ')') {
                                            return None;
                                        }
                                        decoded.push(next);
                                    } else {
                                        decoded.push(c);
                                    }
                                }
                                Condition::Contains(decoded)
                            }
                            ":has(" => {
                                Condition::Has(Box::new(Self::parse_depth(argument, depth + 1)?))
                            }
                            _ => Condition::Not(Box::new(Self::parse_depth(argument, depth + 1)?)),
                        });
                        remaining = &args[end + 1..];
                    }
                    css.push_str(remaining);
                    let css = Selector::parse(if css.is_empty() { "*" } else { &css }).ok()?;
                    parts.push(Part {
                        css,
                        conditions,
                        relation,
                    });
                    relation = ' ';
                }
                if matches!(separator, '>' | '+' | '~') {
                    if relation != ' ' {
                        return None;
                    }
                    relation = separator;
                }
            }
            if parts.is_empty() || relation != ' ' {
                return None;
            }
            alternatives.push(parts);
        }
        Some(Self {
            alternatives,
            candidates: Selector::parse("*").ok()?,
            native: false,
        })
    }
    fn matches(&self, element: ElementRef<'_>) -> bool {
        self.alternatives
            .iter()
            .any(|parts| Self::matches_part(parts, parts.len() - 1, element))
    }
    fn matches_part(parts: &[Part], index: usize, element: ElementRef<'_>) -> bool {
        let part = &parts[index];
        if !part.css.matches(&element)
            || !part.conditions.iter().all(|condition| match condition {
                Condition::Contains(text) => element.text().collect::<String>().contains(text),
                Condition::Has(selector) => selector.select(element).next().is_some(),
                Condition::Not(selector) => !selector.matches(element),
            })
        {
            return false;
        }
        if index == 0 {
            return true;
        }
        match part.relation {
            '>' => element
                .parent()
                .and_then(ElementRef::wrap)
                .is_some_and(|parent| Self::matches_part(parts, index - 1, parent)),
            '+' => element
                .prev_siblings()
                .filter_map(ElementRef::wrap)
                .next()
                .is_some_and(|sibling| Self::matches_part(parts, index - 1, sibling)),
            '~' => element
                .prev_siblings()
                .filter_map(ElementRef::wrap)
                .any(|sibling| Self::matches_part(parts, index - 1, sibling)),
            _ => element
                .ancestors()
                .filter_map(ElementRef::wrap)
                .any(|ancestor| Self::matches_part(parts, index - 1, ancestor)),
        }
    }
    pub fn select<'a>(&'a self, root: ElementRef<'a>) -> impl Iterator<Item = ElementRef<'a>> + 'a {
        root.select(&self.candidates)
            .filter(move |element| self.native || self.matches(*element))
    }
    pub fn select_document<'a>(
        &'a self,
        document: &'a Html,
    ) -> impl Iterator<Item = ElementRef<'a>> + 'a {
        document
            .select(&self.candidates)
            .filter(move |element| self.native || self.matches(*element))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;
    #[test]
    fn text_conditions_are_preserved_on_elements_ancestors_and_siblings() {
        let doc = Html::parse_document(
            "<table><tr><td>VOSE (English)</td><td><span>one</span></td></tr><tr><td>MULTi SPANiSH</td><td><span>two</span></td></tr></table>",
        );
        for (selector, expected) in [
            ("td:contains('VOSE (English)') + td span", "one"),
            ("tr:has(td:contains(\"SPANiSH\")) span", "two"),
            ("tr:not(:contains('VOSE')) span", "two"),
            ("td:contains('missing')", ""),
        ] {
            let actual = TextSelector::parse(selector)
                .unwrap()
                .select(doc.root_element())
                .map(|e| e.text().collect::<String>())
                .collect::<String>();
            assert_eq!(actual, expected, "{selector}");
        }
    }
    #[test]
    fn invalid_conditions_never_become_unconditional_selectors() {
        for selector in [
            "td:contains('unclosed)",
            "tr:has(td:contains('x')",
            "td:unsupported(x):contains('x')",
            "td:contains('\\41')",
        ] {
            assert!(TextSelector::parse(selector).is_none(), "{selector}");
        }
    }

    #[test]
    fn attribute_literals_case_and_multiple_conditions_remain_exact() {
        let document = Html::parse_document(
            r#"<div title=":contains('literal')"><span>VOSE</span></div><div>vose</div>"#,
        );
        assert_eq!(
            TextSelector::parse(r#"div[title=":contains('literal')"]:contains('VOSE')"#)
                .unwrap()
                .select(document.root_element())
                .count(),
            1
        );
        assert_eq!(
            TextSelector::parse("div:contains('VOSE'):not(:contains('missing'))")
                .unwrap()
                .select(document.root_element())
                .count(),
            1
        );
        assert_eq!(
            TextSelector::parse("div:contains('VOSE'), div:contains('vose')")
                .unwrap()
                .select(document.root_element())
                .count(),
            2
        );
        assert!(TextSelector::parse("div:has(> span:contains('VOSE'))").is_none());
    }
    #[test]
    fn native_document_root_and_element_scope_are_preserved() {
        let document = Html::parse_document("<div><span>child</span></div>");
        assert_eq!(
            TextSelector::parse("html")
                .unwrap()
                .select_document(&document)
                .count(),
            1
        );
        let div = document
            .select(&Selector::parse("div").unwrap())
            .next()
            .unwrap();
        assert_eq!(
            TextSelector::parse(":scope > span")
                .unwrap()
                .select(div)
                .count(),
            1
        );
    }
}
