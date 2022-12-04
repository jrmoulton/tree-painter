use crate::{theme, Error, Lang};
use std::collections::HashMap;
use std::fmt::Write;
use tree_sitter_highlight::{Highlight, HighlightConfiguration, Highlighter, HtmlRenderer};

pub(crate) const HIGHLIGHT_NAMES: [&str; 40] = [
    "attribute",
    "comment",
    "constant",
    "constant.numeric",
    "constant.builtin",
    "constant.character.escape",
    "constructor",
    "function",
    "function.builtin",
    "function.macro",
    "keyword",
    "keyword.control",
    "keyword.control.import",
    "keyword.directive",
    "label",
    "namespace",
    "operator",
    "keyword.operator",
    "special",
    "string",
    "type",
    "variable",
    "variable.builtin",
    "variable.parameter",
    "variable.other.member",
    "markup.heading",
    "markup.raw.inline",
    "markup.bold",
    "markup.italic",
    "markup.list",
    "markup.quote",
    "markup.link.url",
    "markup.link.text",
    "diff.plus",
    "diff.delta",
    "diff.minus",
    "info",
    "hint",
    "warning",
    "error",
];

/// HTML syntax highlighting renderer.
pub struct Renderer {
    renderer: HtmlRenderer,
    css_classes: HashMap<usize, String>,
    configs: HashMap<Lang, HighlightConfiguration>,
}

impl Renderer {
    /// Create a new renderer based on `theme`.
    pub fn new(theme: theme::Theme) -> Self {
        let mut css_classes: HashMap<usize, String> = HashMap::default();

        for index in theme.style_map.keys() {
            css_classes.insert(
                *index,
                format!(r#"style="color: {}""#, theme.style_map[index].color),
            );
        }

        Self {
            renderer: HtmlRenderer::new(),
            css_classes,
            configs: HashMap::default(),
        }
    }

    /// Render `source` based on the `lang`.
    pub fn render<'a>(&'a mut self, lang: &Lang, source: &[u8]) -> Result<String, Error> {
        fn foo<'a>(_: &str) -> Option<&'a HighlightConfiguration> {
            None
        }

        let config = match self.configs.get(lang) {
            Some(config) => config,
            None => {
                let mut config = lang.config();
                config.configure(&HIGHLIGHT_NAMES);
                self.configs.insert(lang.clone(), config);
                self.configs.get(lang).unwrap()
            }
        };

        let mut highlighter = Highlighter::new();
        let events = highlighter.highlight(config, source, None, foo)?;

        self.renderer.render(
            events,
            source,
            &|attr: Highlight| match self.css_classes.get(&attr.0) {
                Some(class) => class.as_bytes(),
                None => "".as_bytes(),
            },
        )?;
        let mut raw_out = String::new();
        writeln!(
            &mut raw_out,
            r#"
<div class="tsc-bg">
    <table class="tsc-table">
        <tbody>"#
        )
        .unwrap();
        for (i, line) in self.renderer.lines().enumerate() {
            writeln!(
                &mut raw_out,
                "           <tr><td class=line-number>{i}</td><td class=tsc-line>{}</td></tr>",
                line.trim_end()
            )
            .unwrap();
        }

        writeln!(
            &mut raw_out,
            "        </tbody>
    </table>
</div>"
        )
        .unwrap();

        Ok(raw_out)
    }
}
