mod symbolbox;
mod axes;

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
    let b = Box::new(Orientation::Vertical, 10);
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

    let dow = StoxSidebarItem::new("^DJI");
    dow.0.show();

    let aapl = StoxSidebarItem::new("AAPL");
    aapl.0.show();
    
    let msft = StoxSidebarItem::new("MSFT");
    msft.0.show();

    let sidebar = ListBox::new();
    sidebar.set_height_request(800);
    sidebar.append(&searchbar_row);
    sidebar.append(&dow.0);
    sidebar.append(&aapl.0);
    sidebar.append(&msft.0);

    let viewport = Viewport::builder()
        .child(&sidebar)
        .height_request(500)
        .build();

    viewport.show();
 
    let scroll_window = ScrolledWindow::builder()
        .child(&viewport)
        .halign(Align::Center)
        .focusable(true)
        .min_content_width(300)
        .max_content_width(300)
        .min_content_height(800)
        .build();

    b.append(&scroll_window);

    let window = ApplicationWindow::builder()
        .application(app)
        .child(&b)
        .title("Stox")
        .default_height(800)
        .build();

    window.set_application(Some(app));
    window.present();

    StoxSidebarItem::start_ticking(aapl.1.to_string(), aapl.2, aapl.3);
    StoxSidebarItem::start_ticking(msft.1.to_string(), msft.2, msft.3);
    StoxSidebarItem::start_ticking(dow.1.to_string(), dow.2, dow.3);
}
