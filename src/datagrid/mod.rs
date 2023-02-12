mod imp;

use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use glib::subclass::types::ObjectSubclassIsExt;
use gtk4::*;

use gtk4::glib::*;
use gtk4::traits::WidgetExt;

use lazy_static::lazy_static;

use crate::data_helper::stox_get_complete_info;

glib::wrapper! {
    pub struct StoxDataGrid(ObjectSubclass<imp::StoxDataGrid>)
        @extends Box, Widget,
        @implements Actionable, Accessible, Buildable, ConstraintTarget;
}

lazy_static! {
    static ref UPDATE_LOCK: Arc<Mutex<()>> = Arc::new(Mutex::new(()));
}

impl StoxDataGrid {
    pub fn new() -> Self {
        let obj: StoxDataGrid = Object::builder().build();

        return obj;
    }

    pub fn update(&self, symbol: String, force_update: bool, is_saved: bool) -> bool {
        let lock = UPDATE_LOCK.try_lock();
        if lock.is_err() {
            return true;
        }
        let lock = lock.unwrap();

        let symbol_label = self.imp().symbol_label.borrow();

        if !force_update && symbol_label.label() == symbol {
            return false;
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

        std::thread::spawn(move || match stox_get_complete_info(&*symbol.borrow()) {
            Ok((main_info, extended_info)) => {
                sender.send(Some((main_info, extended_info))).unwrap()
            }
            Err(_) => sender.send(None).unwrap(),
        });

        let name_label = self.imp().name_label.borrow().clone();
        let latest_quote = self.imp().latest_quote.borrow().clone();
        let delta_label = self.imp().delta_label.borrow().clone();
        let info_label = self.imp().info_label.borrow().clone();

        name_label.set_label("--");
        latest_quote.set_label("--");
        delta_label.set_label("--");
        delta_label.set_css_classes(&[]);
        info_label.set_label("--");

        receiver.attach(None, move |complete_info| {
            match complete_info {
                Some((main_info, extended_info)) => {
                    name_label.set_label(&main_info.short_name);
                    latest_quote.set_label(&main_info.last_quote);
                    delta_label.set_label(&main_info.delta);

                    if main_info.delta.chars().nth(0).unwrap() == '-' {
                        delta_label.set_css_classes(&["delta_negative"]);
                    } else {
                        delta_label.set_css_classes(&["delta_positive"]);
                    }

                    info_label.set_label(&format!(
                        "{} - {}",
                        extended_info.exchange_name, main_info.currency
                    ));
                }
                None => {
                    name_label.set_label("???");
                    latest_quote.set_label("???");
                    delta_label.set_label("???");
                    info_label.set_label("???");
                }
            }

            drop(lock.clone());

            Continue(false)
        });

        self.imp().construct_graph();

        return false;
    }
}
