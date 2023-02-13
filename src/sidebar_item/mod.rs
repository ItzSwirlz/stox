mod imp;

use gtk4::*;

glib::wrapper! {
    pub struct StoxSidebarItem(ObjectSubclass<imp::StoxSidebarItem>)
        @extends ListBoxRow, Widget,
        @implements Actionable, Accessible, Buildable, ConstraintTarget;
}

impl StoxSidebarItem {
    pub fn new(symbol: &str, searched: bool) -> Self {
        glib::Object::new(&[("symbol", &symbol), ("searched", &searched)])
    }
}
