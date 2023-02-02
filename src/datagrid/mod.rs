mod imp;

use glib::subclass::types::ObjectSubclassIsExt;
use gtk4::*;

use gtk4::glib::*;

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

    fn update_info(&self, symbol: &str, last_quote: &str, short_name: &str) {
        self.imp().symbol_label.borrow().set_label(symbol);
        self.imp().name_label.borrow().set_label(short_name);
        self.imp().latest_quote.borrow().set_label(last_quote);
    }

    pub fn update(&self, symbol: String) {
        if self.imp().symbol_label.borrow().label() == symbol {
            return;
        }

        match stox_get_main_info(symbol.as_str()) {
            Ok((last_quote, short_name)) => {
                self.update_info(symbol.as_str(), last_quote.as_str(), short_name.as_str());
            }
            Err(_) => self.update_info(symbol.as_str(), "???", "???"),
        }
    }
}
