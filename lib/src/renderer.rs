use crate::{theme, Error, Lang};
use std::collections::HashMap;
use std::fmt::Write;
use tree_sitter_highlight::{Highlight, HighlightConfiguration, Highlighter, HtmlRenderer};

pub(crate) const HIGHLIGHT_NAMES: [&str; 59] = [
    "annotation",
    "attribute",
    "boolean",
    "character",
    "comment",
    "conditional",
    "constant",
    "constant.builtin",
    "constant.macro",
    "constructor",
    "error",
    "exception",
    "field",
    "float",
    "function",
    "function.builtin",
    "function.macro",
    "include",
    "keyword",
    "keyword.function",
    "keyword.operator",
    "label",
    "method",
    "namespace",
    "none",
    "number",
    "operator",
    "parameter",
    "parameter.reference",
    "property",
    "punctuation.delimiter",
    "punctuation.bracket",
    "punctuation.special",
    "repeat",
    "string",
    "string.regex",
    "string.escape",
    "symbol",
    "tag",
    "tag.delimiter",
    "text",
    "text.strong",
    "text.emphasis",
    "text.underline",
    "text.strike",
    "text.title",
    "text.literal",
    "text.uri",
    "text.math",
    "text.reference",
    "text.enviroment",
    "text.enviroment.name",
    "note",
    "warning",
    "danger",
    "type",
    "type.builtin",
    "variable",
    "variable.builtin",
];

/// HTML syntax highlighting renderer.
pub struct Renderer {
    renderer: HtmlRenderer,
    theme: theme::Theme,
    css_classes: HashMap<usize, String>,
    configs: HashMap<Lang, HighlightConfiguration>,
}

impl Renderer {
    /// Create a new renderer based on `theme`.
    pub fn new(theme: theme::Theme) -> Self {
        let mut css_classes = HashMap::default();

        for index in theme.style_map.keys() {
            css_classes.insert(
                *index,
                format!(
                    r#"class="tsc-{}""#,
                    HIGHLIGHT_NAMES[*index].replace('.', "_")
                ),
            );
        }

        Self {
            renderer: HtmlRenderer::new(),
            theme,
            css_classes,
            configs: HashMap::default(),
        }
    }

    /// Generate CSS block to be included in the `<style></style>` block or in an external CSS
    /// file.
    pub fn css(&self) -> String {
        let mut root_str = String::new();
        let mut css = String::new();

        root_str.push_str(&format!(
            ":root {{ --tsc-main-fg-color: {}; --tsc-main-bg-color: {}; ",
            self.theme.foreground.color, self.theme.background.color,
        ));
        for (name, color) in &self.theme.palette {
            root_str.push_str(&format!("--{}: {}; ", name, color.as_str().unwrap()));
        }

        for (index, style) in &self.theme.style_map {
            root_str.push_str(&format!(
                "--tsc-{}: {}; ",
                HIGHLIGHT_NAMES[*index].replace('.', "_"),
                style.color
            ));
            let _ = write!(
                css,
                ".tsc-{} {{ color: var(--tsc-{}); ",
                HIGHLIGHT_NAMES[*index].replace('.', "_"),
                HIGHLIGHT_NAMES[*index].replace('.', "_")
            );

            if style.is_bold {
                css.push_str("font-weight: bold;");
            }

            if style.is_italic {
                css.push_str("font-style: italic;");
            }

            css.push_str("}\n");
        }

        css.push_str(".tsc-line { word-wrap: normal; white-space: pre; }\n");
        root_str.push_str("}\n");
        root_str + &css
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

        self.renderer.reset();
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
<div class="tsc-table-bg">
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
