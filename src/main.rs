mod config;
mod data_helper;
mod datagrid;
mod sidebar_item;

use gettextrs::*;
use std::cell::RefCell;

use datagrid::StoxDataGrid;
use gtk4::gdk::Display;
use gtk4::prelude::*;
use gtk4::*;
use sidebar_item::StoxSidebarItem;
use config::*;

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
    let css_provider = CssProvider::new();

    css_provider.load_from_data(
        r#"
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
        "#
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

    let tickers = ["^DJI", "AAPL", "MSFT"];
    for ticker in tickers {
        let sidebar_item = StoxSidebarItem::new(ticker);
        sidebar_item.show();
        sidebar.append(&sidebar_item);
    }
    //sidebar.connect_row_selected(move |_, _| {
    //    StoxDataGrid::update_symbol(datagrid, sidebar.selected_row().unwrap()
    //});

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
    b.append(&*datagrid.borrow());

    sidebar.connect_row_selected(move |_, row| {
        if let Some(row) = row {
            if row.widget_name() != "StoxSidebarItem" {
                return;
            }

            let symbol = row.property::<String>("symbol");
            (*datagrid.borrow()).update(symbol);
        }
    });

    let window = ApplicationWindow::builder()
        .application(app)
        .child(&b)
        .title("Stox")
        .default_height(800)
        .build();

    let header_bar = HeaderBar::new();
    header_bar.set_title_widget(Some(&Label::new(Some("Stox"))));
    header_bar.set_show_title_buttons(true);

    window.set_titlebar(Some(&header_bar));
    window.set_application(Some(app));
    window.present();
}
