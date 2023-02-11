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
    #[template_child]
    desc_label: TemplateChild<Label>,
    #[template_child]
    quote_label: TemplateChild<Label>,
    #[template_child]
    delta_label: TemplateChild<Label>,
    symbol: RefCell<String>,
    searched: RefCell<bool>,
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
        self.symbol_label
            .get()
            .set_tooltip_text(Some(&self.symbol.borrow()));

        self.start_ticking(
            self.symbol.borrow().to_string(),
            self.desc_label.get(),
            self.quote_label.get(),
            self.symbol_label.get(),
            self.delta_label.get(),
        );
    }
}

impl BuildableImpl for StoxSidebarItem {}

impl ListBoxRowImpl for StoxSidebarItem {}

impl WidgetImpl for StoxSidebarItem {}

impl StoxSidebarItem {
    pub fn start_ticking(
        &self,
        symbol: String,
        desc_label: Label,
        quote_label: Label,
        symbol_label: Label,
        delta_label: Label,
    ) {
        let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);

        std::thread::spawn(move || match stox_get_main_info(symbol.as_str()) {
            Ok(main_info) => sender.send(Some(main_info)).unwrap(),
            Err(_) => sender.send(None).unwrap(),
        });

        receiver.attach(None, move |main_info| {
            match main_info {
                Some(main_info) => {
                    quote_label.set_text(&main_info.last_quote);
                    quote_label.set_tooltip_text(Some(&main_info.last_quote));

                    desc_label.set_text(&main_info.short_name);
                    desc_label.set_tooltip_text(Some(&main_info.short_name));

                    delta_label.set_text(&main_info.delta);
                    delta_label.set_tooltip_text(Some(&main_info.delta));

                    if main_info.delta.chars().nth(0).unwrap() == '-' {
                        delta_label.set_css_classes(&["delta_negative"]);
                    } else {
                        delta_label.set_css_classes(&["delta_positive"]);
                    }

                    if main_info.instrument_type == "FUTURE" {
                        symbol_label.set_markup(
                            String::from(
                                "<span foreground=\"#2ec27e\">".to_owned()
                                    + &symbol_label.text().to_string()
                                    + "</span>",
                            )
                            .as_str(),
                        );
                    }
                }
                None => {
                    quote_label.set_text("???");
                    desc_label.set_text("???");
                    delta_label.set_text("???");
                }
            }

            Continue(true)
        });
    }
}
