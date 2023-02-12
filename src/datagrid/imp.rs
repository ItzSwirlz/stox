use std::cell::RefCell;

use gtk4::builders::DrawingAreaBuilder;
use gtk4::glib::subclass::types::ObjectSubclass;
use gtk4::glib::*;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::*;

use crate::data_helper::*;

use once_cell::sync::Lazy;

#[derive(Default)]
pub struct StoxDataGrid {
    pub symbol_label: RefCell<Label>,
    pub name_label: RefCell<Label>,
    pub latest_quote: RefCell<Label>,
    pub delta_label: RefCell<Label>,
    pub save_btn: RefCell<Button>,
    pub unsave_btn: RefCell<Button>,
    pub notebook: RefCell<Notebook>,
}

#[glib::object_subclass]
impl ObjectSubclass for StoxDataGrid {
    const NAME: &'static str = "StoxDataGrid";
    type Type = super::StoxDataGrid;
    type ParentType = gtk4::Box;
}

impl ObjectImpl for StoxDataGrid {
    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| vec![]);
        PROPERTIES.as_ref()
    }

    fn set_property(&self, _id: usize, _value: &Value, _pspec: &ParamSpec) {
        match _pspec.name() {
            _ => {
                self.constructed(); // ensure we reconstruct
            }
        }
    }

    fn property(&self, _id: usize, _pspec: &ParamSpec) -> Value {
        match _pspec.name() {
            _ => unimplemented!(),
        }
    }

    fn constructed(&self) {
        self.parent_constructed();

        let obj = self.obj();
        obj.set_focusable(true);
        obj.set_visible(true);

        let grid = Grid::builder()
            .halign(Align::Center)
            .width_request(850)
            .margin_start(10)
            .margin_end(10)
            .margin_top(10)
            .margin_bottom(10)
            .build();

        let symbol_label = Label::builder()
            .valign(Align::Baseline)
            .margin_end(10)
            .label("--")
            .name("symbol")
            .build();
        symbol_label.show();

        let name = Label::new(Some("--"));
        name.set_widget_name("company_name");
        name.set_valign(Align::Baseline);
        name.show();

        let latest_quote = Label::new(Some("--"));
        latest_quote.set_widget_name("latest_quote");
        latest_quote.set_halign(Align::End);
        latest_quote.show();

        let delta_label = Label::new(Some("--"));
        delta_label.set_halign(Align::End);
        delta_label.show();

        let notebook = Notebook::builder()
            .focusable(true)
            .hexpand(true)
            .height_request(350)
            .margin_top(10)
            .build();

        grid.attach(&symbol_label, 0, 0, 1, 1);
        grid.attach(&name, 1, 0, 1, 1);

        let quote_box = Box::builder()
            .spacing(0)
            .orientation(Orientation::Vertical)
            .halign(Align::End)
            .valign(Align::Center)
            .hexpand(true)
            .build();
        quote_box.append(&latest_quote);
        quote_box.append(&delta_label);

        grid.attach(&quote_box, 2, 0, 1, 1);
        grid.attach(&notebook, 0, 1, 3, 2);

        grid.show();
        grid.set_parent(&*obj);

        let btns_box = Box::builder()
            .spacing(10)
            .orientation(Orientation::Horizontal)
            .margin_top(10)
            .build();

        {
            let save_btn_box = Box::new(Orientation::Horizontal, 6);

            let star_img = Image::from_icon_name("non-starred");
            save_btn_box.append(&star_img);

            let save_label = Label::new(Some("Save"));
            save_btn_box.append(&save_label);

            let save_btn = Button::builder().child(&save_btn_box).build();
            btns_box.append(&save_btn);

            save_btn.hide();

            *self.save_btn.borrow_mut() = save_btn;
        }

        {
            let unsave_btn_box = Box::new(Orientation::Horizontal, 6);

            let star_img = Image::from_icon_name("starred");
            unsave_btn_box.append(&star_img);

            let unsave_label = Label::new(Some("Unsave"));
            unsave_btn_box.append(&unsave_label);

            let unsave_btn = Button::builder().child(&unsave_btn_box).build();
            btns_box.append(&unsave_btn);

            unsave_btn.hide();

            *self.unsave_btn.borrow_mut() = unsave_btn;
        }

        {
            let refresh_btn_box = Box::new(Orientation::Horizontal, 6);

            let refresh_img = Image::from_icon_name("view-refresh");
            refresh_btn_box.append(&refresh_img);

            let refresh_label = Label::new(Some("Refresh"));
            refresh_btn_box.append(&refresh_label);

            let refresh_btn = Button::builder().child(&refresh_btn_box).build();

            refresh_btn.connect_clicked(clone!(@weak self as this => move |_| {
                let symbol = this.symbol_label.borrow().label().to_string();
                if symbol == "--" {
                    return;
                }

                this.obj().update(symbol, true, this.unsave_btn.borrow().is_visible());
            }));

            btns_box.append(&refresh_btn);
        }

        grid.attach(&btns_box, 0, 3, 3, 1);

        *self.symbol_label.borrow_mut() = symbol_label;
        *self.name_label.borrow_mut() = name;
        *self.latest_quote.borrow_mut() = latest_quote;
        *self.delta_label.borrow_mut() = delta_label;
        *self.notebook.borrow_mut() = notebook;
    }
}

