mod config;
mod data_helper;
mod datagrid;
mod fs_persistence;
mod sidebar_item;

use config::*;
use data_helper::stox_search_symbol;
use datagrid::StoxDataGrid;
use fs_persistence::{read_saved_stocks, write_saved_stocks};
use sidebar_item::StoxSidebarItem;

use gettextrs::*;

use glib::subclass::types::ObjectSubclassIsExt;
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
    let searchbar = SearchEntry::builder()
        .focusable(true)
        .placeholder_text(&gettext("Search for a symbol..."))
        .search_delay(250)
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

    searchbar.connect_search_changed(clone!(@weak sidebar => move |search| {
        if search.text().to_string().is_empty() {
            // Prevent panic
            return
        }
        for item in sidebar_symbols.lock().unwrap().iter() {
            item.hide();
        }
        sidebar_symbols.lock().unwrap().clear();

        let quotes = stox_search_symbol(&search.text().to_string());
        for i in quotes.iter() {
            let sidebar_item = StoxSidebarItem::new(&i.symbol, true);
            sidebar_item.show();
            sidebar.append(&sidebar_item);
            sidebar_symbols.lock().unwrap().push(sidebar_item);
        }
    }));

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
    datagrid.borrow().imp().save_btn.borrow().connect_clicked(
        clone!(@strong datagrid, @weak saved_stocks => move |_| {
            if error_loading_saved_stocks {
                return;
            }

            let symbol = datagrid.borrow().imp().symbol_label.borrow().label().to_string();
            (*saved_stocks).borrow_mut().push(symbol);

            if let Err(_) = write_saved_stocks((*saved_stocks).borrow().to_vec()) {
                return;
            }

            datagrid.borrow().imp().save_btn.borrow().hide();
            datagrid.borrow().imp().unsave_btn.borrow().show();
        }),
    );
    datagrid.borrow().imp().unsave_btn.borrow().connect_clicked(
        clone!(@strong datagrid, @weak sidebar, @weak saved_stocks => move |_| {
            if error_loading_saved_stocks {
                return;
            }

            let symbol = datagrid.borrow().imp().symbol_label.borrow().label().to_string();

            let index = (*saved_stocks).borrow_mut().iter().position(|value| symbol == value.as_str());
            (*saved_stocks).borrow_mut().remove(index.unwrap());

            if let Err(_) = write_saved_stocks((*saved_stocks).borrow().to_vec()) {
                return;
            }

            datagrid.borrow().imp().save_btn.borrow().show();
            datagrid.borrow().imp().unsave_btn.borrow().hide();

            let mut child = sidebar.first_child().unwrap().next_sibling().unwrap();
            while child.property::<String>("symbol") != symbol {
                match child.next_sibling() {
                    Some(next_child) => child = next_child,
                    None => break
                }
            }

            if child.property("searched") {
                return;
            }

            child.hide();
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

    let window = ApplicationWindow::builder()
        .application(app)
        .child(&b)
        .title("Stox")
        .default_height(800)
        .build();

    let error_dialog = MessageDialog::builder()
        .transient_for(&window)
        .modal(true)
        .buttons(ButtonsType::Ok)
        .text("Error")
        .secondary_text(concat!(
            "The saved stocks could not be loaded. Try restaring the app.\n",
            "\nTo prevent data loss, saving/unsaving stocks will be disabled until this is fixed."
        ))
        .message_type(MessageType::Error)
        .build();

    if error_loading_saved_stocks {
        error_dialog.run_async(|obj, _| obj.close());
    }

    let header_bar = HeaderBar::new();
    header_bar.set_title_widget(Some(&Label::new(Some("Stox"))));
    header_bar.set_show_title_buttons(true);

    window.set_titlebar(Some(&header_bar));
    window.set_application(Some(app));
    window.present();
}
