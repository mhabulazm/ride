use serde::Deserialize;

/// A single color+modifier style definition.
#[derive(Debug, Clone, Deserialize)]
pub struct ColorStyle {
    #[serde(default)]
    pub fg: Option<String>,
    #[serde(default)]
    pub bg: Option<String>,
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub italic: bool,
    #[serde(default)]
    pub underline: bool,
}

impl ColorStyle {
    pub const fn new() -> Self {
        Self {
            fg: None,
            bg: None,
            bold: false,
            italic: false,
            underline: false,
        }
    }

    pub fn fg(color: &str) -> Self {
        Self {
            fg: Some(color.to_string()),
            bg: None,
            bold: false,
            italic: false,
            underline: false,
        }
    }

    pub fn fg_bold(color: &str) -> Self {
        Self {
            fg: Some(color.to_string()),
            bg: None,
            bold: true,
            italic: false,
            underline: false,
        }
    }

    pub fn fg_italic(color: &str) -> Self {
        Self {
            fg: Some(color.to_string()),
            bg: None,
            bold: false,
            italic: true,
            underline: false,
        }
    }

    pub fn fg_underline(color: &str) -> Self {
        Self {
            fg: Some(color.to_string()),
            bg: None,
            bold: false,
            italic: false,
            underline: true,
        }
    }

    pub fn fg_bg(fg: &str, bg: &str) -> Self {
        Self {
            fg: Some(fg.to_string()),
            bg: Some(bg.to_string()),
            bold: false,
            italic: false,
            underline: false,
        }
    }

    pub fn fg_bg_bold(fg: &str, bg: &str) -> Self {
        Self {
            fg: Some(fg.to_string()),
            bg: Some(bg.to_string()),
            bold: true,
            italic: false,
            underline: false,
        }
    }

    fn italic_only() -> Self {
        Self {
            fg: None,
            bg: None,
            bold: false,
            italic: true,
            underline: false,
        }
    }
}

/// Syntax highlighting colors.
#[derive(Debug, Clone, Deserialize)]
pub struct SyntaxColors {
    pub keyword: ColorStyle,
    pub type_name: ColorStyle,
    pub string: ColorStyle,
    pub comment: ColorStyle,
    pub number: ColorStyle,
    pub function: ColorStyle,
    pub operator: ColorStyle,
    pub punctuation: ColorStyle,
    pub variable: ColorStyle,
    pub heading: ColorStyle,
    pub link: ColorStyle,
    pub emphasis: ColorStyle,
    pub mermaid_keyword: ColorStyle,
    pub mermaid_arrow: ColorStyle,
    pub log_error: ColorStyle,
    pub log_warn: ColorStyle,
    pub log_info: ColorStyle,
    pub log_debug: ColorStyle,
    pub log_timestamp: ColorStyle,
    pub normal: ColorStyle,
}

/// UI chrome colors.
#[derive(Debug, Clone, Deserialize)]
pub struct UiColors {
    // Borders
    pub border_focused: String,
    pub border_unfocused: String,

    // Editor
    pub line_number: String,
    pub line_number_active: ColorStyle,
    pub bracket_match: ColorStyle,
    pub fold_indicator: ColorStyle,
    pub tilde_empty: String,
    pub wrap_gutter: String,

    // Diagnostics
    pub diagnostic_error: ColorStyle,
    pub diagnostic_warning: ColorStyle,
    pub diagnostic_info: ColorStyle,
    pub diagnostic_hint: ColorStyle,

    // Welcome screen
    pub welcome_title: ColorStyle,
    pub welcome_key: ColorStyle,
    pub welcome_desc: ColorStyle,
    pub welcome_section: ColorStyle,

    // Status bar
    pub status_bar_bg: String,
    pub status_label: ColorStyle,
    pub status_file: ColorStyle,
    pub status_position: ColorStyle,
    pub status_message: ColorStyle,
    pub status_hover: ColorStyle,

    // Tabs
    pub tab_active: ColorStyle,
    pub tab_inactive: ColorStyle,
    pub tab_bar_bg: String,

    // Explorer
    pub explorer_title: ColorStyle,
    pub explorer_dir: ColorStyle,
    pub explorer_file: ColorStyle,
    pub explorer_selected: ColorStyle,

