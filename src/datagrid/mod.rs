mod imp;

use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use glib::subclass::types::ObjectSubclassIsExt;
use gtk4::*;

use gtk4::glib::*;
use gtk4::traits::WidgetExt;

use lazy_static::lazy_static;

use crate::data_helper::{stox_get_complete_info, stox_get_quotes};

use gettextrs::gettext;

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

        obj
    }

    pub fn update(&self, symbol: String, force_update: bool, is_saved: bool) -> bool {
        let lock = UPDATE_LOCK.try_lock();
        if lock.is_err() {
            return true;
        }
        let lock = RefCell::new(Some(lock.unwrap()));

        self.imp().refresh_btn.borrow().show();

        let symbol_label = self.imp().symbol_label.borrow();

        if !force_update && symbol_label.label() == symbol {
            return false;
        }

        symbol_label.set_label(&symbol);
        symbol_label.set_css_classes(&[]);

        if is_saved {
            self.imp().save_btn.borrow().hide();
            self.imp().unsave_btn.borrow().show();
        } else {
            self.imp().save_btn.borrow().show();
            self.imp().unsave_btn.borrow().hide();
        }

        let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);

        let symbol = RefCell::new(symbol);

        std::thread::spawn(move || match stox_get_complete_info(&symbol.borrow()) {
            Ok((main_info, extended_info)) => {
                let quotes = stox_get_quotes(symbol.borrow().to_string(), "1d");
                if quotes.is_err() {
                    sender.send(None).unwrap();
                    return;
                }

                sender
                    .send(Some((main_info, extended_info, quotes.unwrap())))
                    .unwrap()
            }
            Err(_) => sender.send(None).unwrap(),
        });

        let name_label = self.imp().name_label.borrow().clone();
        let latest_quote_label = self.imp().latest_quote_label.borrow().clone();
        let market_change_label = self.imp().market_change_label.borrow().clone();
        let info_label = self.imp().info_label.borrow().clone();
        let notebook = self.imp().notebook.borrow().clone();

        name_label.set_label("--");
        latest_quote_label.set_label("--");
        market_change_label.set_label("--");
        market_change_label.set_css_classes(&[]);
        info_label.set_label("--");

        let save_btn = self.imp().save_btn.borrow().clone();
        let unsave_btn = self.imp().unsave_btn.borrow().clone();
        let refresh_btn = self.imp().refresh_btn.borrow().clone();

        save_btn.set_sensitive(false);
        unsave_btn.set_sensitive(false);
        refresh_btn.set_sensitive(false);

        {
            let notebook = self.imp().notebook.borrow_mut();
            for i in 0..notebook.n_pages() {
                notebook.remove_page(Some(i));
            }
        }

        receiver.attach(
            None,
            clone!(@strong self as this => move |complete_info| {
                match complete_info {
                    Some((main_info, extended_info, quotes)) => {
                        name_label.set_label(&main_info.short_name);
                        latest_quote_label.set_label(&main_info.last_quote);
                        market_change_label.set_label(&format!(
                            "{} ({})",
                            &extended_info.market_change, &extended_info.market_change_percent
                        ));

                        if extended_info.market_change_neg() {
                            market_change_label.set_css_classes(&["market_change_neg"]);
                        } else {
                            market_change_label.set_css_classes(&["market_change_pos"]);
                        }

                        info_label.set_label(&format!(
                            "{} - {}",
                            extended_info.exchange_name, main_info.currency
                        ));

                        this.imp().construct_graph(main_info, extended_info, quotes);
                    }
                    None => {
                        name_label.set_label("???");
                        latest_quote_label.set_label("???");
                        market_change_label.set_label("???");
                        info_label.set_label("???");

                        notebook.append_page(
                            &Label::new(Some(&gettext("The graph could not be loaded."))),
                            Some(&Label::new(Some(&gettext("Error")))),
                        );
                    }
                }

                save_btn.set_sensitive(true);
                unsave_btn.set_sensitive(true);
                refresh_btn.set_sensitive(true);

                drop(lock.replace(None));

                Continue(false)
            }),
        );

        false
    }
}

impl Default for StoxDataGrid {
    fn default() -> Self {
        Self::new()
    }
}
