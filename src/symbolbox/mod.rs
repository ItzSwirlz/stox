mod imp;

use gtk4::traits::*;
use gtk4::*;

use gtk4::glib::*;
use yahoo_finance_api as yahoo;

glib::wrapper! {
    pub struct StoxSidebarItem(ObjectSubclass<imp::StoxSidebarItem>)
        @extends ListBoxRow, Widget,
        @implements Actionable, Accessible, Buildable, ConstraintTarget;
}

impl StoxSidebarItem {
    pub fn new(symbol: &str) -> Self {
        let obj: StoxSidebarItem = Object::builder().property("symbol", symbol.to_value()).build();

        return obj;
    }

    pub fn start_ticking(&mut self, symbol: String, desc_label: Label, quote_label: Label) {
        let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);

        std::thread::spawn(move || {
            let provider = yahoo::YahooConnector::new();

            loop {
                let latest_quote = provider
                    .get_latest_quotes(symbol.as_str(), "1h")
                    .unwrap()
                    .last_quote()
                    .unwrap()
                    .close;
                let latest_quote = format!("{:.2}", latest_quote); // limit to two decimal places

                let ref short_name = provider.search_ticker(&symbol).unwrap().quotes[0].short_name;

                sender.send((latest_quote, short_name.clone())).unwrap();

                std::thread::sleep(std::time::Duration::from_secs(60));
            }
        });

        receiver.attach(None, move |(latest_quote, short_name)| {
            quote_label.set_text(&latest_quote.to_string());
            desc_label.set_text(&short_name.to_string());

            Continue(true)
        });
    }
}