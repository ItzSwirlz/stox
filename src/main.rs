use serde_json::Value;

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Label};

const APP_ID: &str = "org.github.ItzSwirlz.stox";

fn main() {
    quote();
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run();
}

fn build_ui(app: &Application) {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    let q = quote().unwrap();
    let label = Label::new(Some(format!("Apple: {}", q.as_str()).as_str()));
    label.show();

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

#[tokio::main]
async fn quote() -> Result<String, Box<dyn std::error::Error>> {
    let resp = reqwest::get("https://query1.finance.yahoo.com/v11/finance/quoteSummary/AAPL?modules=financialData")
        .await?
        .text()
        .await?;
    let v: Value = serde_json::from_str(resp.as_str())?;

    println!("{:#?}", v["quoteSummary"]["result"][0]["financialData"]["currentPrice"]["raw"]);
    Ok(v["quoteSummary"]["result"][0]["financialData"]["currentPrice"]["raw"].to_string())
}