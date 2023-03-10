mod imp;

use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use glib::subclass::types::ObjectSubclassIsExt;

use gtk4::glib::*;
use gtk4::traits::WidgetExt;
use gtk4::*;

use lazy_static::lazy_static;

use crate::data_helper::{stox_get_datagrid_info, stox_get_quotes};

use gettextrs::gettext;

use self::imp::{GRID_WIDTH, SYMBOL_LABEL_MARGIN_END};

glib::wrapper! {
    pub struct StoxDataGrid(ObjectSubclass<imp::StoxDataGrid>)
        @extends Box, Widget,
        @implements Actionable, Accessible, Buildable, ConstraintTarget;
}

lazy_static! {
    static ref UPDATE_LOCK: Arc<Mutex<()>> = Arc::new(Mutex::new(()));
}

macro_rules! imp_clone {
    ($imp:expr, $($names:ident),* $(,)?) => {
        $(
            let $names = $imp.$names.borrow().clone();
        )*
    }
}

macro_rules! set_labels {
    ($string:literal, $($labels:ident),* $(,)?) => {
        $(
            $labels.set_label($string);
        )*
    }
}

macro_rules! pixel_width {
    ($layout:expr) => {
        $layout.pixel_size().0
    };
}

const ELLIPSIS: &str = "\u{2026}";

fn set_label_with_max_width(label: &Label, text: &str, max_width: i32) -> i32 {
    // This is inefficient but it should be fine because it isn't run often and
    // the loop is only executed when there is an unusually long string to show.

    // TODO: This should go from left to right instead of right to left

    let mut text = text.to_string();

    let layout = label.layout();
    layout.set_width(max_width * pango::SCALE);
    layout.set_ellipsize(pango::EllipsizeMode::End);
    layout.set_text(&text);

    label.set_tooltip_text(None);

    let mut ellipsized = false;

    while layout.is_ellipsized() || pixel_width!(layout) > max_width {
        if !ellipsized {
            label.set_tooltip_text(Some(&text));
            ellipsized = true;
        }

        text.pop();
        layout.set_text(&(text.clone() + ELLIPSIS));
    }

    if ellipsized {
        text.push_str(ELLIPSIS);
    }

    label.set_label(&text);

    pixel_width!(layout)
}

impl StoxDataGrid {
    pub fn new() -> Self {
        let obj: StoxDataGrid = Object::builder().build();

        obj
    }

    pub fn update(
        &self,
        symbol: String,
        force_update: bool,
        is_saved: bool,
        is_default: bool,
    ) -> bool {
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

        if is_saved {
            self.imp().save_btn.borrow().hide();
            self.imp().unsave_btn.borrow().show();
        } else {
            self.imp().save_btn.borrow().show();
            self.imp().unsave_btn.borrow().hide();
        }

        let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);

        let symbol = RefCell::new(symbol);

        std::thread::spawn(clone!(
            @strong symbol => move || match stox_get_datagrid_info(&symbol.borrow()) {
                Ok((main_info, extended_info, stats_info)) => {
                    let quotes = stox_get_quotes(symbol.borrow().to_string(), "1d");
                    if quotes.is_err() {
                        sender.send(None).unwrap();
                        return;
                    }

                    sender
                        .send(Some((main_info, extended_info, stats_info, quotes.unwrap())))
                        .unwrap()
                }
                Err(_) => sender.send(None).unwrap(),
            }
        ));

        imp_clone!(
            self.imp(),
            symbol_label,
            name_label,
            latest_quote_label,
            market_change_label,
            info_label,
            notebook,
            save_btn,
            unsave_btn,
            refresh_btn,
            open_label,
            high_label,
            low_label,
            volume_label,
            pe_ratio_label,
            market_cap_label,
            yield_label,
            beta_label,
            eps_label,
        );

        set_labels!(
            "--",
            name_label,
            latest_quote_label,
            market_change_label,
            info_label,
            open_label,
            high_label,
            low_label,
            volume_label,
            pe_ratio_label,
            market_cap_label,
            yield_label,
            beta_label,
            eps_label,
        );

        market_change_label.set_css_classes(&[]);
        symbol_label.set_css_classes(&[]);

        set_label_with_max_width(
            &symbol_label,
            &symbol.borrow(),
            GRID_WIDTH
                - SYMBOL_LABEL_MARGIN_END
                - pixel_width!(latest_quote_label.layout())
                - pixel_width!(name_label.layout()),
        );

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
            clone!(@strong self as this, @strong symbol => move |complete_info| {
                let mut ok = true;

                match complete_info {
                    Some((main_info, extended_info, stats_info, quotes)) => {
                        latest_quote_label.set_label(&main_info.last_quote);
                        market_change_label.set_label(&format!(
                            "{} ({})",
                            &extended_info.market_change, &extended_info.market_change_percent,
                        ));

                        if extended_info.market_change_neg() {
                            market_change_label.set_css_classes(&["market_change_neg"]);
                        } else {
                            market_change_label.set_css_classes(&["market_change_pos"]);
                        }

                        let quote_box_width = std::cmp::max(
                            pixel_width!(market_change_label.layout()),
                            pixel_width!(latest_quote_label.layout()),
                        );
                        let remaining_width = GRID_WIDTH - SYMBOL_LABEL_MARGIN_END - quote_box_width;

                        let symbol_label_width = set_label_with_max_width(
                            &symbol_label,
                            &symbol.borrow(),
                            remaining_width / 2,
                        );
                        set_label_with_max_width(
                            &name_label,
                            &main_info.name,
                            remaining_width - symbol_label_width,
                        );

                        info_label.set_label(&format!(
                            "{} - {}",
                            extended_info.exchange_name, main_info.currency
                        ));

                        open_label.set_label(&stats_info.open);
                        high_label.set_label(&stats_info.high);
                        low_label.set_label(&stats_info.low);
                        volume_label.set_label(&stats_info.volume);
                        pe_ratio_label.set_label(&stats_info.pe_ratio);
                        market_cap_label.set_label(&stats_info.market_cap);
                        yield_label.set_label(&stats_info.dividend_yield);
                        beta_label.set_label(&stats_info.beta);
                        eps_label.set_label(&stats_info.eps);

                        this.imp().construct_graph(main_info, extended_info, quotes);
                    }
                    None => {
                        ok = false;

                        set_labels!(
                            "???",
                            name_label,
                            latest_quote_label,
                            market_change_label,
                            info_label,
                            open_label,
                            high_label,
                            low_label,
                            volume_label,
                            pe_ratio_label,
                            market_cap_label,
                            yield_label,
                            beta_label,
                            eps_label,
                        );

                        set_label_with_max_width(
                            &symbol_label,
                            &symbol.borrow(),
                            GRID_WIDTH - SYMBOL_LABEL_MARGIN_END
                                - pixel_width!(latest_quote_label.layout())
                                - pixel_width!(name_label.layout()),
                        );

                        notebook.append_page(
                            &Label::new(Some(&gettext("The graph could not be loaded."))),
                            Some(&Label::new(Some(&gettext("Error")))),
                        );
                    }
                }

                save_btn.set_sensitive(!is_default || ok);
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