    // Search bar
    pub search_label: ColorStyle,
    pub search_query: ColorStyle,
    pub search_match_count: ColorStyle,

    // Fuzzy finder
    pub fuzzy_border: String,
    pub fuzzy_title: ColorStyle,
    pub fuzzy_prompt: ColorStyle,
    pub fuzzy_match_count: ColorStyle,
    pub fuzzy_selected: ColorStyle,
    pub fuzzy_item: ColorStyle,

    // Go-to-line
    pub goto_border: String,
    pub goto_title: ColorStyle,
    pub goto_prompt: ColorStyle,

    // Completion popup
    pub completion_border: String,
    pub completion_bg: String,
    pub completion_selected: ColorStyle,
    pub completion_item: ColorStyle,
}

/// The full theme.
#[derive(Debug, Clone, Deserialize)]
pub struct Theme {
    pub name: String,
    pub syntax: SyntaxColors,
    pub ui: UiColors,
}

impl Theme {
    /// Look up a built-in theme by name.
    pub fn builtin(name: &str) -> Option<Self> {
        match name {
            "dark" => Some(dark_theme()),
            "light" => Some(light_theme()),
            "monokai" => Some(monokai_theme()),
            "solarized-dark" => Some(solarized_dark_theme()),
            _ => None,
        }
    }

    /// List available built-in theme names.
    pub fn builtin_names() -> &'static [&'static str] {
        &["dark", "light", "monokai", "solarized-dark"]
    }
}

impl Default for Theme {
    fn default() -> Self {
        dark_theme()
    }
}

// ---------------------------------------------------------------------------
// Built-in themes
// ---------------------------------------------------------------------------

pub fn dark_theme() -> Theme {
    Theme {
        name: "dark".to_string(),
        syntax: SyntaxColors {
            keyword: ColorStyle::fg_bold("magenta"),
            type_name: ColorStyle::fg("cyan"),
            string: ColorStyle::fg("green"),
            comment: ColorStyle::fg_italic("darkgray"),
            number: ColorStyle::fg("yellow"),
            function: ColorStyle::fg("blue"),
            operator: ColorStyle::fg("red"),
            punctuation: ColorStyle::fg("white"),
            variable: ColorStyle::fg("white"),
            heading: ColorStyle::fg_bold("cyan"),
            link: ColorStyle::fg_underline("blue"),
            emphasis: ColorStyle::italic_only(),
            mermaid_keyword: ColorStyle::fg_bold("magenta"),
            mermaid_arrow: ColorStyle::fg("cyan"),
            log_error: ColorStyle::fg_bold("red"),
            log_warn: ColorStyle::fg("yellow"),
            log_info: ColorStyle::fg("green"),
            log_debug: ColorStyle::fg("darkgray"),
            log_timestamp: ColorStyle::fg("blue"),
            normal: ColorStyle::new(),
        },
        ui: dark_ui(),
    }
}

