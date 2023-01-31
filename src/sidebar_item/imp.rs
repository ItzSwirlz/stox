use std::borrow::BorrowMut;
use std::cell::Cell;
use std::cell::RefCell;

use rust_decimal::Decimal;
use rusty_money::{iso, Money};

use gtk4::glib::subclass::types::ObjectSubclass;
use gtk4::glib::*;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::*;

use once_cell::sync::Lazy;
use yahoo_finance_api as yahoo;

#[derive(Default)]
pub struct StoxSidebarItem {
    child: RefCell<Option<gtk4::Widget>>,
    symbol: Cell<String>,
    desc_label: Cell<Label>,
    quote_label: Cell<Label>,
}

#[glib::object_subclass]
impl ObjectSubclass for StoxSidebarItem {
    const NAME: &'static str = "StoxSidebarItem";
    type Type = super::StoxSidebarItem;
    type ParentType = gtk4::ListBoxRow;
}

impl ObjectImpl for StoxSidebarItem {
    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: Lazy<Vec<ParamSpec>> =
            Lazy::new(|| vec![ParamSpecString::builder("symbol").build()]);
        PROPERTIES.as_ref()
    }

    fn set_property(&self, _id: usize, _value: &Value, _pspec: &ParamSpec) {
        match _pspec.name() {
            _ => {
                let symbol = _value
                    .get::<Option<String>>()
                    .expect("Failed to get value")
                    .unwrap();
                self.symbol.set(symbol);
                self.constructed(); // ensure we reconstruct
            }
        }
    }

    fn property(&self, _id: usize, _pspec: &ParamSpec) -> Value {
        match _pspec.name() {
            _ => self.symbol.take().to_string().to_value(),
        }
    }

    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();
        obj.set_height_request(100);
        obj.set_focusable(true);
        obj.set_visible(true);

        let quote_label = Label::builder()
            .halign(Align::End)
            .justify(Justification::Right)
            .label("--.--") // placeholder until pinged
            .build();

        let quote_box = Box::builder()
            .orientation(Orientation::Vertical)
            .valign(Align::Start)
            .build();

        quote_box.append(&quote_label);
        quote_box.show();

        let desc = Label::builder()
            .halign(Align::Start)
            .label("--.--") // placeholder until pinged
            .build();

        desc.show(); // we won't let the UI wait for the yahoo ping

        let symbol = self.symbol.take();
        // Sometimes an empty string value can be initialized, ignore it
        if symbol.len() != 0 {
            let symbol_label = Label::builder()
                .halign(Align::Start)
                .label(symbol.as_str())
                .build();

            symbol_label.show();

            let grid = Grid::builder()
                .margin_start(10)
                .margin_end(10)
                .margin_top(10)
                .margin_bottom(10)
                .column_homogeneous(true)
                .hexpand(true)
                .build();

            grid.set_parent(&*obj);
            grid.attach(&symbol_label, 0, 0, 100, 100);
            grid.attach(&quote_box, 0, 0, 100, 100);
            grid.attach_next_to(&desc, Some(&symbol_label), PositionType::Bottom, 100, 100);
            obj.set_child_visible(true);
            *self.child.borrow_mut() = Some(grid.upcast::<gtk4::Widget>());

            StoxSidebarItem::start_ticking(self, symbol, desc, quote_label); // start ticking immediately
        }
    }
}

impl ListBoxRowImpl for StoxSidebarItem {}

impl WidgetImpl for StoxSidebarItem {}

impl StoxSidebarItem {
    pub fn start_ticking(&self, symbol: String, desc_label: Label, quote_label: Label) {
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
                let latest_quote = (latest_quote * 100.0).trunc() as i64;
                let latest_quote = Decimal::new(latest_quote, 2); // limit to two decimal places

                let ref short_name = provider.search_ticker(&symbol).unwrap().quotes[0].short_name;

                let url = format!(
                    "https://query1.finance.yahoo.com/v7/finance/options/{}",
                    symbol
                );
                let financial_data = reqwest::blocking::get(url).unwrap().text().unwrap();
                let financial_data: serde_json::Value =
                    serde_json::from_str(&financial_data).unwrap();

                let currency = financial_data["optionChain"]["result"][0]["quote"]["currency"]
                    .as_str()
                    .unwrap()
                    .to_uppercase();

                let latest_quote =
                    Money::from_decimal(latest_quote, iso::find(&currency).unwrap()).to_string();

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