impl BoxImpl for StoxDataGrid {}

impl WidgetImpl for StoxDataGrid {}

impl StoxDataGrid {
    pub fn construct_graph(&self) {
        let symbol = self.symbol_label.borrow().text().to_string();

        let x_axis = stox_get_chart_x_axis(symbol.clone(), "1d");
        if !x_axis.is_ok() {
            return;
        }
        let x_axis = x_axis.unwrap();

        let y_axis = stox_get_chart_y_axis(symbol.clone());
        if !y_axis.is_ok() {
            return;
        }
        let y_axis = y_axis.unwrap();

        let quotes = stox_get_quotes(symbol);

        let drawing_area = DrawingArea::new();

        self.notebook
            .take()
            .append_page(&drawing_area, Some(&Label::new(Some("1D"))));

        drawing_area.set_draw_func(move |drawing_area, cr, width, height| {
            let mut x_iter = x_axis.iter();
            let mut y_iter = y_axis.iter();
            cr.set_source_rgb(56.0 / 255.0, 56.0 / 255.0, 56.0 / 255.0); // Background color
            cr.paint().unwrap();
            cr.set_line_width(1.0);

            cr.set_source_rgb(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0); // Set the grid lines color
            let mut x_points: Vec<i32> = vec![];
            let mut y_points: Vec<i32> = vec![];
            for x_grid_line in (0..(width)).step_by(width as usize / 8 as usize) {
                cr.move_to(x_grid_line as f64, height as f64 - 20.0);
                cr.line_to(x_grid_line as f64, (-1 * height) as f64);
                cr.stroke().unwrap();
                x_points.push(x_grid_line);

                cr.move_to(x_grid_line as f64 - 2.0, height as f64 - 5.0);
                cr.show_text(x_iter.next().unwrap()).unwrap();
            }

            for y_grid_line in (0..(height)).step_by(height as usize / 4 as usize).rev() {
                cr.move_to(0.0, y_grid_line as f64);
                cr.line_to(width as f64, y_grid_line as f64);
                cr.stroke().unwrap();
                y_points.push(y_grid_line);

                cr.move_to(2.0, y_grid_line as f64);
                cr.show_text(&*y_iter.next().unwrap().to_string());
            }

            cr.set_source_rgb(0.0, 255.0, 0.0);
            let lines_step = quotes.len();
            let mut quote_iter = quotes.iter();
            cr.move_to(0 as f64, height as f64);
            for i in (0..=width).step_by(width as usize / (lines_step - 50)) {
                let next = quote_iter.next();
                if next != None {
                    cr.line_to(i as f64, (next.unwrap().to_owned()) as f64 / height as f64);
                    cr.stroke().unwrap();
                    cr.line_to(i as f64, (next.unwrap().to_owned()) as f64);
                }
            }
        });
        drawing_area.show();
    }
}