fn dark_ui() -> UiColors {
    UiColors {
        border_focused: "cyan".into(),
        border_unfocused: "darkgray".into(),

        line_number: "darkgray".into(),
        line_number_active: ColorStyle::fg_bold("yellow"),
        bracket_match: ColorStyle::fg_bg_bold("yellow", "darkgray"),
        fold_indicator: ColorStyle::fg_italic("darkgray"),
        tilde_empty: "darkgray".into(),
        wrap_gutter: "darkgray".into(),

        diagnostic_error: ColorStyle::fg_bold("red"),
        diagnostic_warning: ColorStyle::fg_bold("yellow"),
        diagnostic_info: ColorStyle::fg("cyan"),
        diagnostic_hint: ColorStyle::fg("darkgray"),

        welcome_title: ColorStyle::fg_bold("cyan"),
        welcome_key: ColorStyle::fg_bold("yellow"),
        welcome_desc: ColorStyle::fg("white"),
        welcome_section: ColorStyle::fg_bold("magenta"),

        status_bar_bg: "darkgray".into(),
        status_label: ColorStyle::fg_bg_bold("black", "cyan"),
        status_file: ColorStyle::fg_bg("white", "darkgray"),
        status_position: ColorStyle::fg_bg("gray", "darkgray"),
        status_message: ColorStyle::fg("yellow"),
        status_hover: ColorStyle::fg("cyan"),

        tab_active: ColorStyle::fg_bg_bold("white", "darkgray"),
        tab_inactive: ColorStyle::fg("gray"),
        tab_bar_bg: "black".into(),

        explorer_title: ColorStyle::fg_bold("white"),
        explorer_dir: ColorStyle::fg_bold("blue"),
        explorer_file: ColorStyle::fg("white"),
        explorer_selected: ColorStyle::fg_bg_bold("black", "cyan"),

        search_label: ColorStyle::fg_bg_bold("black", "yellow"),
        search_query: ColorStyle::fg("white"),
        search_match_count: ColorStyle::fg("darkgray"),

        fuzzy_border: "cyan".into(),
        fuzzy_title: ColorStyle::fg_bold("white"),
        fuzzy_prompt: ColorStyle::fg("yellow"),
        fuzzy_match_count: ColorStyle::fg("darkgray"),
        fuzzy_selected: ColorStyle::fg_bg_bold("black", "cyan"),
        fuzzy_item: ColorStyle::fg("white"),

        goto_border: "cyan".into(),
        goto_title: ColorStyle::fg_bold("white"),
        goto_prompt: ColorStyle::fg("yellow"),

        completion_border: "darkgray".into(),
        completion_bg: "black".into(),
        completion_selected: ColorStyle::fg_bg_bold("black", "cyan"),
        completion_item: ColorStyle::fg("white"),
    }
}

pub fn light_theme() -> Theme {
    Theme {
        name: "light".to_string(),
        syntax: SyntaxColors {
            keyword: ColorStyle::fg_bold("#7928a1"),
            type_name: ColorStyle::fg("#0550ae"),
            string: ColorStyle::fg("#0a3069"),
            comment: ColorStyle::fg_italic("#6e7781"),
            number: ColorStyle::fg("#0550ae"),
            function: ColorStyle::fg("#6639ba"),
            operator: ColorStyle::fg("#cf222e"),
            punctuation: ColorStyle::fg("#24292f"),
            variable: ColorStyle::fg("#24292f"),
            heading: ColorStyle::fg_bold("#0550ae"),
            link: ColorStyle::fg_underline("#0969da"),
            emphasis: ColorStyle::italic_only(),
            mermaid_keyword: ColorStyle::fg_bold("#7928a1"),
            mermaid_arrow: ColorStyle::fg("#0550ae"),
            log_error: ColorStyle::fg_bold("#cf222e"),
            log_warn: ColorStyle::fg("#9a6700"),
            log_info: ColorStyle::fg("#116329"),
            log_debug: ColorStyle::fg("#6e7781"),
            log_timestamp: ColorStyle::fg("#0550ae"),
            normal: ColorStyle::new(),
        },
        ui: UiColors {
            border_focused: "#0550ae".into(),
            border_unfocused: "#d0d7de".into(),

            line_number: "#6e7781".into(),
            line_number_active: ColorStyle::fg_bold("#24292f"),
            bracket_match: ColorStyle::fg_bg_bold("#24292f", "#ddf4ff"),
            fold_indicator: ColorStyle::fg_italic("#6e7781"),
            tilde_empty: "#d0d7de".into(),
            wrap_gutter: "#d0d7de".into(),

            diagnostic_error: ColorStyle::fg_bold("#cf222e"),
            diagnostic_warning: ColorStyle::fg_bold("#9a6700"),
            diagnostic_info: ColorStyle::fg("#0550ae"),
            diagnostic_hint: ColorStyle::fg("#6e7781"),

            welcome_title: ColorStyle::fg_bold("#0550ae"),
            welcome_key: ColorStyle::fg_bold("#7928a1"),
            welcome_desc: ColorStyle::fg("#24292f"),
            welcome_section: ColorStyle::fg_bold("#cf222e"),

            status_bar_bg: "#d0d7de".into(),
            status_label: ColorStyle::fg_bg_bold("#ffffff", "#0550ae"),
            status_file: ColorStyle::fg_bg("#24292f", "#d0d7de"),
            status_position: ColorStyle::fg_bg("#57606a", "#d0d7de"),
            status_message: ColorStyle::fg("#9a6700"),
            status_hover: ColorStyle::fg("#0550ae"),

            tab_active: ColorStyle::fg_bg_bold("#24292f", "#ffffff"),
            tab_inactive: ColorStyle::fg("#57606a"),
            tab_bar_bg: "#f6f8fa".into(),

            explorer_title: ColorStyle::fg_bold("#24292f"),
            explorer_dir: ColorStyle::fg_bold("#0550ae"),
            explorer_file: ColorStyle::fg("#24292f"),
            explorer_selected: ColorStyle::fg_bg_bold("#ffffff", "#0550ae"),

            search_label: ColorStyle::fg_bg_bold("#ffffff", "#9a6700"),
            search_query: ColorStyle::fg("#24292f"),
            search_match_count: ColorStyle::fg("#6e7781"),

            fuzzy_border: "#0550ae".into(),
            fuzzy_title: ColorStyle::fg_bold("#24292f"),
            fuzzy_prompt: ColorStyle::fg("#7928a1"),
            fuzzy_match_count: ColorStyle::fg("#6e7781"),
            fuzzy_selected: ColorStyle::fg_bg_bold("#ffffff", "#0550ae"),
            fuzzy_item: ColorStyle::fg("#24292f"),

            goto_border: "#0550ae".into(),
            goto_title: ColorStyle::fg_bold("#24292f"),
            goto_prompt: ColorStyle::fg("#7928a1"),

            completion_border: "#d0d7de".into(),
            completion_bg: "#ffffff".into(),
            completion_selected: ColorStyle::fg_bg_bold("#ffffff", "#0550ae"),
            completion_item: ColorStyle::fg("#24292f"),
        },
    }
}

