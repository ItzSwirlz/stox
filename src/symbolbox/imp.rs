use gtk4::subclass::prelude::*;
use gtk4::glib::subclass::types::ObjectSubclass;

#[derive(Default)]
pub struct ListBoxRow {}

#[glib::object_subclass]
impl ObjectSubclass for ListBoxRow {
    const NAME: &'static str = "SymbolBox";
    type Type = super::StoxSidebarItem;
    type ParentType = gtk4::ListBoxRow;
}

impl ObjectImpl for ListBoxRow {}

impl ListBoxRowImpl for ListBoxRow {}

impl WidgetImpl for ListBoxRow {}