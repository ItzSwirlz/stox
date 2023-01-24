mod imp;

use gtk4::*;
use gtk4::traits::*;

use gtk4::glib::*;
use yahoo_finance_api as yahoo;

glib::wrapper! {
    pub struct StoxSidebarItem(ObjectSubclass<imp::ListBoxRow>)
        @extends ListBoxRow, Widget,
        @implements Actionable, Accessible, Buildable, ConstraintTarget;
}

impl StoxSidebarItem {
    pub fn new(symbol: &str) -> (ListBoxRow, &str, Label, Label) {
        let quote_label = Label::builder()
                .halign(Align::End)
                .justify(Justification::Right)
                .label("--.--")  // placeholder until pinged
                .build();

        let quote_box = Box::builder()
            .orientation(Orientation::Vertical)
            .valign(Align::Start)
            .build();
    
        quote_box.append(&quote_label);
        quote_box.show();

        let desc = Label::builder()
                .halign(Align::Start)
                .label("--.--")  // placeholder until pinged
                .build();

        desc.show();  // we won't let the UI wait for the yahoo ping

        let symbol_label = Label::builder()
                .halign(Align::Start)
                .label(&symbol)
                .visible(true)
                .build();

        if symbol == "^DJI" {
            symbol_label.set_text("Dow Jones");  // nobody thinks of it as "^DJI"
        }

        symbol_label.show();

        let grid = Grid::builder()
            .margin_start(10)
            .margin_end(10)
            .margin_top(10)
            .margin_bottom(10)
            .column_homogeneous(true)
            .hexpand(true)
            .visible(true)
            .build();

        grid.attach(&symbol_label, 0, 0, 100, 100);
        grid.attach(&quote_box, 0, 0, 100, 100);
        grid.attach_next_to(&desc, Some(&symbol_label), PositionType::Bottom, 100, 100);
        grid.show();

        let c = gtk4::ListBoxRow::builder()
            .height_request(100)
            .focusable(true)
            .child(&grid)
            .build();
        
        c.set_child(Some(&grid));
        c.show();
        
        return (c, symbol, desc, quote_label);
        
    }
    
    pub fn start_ticking(symbol: String, desc_label: Label, quote_label: Label) {
        // This practically creates another loop to monitor quote updates.
        let main_context = MainContext::default();

        // Don't pause the main loop to wait for our information
        main_context.spawn_local(clone!(@weak quote_label => async move {
            let provider = yahoo::YahooConnector::new();
            let latest_quote = provider.get_latest_quotes(symbol.as_str(), "1h").unwrap().last_quote().unwrap().close;
            let latest_quote = format!("{:.2}", latest_quote);  // limit to two decimal places

            quote_label.set_text(latest_quote.to_string().as_str());
            desc_label.set_text(provider.search_ticker(&symbol).unwrap().quotes[0].short_name.as_str());
        }));
    }
}