pub fn monokai_theme() -> Theme {
    Theme {
        name: "monokai".to_string(),
        syntax: SyntaxColors {
            keyword: ColorStyle::fg_bold("#f92672"),
            type_name: ColorStyle::fg("#66d9ef"),
            string: ColorStyle::fg("#e6db74"),
            comment: ColorStyle::fg_italic("#75715e"),
            number: ColorStyle::fg("#ae81ff"),
            function: ColorStyle::fg("#a6e22e"),
            operator: ColorStyle::fg("#f92672"),
            punctuation: ColorStyle::fg("#f8f8f2"),
            variable: ColorStyle::fg("#f8f8f2"),
            heading: ColorStyle::fg_bold("#66d9ef"),
            link: ColorStyle::fg_underline("#66d9ef"),
            emphasis: ColorStyle::italic_only(),
            mermaid_keyword: ColorStyle::fg_bold("#f92672"),
            mermaid_arrow: ColorStyle::fg("#66d9ef"),
            log_error: ColorStyle::fg_bold("#f92672"),
            log_warn: ColorStyle::fg("#e6db74"),
            log_info: ColorStyle::fg("#a6e22e"),
            log_debug: ColorStyle::fg("#75715e"),
            log_timestamp: ColorStyle::fg("#66d9ef"),
            normal: ColorStyle::new(),
        },
        ui: UiColors {
            border_focused: "#66d9ef".into(),
            border_unfocused: "#75715e".into(),

            line_number: "#75715e".into(),
            line_number_active: ColorStyle::fg_bold("#f8f8f2"),
            bracket_match: ColorStyle::fg_bg_bold("#e6db74", "#49483e"),
            fold_indicator: ColorStyle::fg_italic("#75715e"),
            tilde_empty: "#75715e".into(),
            wrap_gutter: "#75715e".into(),

            diagnostic_error: ColorStyle::fg_bold("#f92672"),
            diagnostic_warning: ColorStyle::fg_bold("#e6db74"),
            diagnostic_info: ColorStyle::fg("#66d9ef"),
            diagnostic_hint: ColorStyle::fg("#75715e"),

            welcome_title: ColorStyle::fg_bold("#66d9ef"),
            welcome_key: ColorStyle::fg_bold("#e6db74"),
            welcome_desc: ColorStyle::fg("#f8f8f2"),
            welcome_section: ColorStyle::fg_bold("#f92672"),

            status_bar_bg: "#49483e".into(),
            status_label: ColorStyle::fg_bg_bold("#272822", "#a6e22e"),
            status_file: ColorStyle::fg_bg("#f8f8f2", "#49483e"),
            status_position: ColorStyle::fg_bg("#75715e", "#49483e"),
            status_message: ColorStyle::fg("#e6db74"),
            status_hover: ColorStyle::fg("#66d9ef"),

            tab_active: ColorStyle::fg_bg_bold("#f8f8f2", "#49483e"),
            tab_inactive: ColorStyle::fg("#75715e"),
            tab_bar_bg: "#272822".into(),

            explorer_title: ColorStyle::fg_bold("#f8f8f2"),
            explorer_dir: ColorStyle::fg_bold("#66d9ef"),
            explorer_file: ColorStyle::fg("#f8f8f2"),
            explorer_selected: ColorStyle::fg_bg_bold("#272822", "#a6e22e"),

            search_label: ColorStyle::fg_bg_bold("#272822", "#e6db74"),
            search_query: ColorStyle::fg("#f8f8f2"),
            search_match_count: ColorStyle::fg("#75715e"),

            fuzzy_border: "#66d9ef".into(),
            fuzzy_title: ColorStyle::fg_bold("#f8f8f2"),
            fuzzy_prompt: ColorStyle::fg("#e6db74"),
            fuzzy_match_count: ColorStyle::fg("#75715e"),
            fuzzy_selected: ColorStyle::fg_bg_bold("#272822", "#a6e22e"),
            fuzzy_item: ColorStyle::fg("#f8f8f2"),

            goto_border: "#66d9ef".into(),
            goto_title: ColorStyle::fg_bold("#f8f8f2"),
            goto_prompt: ColorStyle::fg("#e6db74"),

            completion_border: "#75715e".into(),
            completion_bg: "#272822".into(),
            completion_selected: ColorStyle::fg_bg_bold("#272822", "#a6e22e"),
            completion_item: ColorStyle::fg("#f8f8f2"),
        },
    }
}

