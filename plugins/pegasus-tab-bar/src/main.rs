use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use zellij_tile::prelude::*;

#[derive(Default)]
struct PegasusTabBar {
    permission: PermissionState,
    mode_info: Option<ModeInfo>,
    tabs: Vec<TabInfo>,
}

#[derive(Default, PartialEq, Eq)]
enum PermissionState {
    #[default]
    Requesting,
    Granted,
    Denied,
}

impl ZellijPlugin for PegasusTabBar {
    fn load(&mut self, _configuration: std::collections::BTreeMap<String, String>) {
        set_selectable(false);
        subscribe(&[EventType::PermissionRequestResult, EventType::Timer]);
        request_permission(&[PermissionType::ReadApplicationState]);
        set_timeout(0.0);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::Timer(_) => true,
            Event::PermissionRequestResult(PermissionStatus::Granted) => {
                self.permission = PermissionState::Granted;
                subscribe(&[EventType::ModeUpdate, EventType::TabUpdate]);
                true
            }
            Event::PermissionRequestResult(PermissionStatus::Denied) => {
                self.permission = PermissionState::Denied;
                true
            }
            Event::ModeUpdate(mode_info) if self.permission == PermissionState::Granted => {
                self.mode_info = Some(mode_info);
                true
            }
            Event::TabUpdate(tabs) if self.permission == PermissionState::Granted => {
                self.tabs = tabs;
                true
            }
            _ => false,
        }
    }

    fn render(&mut self, _rows: usize, cols: usize) {
        let line = match self.permission {
            PermissionState::Requesting => "Requesting read access…".to_owned(),
            PermissionState::Denied => "Read access denied".to_owned(),
            PermissionState::Granted => self.render_tabs(cols),
        };

        print!("{line}\u{1b}[0K");
    }
}

impl PegasusTabBar {
    fn render_tabs(&self, cols: usize) -> String {
        let Some(mode_info) = &self.mode_info else {
            return "Loading session…".to_owned();
        };

        let unselected = mode_info.style.colors.ribbon_unselected;
        let selected = mode_info.style.colors.ribbon_selected;
        let session_name = mode_info.session_name.as_deref().unwrap_or("session");
        let mut remaining = cols;
        let mut line = String::new();

        let session_budget = cols.saturating_div(3).max(1);
        push_segment(
            &mut line,
            &mut remaining,
            session_name,
            session_budget,
            unselected,
        );

        for tab in &self.tabs {
            if remaining < 3 {
                break;
            }
            let tab_budget = remaining;
            push_segment(
                &mut line,
                &mut remaining,
                &tab.name,
                tab_budget,
                if tab.active { selected } else { unselected },
            );
        }

        if remaining > 0 {
            line.push_str(&paint(" ".repeat(remaining), unselected));
        }

        line
    }
}

fn push_segment(
    line: &mut String,
    remaining: &mut usize,
    label: &str,
    budget: usize,
    colors: StyleDeclaration,
) {
    if *remaining == 0 {
        return;
    }

    let content_width = budget.min(*remaining).saturating_sub(2);
    let label = truncate_to_width(label, content_width);
    let content = format!(" {label} ");
    let width = display_width(&content).min(*remaining);
    let content = truncate_to_width(&content, width);

    line.push_str(&paint(content, colors));
    *remaining -= width;
}

fn paint(content: String, colors: StyleDeclaration) -> String {
    format!(
        "{}{}{}\x1b[0m",
        ansi_colour(colors.base, true),
        ansi_colour(colors.background, false),
        content
    )
}

fn ansi_colour(colour: PaletteColor, foreground: bool) -> String {
    match colour {
        PaletteColor::Rgb((red, green, blue)) => {
            let channel = if foreground { 38 } else { 48 };
            format!("\x1b[{channel};2;{red};{green};{blue}m")
        }
        PaletteColor::EightBit(value) => {
            let channel = if foreground { 38 } else { 48 };
            format!("\x1b[{channel};5;{value}m")
        }
    }
}

fn display_width(value: &str) -> usize {
    UnicodeWidthStr::width(value)
}

fn truncate_to_width(value: &str, max_width: usize) -> String {
    if display_width(value) <= max_width {
        return value.to_owned();
    }
    if max_width == 0 {
        return String::new();
    }
    if max_width == 1 {
        return "…".to_owned();
    }

    let mut truncated = String::new();
    let mut width = 0;
    for character in value.chars() {
        let character_width = UnicodeWidthChar::width(character).unwrap_or(0);
        if width + character_width > max_width - 1 {
            break;
        }
        truncated.push(character);
        width += character_width;
    }
    truncated.push('…');
    truncated
}

register_plugin!(PegasusTabBar);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_short_labels() {
        assert_eq!(
            truncate_to_width("OC | working (1)", 20),
            "OC | working (1)"
        );
    }

    #[test]
    fn truncates_wide_labels_without_exceeding_the_budget() {
        let truncated = truncate_to_width("状態を確認しています", 7);

        assert_eq!(truncated, "状態を…");
        assert!(display_width(&truncated) <= 7);
    }

    #[test]
    fn truncates_to_a_single_ellipsis_when_only_one_column_is_available() {
        assert_eq!(truncate_to_width("session", 1), "…");
    }
}
