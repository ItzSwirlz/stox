use std::cell::RefCell;

use gtk4::glib::subclass::types::ObjectSubclass;
use gtk4::glib::*;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::*;

use once_cell::sync::Lazy;

use crate::data_helper::stox_get_main_info;

#[derive(Default, CompositeTemplate)]
#[template(resource = "/org/itzswirlz/stox/resources/ui/stoxsidebaritem.ui")]
pub struct StoxSidebarItem {
    #[template_child]
    symbol_label: TemplateChild<Label>,
    symbol: RefCell<String>,
    searched: RefCell<bool>,
    #[template_child]
    desc_label: TemplateChild<Label>,
    #[template_child]
    quote_label: TemplateChild<Label>,
}

#[glib::object_subclass]
impl ObjectSubclass for StoxSidebarItem {
    const NAME: &'static str = "StoxSidebarItem";
    type Type = super::StoxSidebarItem;
    type ParentType = gtk4::ListBoxRow;

    fn class_init(klass: &mut Self::Class) {
        Self::bind_template(klass);
    }
    
    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for StoxSidebarItem {
    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
            vec![
                ParamSpecString::builder("symbol").build(),
                ParamSpecBoolean::builder("searched").build(),
            ]
        });
        PROPERTIES.as_ref()
    }

    fn set_property(&self, _id: usize, value: &Value, _pspec: &ParamSpec) {
        match _pspec.name() {
            "symbol" => {
                let symbol = value
                    .get::<Option<String>>()
                    .expect("Failed to get value")
                    .unwrap();
                *self.symbol.borrow_mut() = symbol;
                self.constructed(); // ensure we reconstruct
            }
            "searched" => {
                let searched = value.get::<bool>().unwrap();
                *self.searched.borrow_mut() = searched;
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _id: usize, _pspec: &ParamSpec) -> Value {
        match _pspec.name() {
            "symbol" => self.symbol.borrow().to_string().to_value(),
            "searched" => self.searched.borrow().to_value(),
            _ => unimplemented!(),
        }
    }

    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();
        obj.set_height_request(100);
        obj.set_width_request(325);
        obj.set_focusable(true);
        obj.set_visible(true);

        self.symbol_label.get().set_label(&self.symbol.borrow());
        self.start_ticking(self.symbol.borrow().to_string(), self.desc_label.get(), self.quote_label.get());
    }
}

impl BuildableImpl for StoxSidebarItem {}

impl ListBoxRowImpl for StoxSidebarItem {}

impl WidgetImpl for StoxSidebarItem {}

impl StoxSidebarItem {
    pub fn start_ticking(&self, symbol: String, desc_label: Label, quote_label: Label) {
        let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);

        std::thread::spawn(move || match stox_get_main_info(symbol.as_str()) {
            Ok(main_info) => sender.send(main_info).unwrap(),
            Err(_) => sender.send(("???".to_string(), "???".to_string())).unwrap(),
        });

        receiver.attach(None, move |(last_quote, short_name)| {
            quote_label.set_text(&last_quote.to_string());
            desc_label.set_text(&short_name.to_string());
            Continue(true)
        });
    }
}
