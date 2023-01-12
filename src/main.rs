use std::collections::HashMap;

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};

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
    // Create a window and set the title
    let window = ApplicationWindow::builder()
        .application(app)
        .title("My GTK App")
        .build();
    
    // Present window
    window.present();
}

#[tokio::main]
async fn quote() -> Result<(), Box<dyn std::error::Error>> {
    let resp = reqwest::get("https://query1.finance.yahoo.com/v11/finance/quoteSummary/aapl?modules=earningsHistory")
        .await?
        .text()
        .await?;
    println!("{:#?}", resp);
    Ok(())
}