pub fn solarized_dark_theme() -> Theme {
    Theme {
        name: "solarized-dark".to_string(),
        syntax: SyntaxColors {
            keyword: ColorStyle::fg_bold("#b58900"),
            type_name: ColorStyle::fg("#268bd2"),
            string: ColorStyle::fg("#2aa198"),
            comment: ColorStyle::fg_italic("#586e75"),
            number: ColorStyle::fg("#d33682"),
            function: ColorStyle::fg("#268bd2"),
            operator: ColorStyle::fg("#cb4b16"),
            punctuation: ColorStyle::fg("#839496"),
            variable: ColorStyle::fg("#839496"),
            heading: ColorStyle::fg_bold("#268bd2"),
            link: ColorStyle::fg_underline("#268bd2"),
            emphasis: ColorStyle::italic_only(),
            mermaid_keyword: ColorStyle::fg_bold("#b58900"),
            mermaid_arrow: ColorStyle::fg("#268bd2"),
            log_error: ColorStyle::fg_bold("#dc322f"),
            log_warn: ColorStyle::fg("#b58900"),
            log_info: ColorStyle::fg("#859900"),
            log_debug: ColorStyle::fg("#586e75"),
            log_timestamp: ColorStyle::fg("#268bd2"),
            normal: ColorStyle::new(),
        },
        ui: UiColors {
            border_focused: "#268bd2".into(),
            border_unfocused: "#586e75".into(),

            line_number: "#586e75".into(),
            line_number_active: ColorStyle::fg_bold("#839496"),
            bracket_match: ColorStyle::fg_bg_bold("#b58900", "#073642"),
            fold_indicator: ColorStyle::fg_italic("#586e75"),
            tilde_empty: "#586e75".into(),
            wrap_gutter: "#586e75".into(),

            diagnostic_error: ColorStyle::fg_bold("#dc322f"),
            diagnostic_warning: ColorStyle::fg_bold("#b58900"),
            diagnostic_info: ColorStyle::fg("#268bd2"),
            diagnostic_hint: ColorStyle::fg("#586e75"),

            welcome_title: ColorStyle::fg_bold("#268bd2"),
            welcome_key: ColorStyle::fg_bold("#b58900"),
            welcome_desc: ColorStyle::fg("#839496"),
            welcome_section: ColorStyle::fg_bold("#cb4b16"),

            status_bar_bg: "#073642".into(),
            status_label: ColorStyle::fg_bg_bold("#fdf6e3", "#268bd2"),
            status_file: ColorStyle::fg_bg("#839496", "#073642"),
            status_position: ColorStyle::fg_bg("#586e75", "#073642"),
            status_message: ColorStyle::fg("#b58900"),
            status_hover: ColorStyle::fg("#268bd2"),

            tab_active: ColorStyle::fg_bg_bold("#839496", "#073642"),
            tab_inactive: ColorStyle::fg("#586e75"),
            tab_bar_bg: "#002b36".into(),

            explorer_title: ColorStyle::fg_bold("#839496"),
            explorer_dir: ColorStyle::fg_bold("#268bd2"),
            explorer_file: ColorStyle::fg("#839496"),
            explorer_selected: ColorStyle::fg_bg_bold("#fdf6e3", "#268bd2"),

            search_label: ColorStyle::fg_bg_bold("#fdf6e3", "#b58900"),
            search_query: ColorStyle::fg("#839496"),
            search_match_count: ColorStyle::fg("#586e75"),

            fuzzy_border: "#268bd2".into(),
            fuzzy_title: ColorStyle::fg_bold("#839496"),
            fuzzy_prompt: ColorStyle::fg("#b58900"),
            fuzzy_match_count: ColorStyle::fg("#586e75"),
            fuzzy_selected: ColorStyle::fg_bg_bold("#fdf6e3", "#268bd2"),
            fuzzy_item: ColorStyle::fg("#839496"),

            goto_border: "#268bd2".into(),
            goto_title: ColorStyle::fg_bold("#839496"),
            goto_prompt: ColorStyle::fg("#b58900"),

            completion_border: "#586e75".into(),
            completion_bg: "#002b36".into(),
            completion_selected: ColorStyle::fg_bg_bold("#fdf6e3", "#268bd2"),
            completion_item: ColorStyle::fg("#839496"),
        },
    }
}

