mod imp;

use glib::subclass::types::ObjectSubclassIsExt;
use gtk4::*;

use gtk4::glib::*;
use yahoo_finance_api::YahooConnector;

use crate::data_helper::stox_get_main_info;

glib::wrapper! {
    pub struct StoxDataGrid(ObjectSubclass<imp::StoxDataGrid>)
        @extends Box, Widget,
        @implements Actionable, Accessible, Buildable, ConstraintTarget;
}

impl StoxDataGrid {
    pub fn new() -> Self {
        let obj: StoxDataGrid = Object::builder().build();

        return obj;
    }

    pub fn update(&self, symbol: String) {
        if self.imp().symbol_label.borrow().label() == symbol {
            return;
        }

        let provider = YahooConnector::new();

        let (last_quote, short_name) = stox_get_main_info(&provider, symbol.as_str());

        self.imp().symbol_label.borrow().set_label(symbol.as_str());
        self.imp()
            .name_label
            .borrow()
            .set_label(short_name.as_str());
        self.imp()
            .latest_quote
            .borrow()
            .set_label(last_quote.as_str());
    }
}
