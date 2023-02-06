mod imp;

use std::cell::RefCell;

use glib::subclass::types::ObjectSubclassIsExt;
use gtk4::*;

use gtk4::glib::*;
use gtk4::traits::WidgetExt;

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

    pub fn update(&self, symbol: String, force_update: bool, is_saved: bool) {
        let symbol_label = self.imp().symbol_label.borrow();

        if !force_update && symbol_label.label() == symbol {
            return;
        }

        symbol_label.set_label(&symbol);

        if is_saved {
            self.imp().save_btn.borrow().hide();
            self.imp().unsave_btn.borrow().show();
        } else {
            self.imp().save_btn.borrow().show();
            self.imp().unsave_btn.borrow().hide();
        }

        let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);

        let symbol = RefCell::new(symbol);

        std::thread::spawn(move || match stox_get_main_info(&*symbol.borrow()) {
            Ok(main_info) => sender.send(main_info).unwrap(),
            Err(_) => sender.send(("???".to_string(), "???".to_string())).unwrap(),
        });

        let name_label = self.imp().name_label.borrow().clone();
        let latest_quote = self.imp().latest_quote.borrow().clone();

        name_label.set_label("--");
        latest_quote.set_label("--");

        receiver.attach(None, move |(last_quote, short_name)| {
            latest_quote.set_label(&last_quote);
            name_label.set_label(&short_name);

            Continue(false)
        });
    }
}