// ---------------------------------------------------------------------------
// Theme override (merge partial user config on top of a base)
// ---------------------------------------------------------------------------

/// Intermediate deserialization type for the "theme" field in settings.json.
/// Accepts either a string (built-in name) or an object (with base + overrides).
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ThemeConfig {
    Name(String),
    Custom(ThemeOverride),
}

#[derive(Debug, Clone, Deserialize)]
pub struct ThemeOverride {
    #[serde(default = "default_base")]
    pub base: String,
    #[serde(default)]
    pub syntax: Option<SyntaxOverride>,
    #[serde(default)]
    pub ui: Option<UiOverride>,
}

fn default_base() -> String {
    "dark".to_string()
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SyntaxOverride {
    pub keyword: Option<ColorStyle>,
    pub type_name: Option<ColorStyle>,
    pub string: Option<ColorStyle>,
    pub comment: Option<ColorStyle>,
    pub number: Option<ColorStyle>,
    pub function: Option<ColorStyle>,
    pub operator: Option<ColorStyle>,
    pub punctuation: Option<ColorStyle>,
    pub variable: Option<ColorStyle>,
    pub heading: Option<ColorStyle>,
    pub link: Option<ColorStyle>,
    pub emphasis: Option<ColorStyle>,
    pub mermaid_keyword: Option<ColorStyle>,
    pub mermaid_arrow: Option<ColorStyle>,
    pub log_error: Option<ColorStyle>,
    pub log_warn: Option<ColorStyle>,
    pub log_info: Option<ColorStyle>,
    pub log_debug: Option<ColorStyle>,
    pub log_timestamp: Option<ColorStyle>,
    pub normal: Option<ColorStyle>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct UiOverride {
    pub border_focused: Option<String>,
    pub border_unfocused: Option<String>,
    pub line_number: Option<String>,
    pub line_number_active: Option<ColorStyle>,
    pub bracket_match: Option<ColorStyle>,
    pub fold_indicator: Option<ColorStyle>,
    pub tilde_empty: Option<String>,
    pub wrap_gutter: Option<String>,
    pub diagnostic_error: Option<ColorStyle>,
    pub diagnostic_warning: Option<ColorStyle>,
    pub diagnostic_info: Option<ColorStyle>,
    pub diagnostic_hint: Option<ColorStyle>,
    pub welcome_title: Option<ColorStyle>,
    pub welcome_key: Option<ColorStyle>,
    pub welcome_desc: Option<ColorStyle>,
    pub welcome_section: Option<ColorStyle>,
    pub status_bar_bg: Option<String>,
    pub status_label: Option<ColorStyle>,
    pub status_file: Option<ColorStyle>,
    pub status_position: Option<ColorStyle>,
    pub status_message: Option<ColorStyle>,
    pub status_hover: Option<ColorStyle>,
    pub tab_active: Option<ColorStyle>,
    pub tab_inactive: Option<ColorStyle>,
    pub tab_bar_bg: Option<String>,
    pub explorer_title: Option<ColorStyle>,
    pub explorer_dir: Option<ColorStyle>,
    pub explorer_file: Option<ColorStyle>,
    pub explorer_selected: Option<ColorStyle>,
    pub search_label: Option<ColorStyle>,
    pub search_query: Option<ColorStyle>,
    pub search_match_count: Option<ColorStyle>,
    pub fuzzy_border: Option<String>,
    pub fuzzy_title: Option<ColorStyle>,
    pub fuzzy_prompt: Option<ColorStyle>,
    pub fuzzy_match_count: Option<ColorStyle>,
    pub fuzzy_selected: Option<ColorStyle>,
    pub fuzzy_item: Option<ColorStyle>,
    pub goto_border: Option<String>,
    pub goto_title: Option<ColorStyle>,
    pub goto_prompt: Option<ColorStyle>,
    pub completion_border: Option<String>,
    pub completion_bg: Option<String>,
    pub completion_selected: Option<ColorStyle>,
    pub completion_item: Option<ColorStyle>,
}

impl Theme {
    /// Resolve a ThemeConfig to a concrete Theme.
    pub fn resolve(config: &ThemeConfig) -> Self {
        match config {
            ThemeConfig::Name(name) => Theme::builtin(name).unwrap_or_default(),
            ThemeConfig::Custom(ovr) => {
                let mut theme = Theme::builtin(&ovr.base).unwrap_or_default();
                theme.name = format!("{} (custom)", ovr.base);
                if let Some(ref s) = ovr.syntax {
                    macro_rules! apply_syn {
                        ($field:ident) => {
                            if let Some(ref v) = s.$field {
                                theme.syntax.$field = v.clone();
                            }
                        };
                    }
                    apply_syn!(keyword);
                    apply_syn!(type_name);
                    apply_syn!(string);
                    apply_syn!(comment);
                    apply_syn!(number);
                    apply_syn!(function);
                    apply_syn!(operator);
                    apply_syn!(punctuation);
                    apply_syn!(variable);
                    apply_syn!(heading);
                    apply_syn!(link);
                    apply_syn!(emphasis);
                    apply_syn!(mermaid_keyword);
                    apply_syn!(mermaid_arrow);
                    apply_syn!(log_error);
                    apply_syn!(log_warn);
                    apply_syn!(log_info);
                    apply_syn!(log_debug);
                    apply_syn!(log_timestamp);
                    apply_syn!(normal);
                }
                if let Some(ref u) = ovr.ui {
                    macro_rules! apply_ui_str {
                        ($field:ident) => {
                            if let Some(ref v) = u.$field {
                                theme.ui.$field = v.clone();
                            }
                        };
                    }
                    macro_rules! apply_ui_cs {
                        ($field:ident) => {
                            if let Some(ref v) = u.$field {
                                theme.ui.$field = v.clone();
                            }
                        };
                    }
                    apply_ui_str!(border_focused);
                    apply_ui_str!(border_unfocused);
                    apply_ui_str!(line_number);
                    apply_ui_cs!(line_number_active);
                    apply_ui_cs!(bracket_match);
                    apply_ui_cs!(fold_indicator);
                    apply_ui_str!(tilde_empty);
                    apply_ui_str!(wrap_gutter);
                    apply_ui_cs!(diagnostic_error);
                    apply_ui_cs!(diagnostic_warning);
                    apply_ui_cs!(diagnostic_info);
                    apply_ui_cs!(diagnostic_hint);
                    apply_ui_cs!(welcome_title);
                    apply_ui_cs!(welcome_key);
                    apply_ui_cs!(welcome_desc);
                    apply_ui_cs!(welcome_section);
                    apply_ui_str!(status_bar_bg);
                    apply_ui_cs!(status_label);
                    apply_ui_cs!(status_file);
                    apply_ui_cs!(status_position);
                    apply_ui_cs!(status_message);
                    apply_ui_cs!(status_hover);
                    apply_ui_cs!(tab_active);
                    apply_ui_cs!(tab_inactive);
                    apply_ui_str!(tab_bar_bg);
                    apply_ui_cs!(explorer_title);
                    apply_ui_cs!(explorer_dir);
                    apply_ui_cs!(explorer_file);
                    apply_ui_cs!(explorer_selected);
                    apply_ui_cs!(search_label);
                    apply_ui_cs!(search_query);
                    apply_ui_cs!(search_match_count);
                    apply_ui_str!(fuzzy_border);
                    apply_ui_cs!(fuzzy_title);
                    apply_ui_cs!(fuzzy_prompt);
                    apply_ui_cs!(fuzzy_match_count);
                    apply_ui_cs!(fuzzy_selected);
                    apply_ui_cs!(fuzzy_item);
                    apply_ui_str!(goto_border);
                    apply_ui_cs!(goto_title);
                    apply_ui_cs!(goto_prompt);
                    apply_ui_str!(completion_border);
                    apply_ui_str!(completion_bg);
                    apply_ui_cs!(completion_selected);
                    apply_ui_cs!(completion_item);
                }
                theme
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme_is_dark() {
        let theme = Theme::default();
        assert_eq!(theme.name, "dark");
    }

    #[test]
    fn test_builtin_lookup() {
        assert!(Theme::builtin("dark").is_some());
        assert!(Theme::builtin("light").is_some());
        assert!(Theme::builtin("monokai").is_some());
        assert!(Theme::builtin("solarized-dark").is_some());
        assert!(Theme::builtin("nonexistent").is_none());
    }

    #[test]
    fn test_resolve_name() {
        let config = ThemeConfig::Name("monokai".to_string());
        let theme = Theme::resolve(&config);
        assert_eq!(theme.name, "monokai");
    }

    #[test]
    fn test_resolve_unknown_name_falls_back() {
        let config = ThemeConfig::Name("unknown".to_string());
        let theme = Theme::resolve(&config);
        assert_eq!(theme.name, "dark");
    }

    #[test]
    fn test_resolve_custom_override() {
        let json = r##"{
            "base": "dark",
            "syntax": {
                "keyword": { "fg": "#ff0000", "bold": true }
            },
            "ui": {
                "border_focused": "#00ff00"
            }
        }"##;
        let config: ThemeConfig = serde_json::from_str(json).unwrap();
        let theme = Theme::resolve(&config);
        assert_eq!(theme.syntax.keyword.fg.as_deref(), Some("#ff0000"));
        assert!(theme.syntax.keyword.bold);
        assert_eq!(theme.ui.border_focused, "#00ff00");
        // Non-overridden fields keep base values
        assert_eq!(theme.syntax.string.fg.as_deref(), Some("green"));
    }

    #[test]
    fn test_deserialize_theme_name_string() {
        let json = r#""monokai""#;
        let config: ThemeConfig = serde_json::from_str(json).unwrap();
        match config {
            ThemeConfig::Name(n) => assert_eq!(n, "monokai"),
            _ => panic!("expected Name variant"),
        }
    }

    #[test]
    fn test_builtin_names() {
        let names = Theme::builtin_names();
        assert_eq!(names.len(), 4);
        assert!(names.contains(&"dark"));
    }

    #[test]
    fn test_color_style_constructors() {
        let s = ColorStyle::fg("red");
        assert_eq!(s.fg.as_deref(), Some("red"));
        assert!(!s.bold);

        let s = ColorStyle::fg_bold("blue");
        assert!(s.bold);

        let s = ColorStyle::fg_italic("gray");
        assert!(s.italic);

        let s = ColorStyle::fg_bg_bold("white", "black");
        assert_eq!(s.fg.as_deref(), Some("white"));
        assert_eq!(s.bg.as_deref(), Some("black"));
        assert!(s.bold);
    }
}
