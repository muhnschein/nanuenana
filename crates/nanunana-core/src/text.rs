// SPDX-License-Identifier: GPL-3.0-or-later

use crate::model::Text;

const ENTITIES: &[(&str, &str)] = &[
    ("&nbsp;", " "),
    ("&lt;", "<"),
    ("&gt;", ">"),
    ("&quot;", "\""),
    ("&#39;", "'"),
    ("&apos;", "'"),
    ("&amp;", "&"),
];
const BREAKING: &[&str] = &["br", "p", "div", "li", "tr"];

fn tag_name(tag: &str) -> String {
    tag.trim_start_matches('/')
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase()
}

fn without_tags(markup: &str) -> String {
    let mut out = String::with_capacity(markup.len());
    let mut rest = markup;
    while let Some((before, after)) = rest.split_once('<') {
        let named = after.starts_with(|c: char| c.is_ascii_alphabetic() || c == '/');
        out.push_str(before);
        match after.split_once('>') {
            Some((tag, tail)) if named => {
                if BREAKING.contains(&tag_name(tag).as_str()) {
                    out.push('\n');
                }
                rest = tail;
            }
            _ => {
                out.push('<');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

fn tidy(text: &str) -> String {
    let mut lines: Vec<&str> = Vec::new();
    for line in text.lines().map(str::trim) {
        if line.is_empty() && lines.last().is_none_or(|last| last.is_empty()) {
            continue;
        }
        lines.push(line);
    }
    while lines.last().is_some_and(|last| last.is_empty()) {
        lines.pop();
    }
    lines.join("\n")
}

pub fn plain(markup: &str) -> String {
    let mut text = without_tags(markup);
    for (entity, replacement) in ENTITIES {
        text = text.replace(entity, replacement);
    }
    tidy(&text)
}

fn primary(language: &str) -> String {
    language
        .split(['-', '_'])
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase()
}

fn plain_variant(language: &str) -> bool {
    !language.contains(['-', '_']) || language.len() == 5 && !language.to_ascii_lowercase().ends_with("ls")
}

pub fn original(texts: &[Text]) -> Option<&Text> {
    texts
        .iter()
        .find(|t| primary(&t.language) == "de" && plain_variant(&t.language))
        .or_else(|| texts.first())
}

pub fn pick<'a>(texts: &'a [Text], wanted: &str) -> Option<&'a Text> {
    let wanted = primary(wanted);
    texts
        .iter()
        .find(|t| !wanted.is_empty() && primary(&t.language) == wanted && plain_variant(&t.language))
        .or_else(|| original(texts))
}

pub fn languages(texts: &[Text]) -> Vec<String> {
    texts.iter().map(|t| t.language.clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(language: &str) -> Text {
        Text {
            language: language.to_owned(),
            headline: format!("headline {language}"),
            ..Text::default()
        }
    }

    #[test]
    fn breaks_become_lines() {
        assert_eq!(plain("a<br/>b<br>c<BR />d"), "a\nb\nc\nd");
        assert_eq!(plain("<p>one</p><p>two</p>"), "one\n\ntwo");
        assert_eq!(plain("<ul><li>x</li><li>y</li></ul>"), "x\n\ny");
    }

    #[test]
    fn other_tags_vanish() {
        assert_eq!(
            plain("<b>Achtung</b>: <a href=\"https://x\">hier</a>"),
            "Achtung: hier"
        );
        assert_eq!(plain("<span class='a'>x</span>y"), "xy");
    }

    #[test]
    fn stray_angle_brackets_stay() {
        assert_eq!(plain("Pegel < 3 m und > 1 m"), "Pegel < 3 m und > 1 m");
        assert_eq!(plain("a < b"), "a < b");
        assert_eq!(plain("offen <"), "offen <");
        assert_eq!(plain("5 <6> 7"), "5 <6> 7");
    }

    #[test]
    fn entities_are_decoded_once() {
        assert_eq!(
            plain("A &amp; B &lt;C&gt; &quot;D&quot; &#39;E&#39;&nbsp;F"),
            "A & B <C> \"D\" 'E' F"
        );
        assert_eq!(plain("&amp;lt;"), "&lt;");
    }

    #[test]
    fn blank_runs_collapse_and_edges_trim() {
        assert_eq!(plain("  a  <br/><br/><br/><br/>  b  <br/>"), "a\n\nb");
        assert_eq!(plain("<br/><br/>a"), "a");
        assert_eq!(plain(""), "");
    }

    #[test]
    fn pick_prefers_wanted_language() {
        let texts = [text("de"), text("de-LS"), text("en"), text("tr")];
        assert_eq!(pick(&texts, "en_GB").unwrap().language, "en");
        assert_eq!(pick(&texts, "tr").unwrap().language, "tr");
        assert_eq!(pick(&texts, "de_DE").unwrap().language, "de");
    }

    #[test]
    fn pick_falls_back_to_german_then_first() {
        let texts = [text("en"), text("de-LS"), text("de")];
        assert_eq!(pick(&texts, "fi").unwrap().language, "de");
        assert_eq!(pick(&texts, "").unwrap().language, "de");
        let foreign = [text("pl"), text("en")];
        assert_eq!(pick(&foreign, "fi").unwrap().language, "pl");
        assert!(pick(&[], "de").is_none());
    }

    #[test]
    fn easy_language_is_not_the_original() {
        let texts = [text("de-LS"), text("de-DE")];
        assert_eq!(original(&texts).unwrap().language, "de-DE");
        assert_eq!(pick(&texts, "de").unwrap().language, "de-DE");
        let only_easy = [text("de-LS")];
        assert_eq!(original(&only_easy).unwrap().language, "de-LS");
    }

    #[test]
    fn languages_lists_all_blocks() {
        let texts = [text("de"), text("en")];
        assert_eq!(languages(&texts), vec!["de".to_owned(), "en".to_owned()]);
    }
}
