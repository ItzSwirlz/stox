mod imp;

use gtk4::traits::*;
use gtk4::*;

use gtk4::glib::*;

glib::wrapper! {
    pub struct StoxSidebarItem(ObjectSubclass<imp::StoxSidebarItem>)
        @extends ListBoxRow, Widget,
        @implements Actionable, Accessible, Buildable, ConstraintTarget;
}

impl StoxSidebarItem {
    pub fn new(symbol: &str) -> Self {
        let obj: StoxSidebarItem = Object::builder()
            .property("symbol", symbol.to_value())
            .build();

        return obj;
    }
}
