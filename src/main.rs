mod data_helper;
mod datagrid;
mod symbolbox;

use datagrid::StoxDataGrid;
use glib::Value;
use gtk4::prelude::*;
use gtk4::*;
use symbolbox::StoxSidebarItem;

const APP_ID: &str = "org.github.ItzSwirlz.stox";

fn main() {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run();
}

fn build_ui(app: &Application) {
    let b = Box::new(Orientation::Horizontal, 10);
    let searchbar = SearchEntry::builder()
        .focusable(true)
        .placeholder_text("Search for a symbol...")
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
        let sidebar_item = StoxSidebarItem::new(ticker.to_string());
        sidebar_item.show();
        sidebar.append(&sidebar_item);
       // sidebar_item.start_ticking();
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

    let datagrid = StoxDataGrid::new_initial();
    datagrid.show();

    b.append(&datagrid);

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
