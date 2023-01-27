use gtk4::glib::subclass::types::ObjectSubclass;
use gtk4::subclass::prelude::*;

#[derive(Default)]
pub struct StoxDataGrid {}

#[glib::object_subclass]
impl ObjectSubclass for StoxDataGrid {
    const NAME: &'static str = "DataGrid";
    type Type = super::StoxDataGrid;
    type ParentType = gtk4::Grid;
}

impl ObjectImpl for StoxDataGrid {}

impl GridImpl for StoxDataGrid {}

impl WidgetImpl for StoxDataGrid {}
