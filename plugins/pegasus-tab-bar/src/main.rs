use zellij_tile::prelude::*;

#[derive(Default)]
struct PegasusTabBar;

impl ZellijPlugin for PegasusTabBar {
    fn load(&mut self, _configuration: std::collections::BTreeMap<String, String>) {
        set_selectable(false);
        subscribe(&[EventType::Timer]);
        set_timeout(0.0);
    }

    fn update(&mut self, event: Event) -> bool {
        matches!(event, Event::Timer(_))
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        println!("Pegasus tab bar");
    }
}

register_plugin!(PegasusTabBar);
