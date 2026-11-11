use crate::i18n;
use inquire::ui::{Attributes, Color, IndexPrefix, RenderConfig, StyleSheet, Styled};

pub fn setup_inquire_theme() {
    inquire::set_global_render_config(loveformat_render_config());
}

pub fn loveformat_render_config() -> RenderConfig<'static> {
    let mut config = RenderConfig::default();

    config.prompt_prefix = Styled::new("::").with_fg(Color::LightCyan);
    config.answered_prompt_prefix = Styled::new("[ok]").with_fg(Color::LightGreen);
    config.highlighted_option_prefix = Styled::new(">").with_fg(Color::LightMagenta);
    config.unselected_checkbox = Styled::new("[ ]").with_fg(Color::DarkGrey);
    config.selected_checkbox = Styled::new("[x]").with_fg(Color::LightGreen);
    config.scroll_up_prefix = Styled::new("^").with_fg(Color::DarkGrey);
    config.scroll_down_prefix = Styled::new("v").with_fg(Color::DarkGrey);
    config.option_index_prefix = IndexPrefix::None;

    config.prompt = StyleSheet::new().with_attr(Attributes::BOLD);

    config.answer = StyleSheet::new()
        .with_fg(Color::LightCyan)
        .with_attr(Attributes::BOLD);

    config.selected_option = Some(
        StyleSheet::new()
            .with_fg(Color::LightCyan)
            .with_attr(Attributes::BOLD),
    );

    config.help_message = StyleSheet::new().with_fg(Color::DarkGrey);
    config.error_message = config
        .error_message
        .with_prefix(Styled::new("[!]").with_fg(Color::LightRed));

    config
}

pub fn help_select() -> String {
    i18n::t("help_select")
}

pub fn help_multi() -> String {
    i18n::t("help_multi")
}

pub fn help_text() -> String {
    i18n::t("help_text")
}
