mod imp;

use gtk4::*;
use gtk4::traits::*;

use gtk4::glib::*;
use yahoo_finance_api as yahoo;

glib::wrapper! {
    pub struct StoxSidebar(ObjectSubclass<imp::ListBoxRow>)
        @extends ListBoxRow, Widget,
        @implements Actionable, Accessible, Buildable, ConstraintTarget;
}

impl StoxSidebar {
    pub fn create(symbol: &str) -> (ListBoxRow, &str, Label) {
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
        
        return (c, symbol, quote_label);
        
    }
    
    pub fn start_ticking(symbol: String, label: Label) {
        // This practically creates another loop to monitor quote updates.
        let main_context = MainContext::default();

        // Don't pause the main loop to wait for our information
        main_context.spawn_local(clone!(@weak label => async move {
            let provider = yahoo::YahooConnector::new();
            let response = provider.get_latest_quotes(symbol.as_str(), "1h").unwrap().last_quote().unwrap().close;
            let response = format!("{:.2}", response);  // limit to two decimal places
            label.set_text(response.to_string().as_str());
        }));
    }
}