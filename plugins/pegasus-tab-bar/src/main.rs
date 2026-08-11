use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use zellij_tile::prelude::*;

#[derive(Default)]
struct PegasusTabBar {
    permission: PermissionState,
    mode_info: Option<ModeInfo>,
    tabs: Vec<TabInfo>,
    tab_ranges: Vec<TabRange>,
}

#[derive(Debug, PartialEq, Eq)]
struct TabRange {
    start: usize,
    end: usize,
    position: usize,
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
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
        ]);
        set_timeout(0.0);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::Timer(_) => true,
            Event::PermissionRequestResult(PermissionStatus::Granted) => {
                self.permission = PermissionState::Granted;
                subscribe(&[
                    EventType::ModeUpdate,
                    EventType::TabUpdate,
                    EventType::Mouse,
                ]);
                true
            }
            Event::PermissionRequestResult(PermissionStatus::Denied) => {
                self.permission = PermissionState::Denied;
                true
            }
            Event::ModeUpdate(mode_info) if self.permission == PermissionState::Granted => {
                self.mode_info = Some(mode_info);
                self.tab_ranges.clear();
                true
            }
            Event::TabUpdate(tabs) if self.permission == PermissionState::Granted => {
                self.tabs = tabs;
                self.tab_ranges.clear();
                true
            }
            Event::Mouse(Mouse::LeftClick(0, column))
                if self.permission == PermissionState::Granted =>
            {
                if let Some(position) = tab_position_at(&self.tab_ranges, column) {
                    if let Ok(tab_index) = u32::try_from(position.saturating_add(1)) {
                        switch_tab_to(tab_index);
                    }
                }
                false
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
    fn render_tabs(&mut self, cols: usize) -> String {
        self.tab_ranges.clear();
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
            let start = cols - remaining;
            let width = push_segment(
                &mut line,
                &mut remaining,
                &tab.name,
                tab_budget,
                if tab.active { selected } else { unselected },
            );
            self.tab_ranges.push(TabRange {
                start,
                end: start + width,
                position: tab.position,
            });
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
) -> usize {
    let (content, width) = segment_content(label, budget, remaining);

    line.push_str(&paint(content, colors));
    width
}

fn tab_position_at(tab_ranges: &[TabRange], column: usize) -> Option<usize> {
    tab_ranges
        .iter()
        .find(|range| range.start <= column && column < range.end)
        .map(|range| range.position)
}

#[cfg(test)]
fn tab_ranges_for(cols: usize, session_name: &str, tabs: &[(usize, &str)]) -> Vec<TabRange> {
    let mut remaining = cols;
    let session_budget = cols.saturating_div(3).max(1);
    segment_content(session_name, session_budget, &mut remaining);

    let mut tab_ranges = Vec::new();
    for (position, name) in tabs {
        if remaining < 3 {
            break;
        }
        let start = cols - remaining;
        let width = segment_content(name, remaining, &mut remaining).1;
        tab_ranges.push(TabRange {
            start,
            end: start + width,
            position: *position,
        });
    }
    tab_ranges
}

fn segment_content(label: &str, budget: usize, remaining: &mut usize) -> (String, usize) {
    if *remaining == 0 {
        return (String::new(), 0);
    }

    let content_width = budget.min(*remaining).saturating_sub(2);
    let label = truncate_to_width(label, content_width);
    let content = format!(" {label} ");
    let width = display_width(&content).min(*remaining);
    let content = truncate_to_width(&content, width);
    *remaining -= width;
    (content, width)
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

    #[test]
    fn clicks_map_to_the_actual_tab_positions_not_their_render_order() {
        let ranges = tab_ranges_for(30, "work", &[(4, "inactive"), (1, "active")]);

        assert_eq!(
            tab_position_at(&ranges, 2),
            None,
            "session is not clickable"
        );
        assert_eq!(tab_position_at(&ranges, ranges[0].start), Some(4));
        assert_eq!(tab_position_at(&ranges, ranges[0].end - 1), Some(4));
        assert_eq!(tab_position_at(&ranges, ranges[1].start), Some(1));
        assert_eq!(tab_position_at(&ranges, ranges[1].end - 1), Some(1));
        assert_eq!(
            tab_position_at(&ranges, ranges[1].end),
            None,
            "padding is inert"
        );
    }

    #[test]
    fn tab_ranges_stop_before_the_trailing_padding() {
        let ranges = tab_ranges_for(40, "session", &[(0, "one"), (1, "two")]);

        assert_eq!(tab_position_at(&ranges, 0), None);
        assert_eq!(tab_position_at(&ranges, 8), None);
        assert_eq!(tab_position_at(&ranges, 9), Some(0));
        assert_eq!(tab_position_at(&ranges, 14), Some(1));
        assert_eq!(tab_position_at(&ranges, 40 - 1), None);
    }

    #[test]
    fn truncated_tab_label_remains_clickable_across_its_visible_range() {
        let ranges = tab_ranges_for(12, "session-name", &[(7, "状態を確認しています")]);

        assert_eq!(
            ranges,
            vec![TabRange {
                start: 4,
                end: 11,
                position: 7
            }]
        );
        assert_eq!(tab_position_at(&ranges, 4), Some(7));
        assert_eq!(tab_position_at(&ranges, 10), Some(7));
        assert_eq!(tab_position_at(&ranges, 11), None);
        assert_eq!(tab_position_at(&ranges, 12), None);
    }
}
