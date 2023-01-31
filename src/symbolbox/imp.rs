use std::cell::RefCell;
use std::cell::Cell;

use gtk4::glib::subclass::types::ObjectSubclass;
use gtk4::subclass::prelude::*;
use gtk4::*;
use gtk4::prelude::*;
use gtk4::glib::*;
use once_cell::sync::Lazy;
#[derive(Default)]
pub struct StoxSidebarItem {child: RefCell<Option<gtk4::Widget>>, symbol: Cell<String>}

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
                let symbol = _value.get::<Option<String>>().expect("Failed to get value").unwrap();
                self.symbol.set(symbol);
                self.constructed();  // ensure we reconstruct
            }
        }
    }

    fn property(&self, _id: usize, _pspec: &ParamSpec) -> Value {
        match _pspec.name() {
            _ => {
                self.symbol.take().to_string().to_value()
            }
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
            .   build();

            grid.set_parent(&*obj);
            grid.attach(&symbol_label, 0, 0, 100, 200);
            grid.attach(&quote_box, 0, 0, 100, 100);
            grid.attach_next_to(&desc, Some(&symbol_label), PositionType::Bottom, 100, 100);
            obj.set_child_visible(true);
            *self.child.borrow_mut() = Some(grid.upcast::<gtk4::Widget>());
        }
    }
}

impl ListBoxRowImpl for StoxSidebarItem {}

impl WidgetImpl for StoxSidebarItem {}