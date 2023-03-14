mod imp;

use gtk4::*;

glib::wrapper! {
    pub struct StoxSidebarItem(ObjectSubclass<imp::StoxSidebarItem>)
        @extends ListBoxRow, Widget,
        @implements Actionable, Accessible, Buildable, ConstraintTarget;
}

impl StoxSidebarItem {
    pub fn new(symbol: &str, searched: bool) -> Self {
        glib::Object::builder()
            .property("symbol", &symbol)
            .property("searched", &searched)
            .build()
    }

    pub fn tick(item: imp::StoxSidebarItem) {
        // fancy hack for making the function use the implementation
        imp::StoxSidebarItem::tick(&item);
    }
}
