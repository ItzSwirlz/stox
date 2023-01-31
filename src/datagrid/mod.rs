mod imp;

use gtk4::traits::*;
use gtk4::*;

use crate::data_helper::{stox_get_main_info, stox_get_ranges};

glib::wrapper! {
    pub struct StoxDataGrid(ObjectSubclass<imp::StoxDataGrid>)
        @extends Grid, Widget,
        @implements Actionable, Accessible, Buildable, ConstraintTarget;
}

impl StoxDataGrid {
    // What initially pops up
    pub fn new_initial() -> Grid {
        let grid = Grid::builder()
            .hexpand(true)
            .width_request(850)
            .halign(Align::Center)
            .margin_start(10)
            .margin_end(10)
            .margin_top(10)
            .margin_bottom(10)
            .vexpand(true)
            .build();
            
        grid.show();
        return grid
    }

    pub fn update_symbol(grid: Grid, symbol: &str) -> Grid {
        let symbol_label = Label::builder()
            .valign(Align::Baseline)
            .margin_end(10)
            .label(&symbol)
            .build();

        symbol_label.show();

        let info = stox_get_main_info(symbol.to_string());
        let name = Label::new(Some(info.0.as_str()));
        name.show();

        let latest_quote = Label::new(Some(info.1.as_str()));
        latest_quote.set_halign(Align::End);
        latest_quote.show();

        let notebook = Notebook::builder()
            .focusable(true)
            .hexpand(true)
            .vexpand(true)
            .margin_top(10)
            .build();

        grid.attach(&symbol_label, 0, 0, 100, 100);
        grid.attach_next_to(&name, Some(&symbol_label), PositionType::Bottom, 100, 100);
        grid.attach_next_to(
            &latest_quote,
            Some(&symbol_label),
            PositionType::Right,
            100,
            100,
        );
        grid.attach_next_to(&notebook, Some(&name), PositionType::Bottom, 300, 100);

        grid.show();

        return grid;
    }
}
