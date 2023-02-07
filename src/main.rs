mod config;
mod data_helper;
mod datagrid;
mod dialogs;
mod fs_persistence;
mod sidebar_item;

use config::*;
use data_helper::stox_search_symbol;
use datagrid::StoxDataGrid;
use fs_persistence::{read_saved_stocks, write_saved_stocks};
use sidebar_item::StoxSidebarItem;

use gettextrs::*;

use glib::subclass::types::ObjectSubclassIsExt;
use glib::SourceId;

use gtk4::gdk::Display;
use gtk4::glib::clone;
use gtk4::prelude::*;
use gtk4::*;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::*;

const APP_ID: &str = "org.github.ItzSwirlz.stox";

fn main() {
    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR)
        .unwrap_or_else(|_| panic!("Unable to bind text domain for {}", GETTEXT_PACKAGE));
    bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8").unwrap();
    textdomain(GETTEXT_PACKAGE)
        .unwrap_or_else(|_| panic!("Unable to switch to text domain {}", GETTEXT_PACKAGE));

    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run();
}

fn build_ui(app: &Application) {
    let mut error_loading_saved_stocks = false;

    let mut saved_stocks = read_saved_stocks();
    if let Err(_) = saved_stocks {
        error_loading_saved_stocks = true;
        saved_stocks = Ok(vec![]);
    }

    let saved_stocks = Rc::new(RefCell::new(saved_stocks.unwrap()));

    let css_provider = CssProvider::new();
    css_provider.load_from_data(
        "
            #sidebar_symbol_label {
                font-size: 23px;
            }

            #symbol {
                font-weight: bold;
                font-size: 50px;
            }

            #company_name {
                font-style: italic;
                font-size: 25px;
            }

            #latest_quote {
                font-size: 25px;
            }
        "
        .as_bytes(),
    );

    StyleContext::add_provider_for_display(
        &Display::default().unwrap(),
        &css_provider,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let b = Box::new(Orientation::Horizontal, 10);

    let window = ApplicationWindow::builder()
        .application(app)
        .child(&b)
        .title("Stox")
        .default_height(800)
        .build();

    let searchbar = SearchEntry::builder()
        .focusable(true)
        .placeholder_text(&gettext("Search for a symbol..."))
        .build();
    searchbar.show();

    let searchbar_row = ListBoxRow::builder()
        .height_request(50)
        .focusable(true)
        .margin_start(10)
        .margin_end(10)
        .margin_top(10)
        .margin_bottom(10)
        .child(&searchbar)
        .build();

    let sidebar = ListBox::new();
    sidebar.set_height_request(800);
    sidebar.append(&searchbar_row);

    let sidebar_symbols: Arc<Mutex<Vec<StoxSidebarItem>>> = Arc::new(Mutex::new(Vec::new()));
    for ticker in &*saved_stocks.borrow() {
        let sidebar_item = StoxSidebarItem::new(&ticker, false);
        sidebar_item.show();
        sidebar.append(&sidebar_item);
        sidebar_symbols.lock().unwrap().push(sidebar_item);
    }

    let (debounce_sender, debounce_receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    let debounce_source_id = Arc::new(Mutex::new(None::<SourceId>));

    searchbar.connect_search_changed(clone!(@weak debounce_source_id => move |search| {
        let mut debounce_source_id = debounce_source_id.lock().unwrap();
        if let Some(debounce_source_id) = debounce_source_id.take() {
            debounce_source_id.remove();
        }

        let query = search.text().to_string();

        *debounce_source_id = Some(glib::timeout_add_local_once(
            std::time::Duration::from_millis(250),
            clone!(@strong debounce_sender => move || {
                debounce_sender.send(query).unwrap();
            }),
        ));
    }));

    debounce_receiver.attach(
        None,
        clone!(
            @strong sidebar_symbols, @weak sidebar, @weak saved_stocks => @default-panic,
            move |query: String| {
                *debounce_source_id.lock().unwrap() = None;

                sidebar_symbols.lock().unwrap().retain(|item| {
                    let symbol = item.property::<String>("symbol");
                    let is_searched = item.property::<bool>("searched");

                    let is_saved = (*saved_stocks.borrow()).contains(&symbol);

                    if query.is_empty() && !is_searched && is_saved {
                        item.show()
                    } else {
                        item.hide();
                    }

                    !is_searched
                });

                // Do not try to ping Yahoo with invalid characters.
                if query.is_empty() || !query.is_ascii() {
                    return Continue(true)
                }

                let quotes = stox_search_symbol(&query);
                for i in quotes.iter() {
                    let sidebar_item = StoxSidebarItem::new(&i.symbol, true);
                    sidebar_item.show();
                    sidebar.append(&sidebar_item);
                    sidebar_symbols.lock().unwrap().push(sidebar_item);
                }

                Continue(true)
            }
        ),
    );

    let viewport = Viewport::builder()
        .child(&sidebar)
        .height_request(500)
        .build();

    viewport.show();

    let scroll_window = ScrolledWindow::builder()
        .child(&viewport)
        .halign(Align::Center)
        .focusable(true)
        .min_content_width(325)
        .max_content_width(325)
        .min_content_height(800)
        .build();

    b.append(&scroll_window);

    let datagrid = RefCell::new(StoxDataGrid::new());
    datagrid
        .borrow()
        .imp()
        .save_btn
        .borrow()
        .connect_clicked(clone!(
            @strong datagrid, @weak sidebar, @weak sidebar_symbols, @weak saved_stocks, @weak window =>
            move |_| {
                if error_loading_saved_stocks {
                    dialogs::show_saving_unsaving_disabled_dialog(&window);
                    return;
                }

                let symbol = datagrid.borrow().imp().symbol_label.borrow().label().to_string();
                (*saved_stocks).borrow_mut().push(symbol.clone());

                if let Err(_) = write_saved_stocks((*saved_stocks).borrow().to_vec()) {
                    dialogs::show_save_stock_failed_dialog(&window);
                    return;
                }

                datagrid.borrow().imp().save_btn.borrow().hide();
                datagrid.borrow().imp().unsave_btn.borrow().show();

                let sidebar_item = StoxSidebarItem::new(&symbol, false);
                sidebar.append(&sidebar_item);

                if searchbar.text().to_string().is_empty() {
                    sidebar_item.show();
                    sidebar_item.activate();
                } else {
                    sidebar_item.hide();
                }

                sidebar_symbols.lock().unwrap().push(sidebar_item);
            }
        ));
    datagrid.borrow().imp().unsave_btn.borrow().connect_clicked(
        clone!(@strong datagrid, @strong saved_stocks, @weak sidebar, @weak window => move |_| {
            if error_loading_saved_stocks {
                dialogs::show_saving_unsaving_disabled_dialog(&window);
                return;
            }

            let symbol = datagrid.borrow().imp().symbol_label.borrow().label().to_string();

            let index = (*saved_stocks).borrow_mut().iter().position(|value| symbol == value.as_str());
            (*saved_stocks).borrow_mut().remove(index.unwrap());

            if let Err(_) = write_saved_stocks((*saved_stocks).borrow().to_vec()) {
                dialogs::show_unsave_stock_failed_dialog(&window);
                return;
            }

            datagrid.borrow().imp().save_btn.borrow().show();
            datagrid.borrow().imp().unsave_btn.borrow().hide();

            let mut child = sidebar.first_child().unwrap().next_sibling().unwrap();
            while child.property::<String>("symbol") != symbol || child.property("searched") {
                match child.next_sibling() {
                    Some(next_child) => child = next_child,
                    None => break
                }
            }

            sidebar.remove(&child);
        }),
    );

    b.append(&*datagrid.borrow());

    sidebar.connect_row_selected(move |_, row| {
        if let Some(row) = row {
            if row.widget_name() != "StoxSidebarItem" {
                return;
            }

            let symbol = row.property::<String>("symbol");
            let symbol = symbol.as_str();

            datagrid.borrow().update(
                symbol.to_string(),
                false,
                (*saved_stocks).borrow().contains(&symbol.to_string()),
            );
        }
    });

    if error_loading_saved_stocks {
        dialogs::show_load_saved_stocks_failed_dialog(&window);
    }

    let header_bar = HeaderBar::new();
    header_bar.set_title_widget(Some(&Label::new(Some("Stox"))));
    header_bar.set_show_title_buttons(true);

    window.set_titlebar(Some(&header_bar));
    window.set_application(Some(app));
    window.present();
}
