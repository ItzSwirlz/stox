mod axes;

use axes::x_axis;
use chrono::prelude::*;

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Label};

use yahoo_finance_api as yahoo;

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
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 12);

    let provider = yahoo::YahooConnector::new();
    let response = provider.get_quote_range("AAPL", "60m", "1d").unwrap();

    let x_axis = x_axis(response);
    let x_axis: String = x_axis.into_iter().collect();

    let label = Label::new(Some(&x_axis));
    b.append(&label);

    b.show();

    let window = ApplicationWindow::builder()
        .application(app)
        .child(&b)
        .title("Stox")
        .width_request(200)
        .height_request(200)
        .build();

    window.present();
}