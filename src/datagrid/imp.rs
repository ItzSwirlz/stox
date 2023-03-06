use std::cell::RefCell;

use gtk4::glib::subclass::types::ObjectSubclass;
use gtk4::glib::*;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::*;

use crate::data_helper::*;

use once_cell::sync::Lazy;

use gettextrs::gettext;

#[derive(Default)]
pub struct StoxDataGrid {
    pub symbol_label: RefCell<Label>,
    pub name_label: RefCell<Label>,
    pub latest_quote_label: RefCell<Label>,
    pub market_change_label: RefCell<Label>,
    pub info_label: RefCell<Label>,
    pub save_btn: RefCell<Button>,
    pub unsave_btn: RefCell<Button>,
    pub refresh_btn: RefCell<Button>,
    pub notebook: RefCell<Notebook>,
    pub open_label: RefCell<Label>,
    pub high_label: RefCell<Label>,
    pub low_label: RefCell<Label>,
    pub volume_label: RefCell<Label>,
    pub pe_ratio_label: RefCell<Label>,
    pub market_cap_label: RefCell<Label>,
    pub yield_label: RefCell<Label>,
    pub beta_label: RefCell<Label>,
    pub eps_label: RefCell<Label>,
}

pub const GRID_WIDTH: i32 = 850;
pub const SYMBOL_LABEL_MARGIN_END: i32 = 10;

macro_rules! stat_col {
    ($stats_grid:ident, $stat_name:expr, $col:literal, $row:literal) => {{
        let stat_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .css_classes(vec!["stats_col_".to_owned() + &($col + 1).to_string()])
            .width_request(GRID_WIDTH / 3)
            .build();

        let stat_name_label = Label::builder()
            .label($stat_name)
            .halign(Align::Start)
            .margin_end(10)
            .build();

        let stat_data_label = Label::builder()
            .label("--")
            .css_classes(vec!["stat_data_label"])
            .ellipsize(pango::EllipsizeMode::Start)
            .hexpand(true)
            .max_width_chars(1)
            .xalign(1.0)
            .build();

        stat_box.append(&stat_name_label);
        stat_box.append(&stat_data_label);

        $stats_grid.attach(&stat_box, $col, $row, 1, 1);

        stat_data_label
    }};
}

#[glib::object_subclass]
impl ObjectSubclass for StoxDataGrid {
    const NAME: &'static str = "StoxDataGrid";
    type Type = super::StoxDataGrid;
    type ParentType = gtk4::Box;
}

impl ObjectImpl for StoxDataGrid {
    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(Vec::new);
        &PROPERTIES
    }

    fn set_property(&self, _id: usize, _value: &Value, _pspec: &ParamSpec) {
        unimplemented!()
    }

    fn property(&self, _id: usize, _pspec: &ParamSpec) -> Value {
        unimplemented!()
    }

    fn constructed(&self) {
        self.parent_constructed();

        let obj = self.obj();
        obj.set_focusable(true);
        obj.set_visible(true);

        let grid = Grid::builder()
            .halign(Align::Center)
            .width_request(GRID_WIDTH)
            .margin_start(10)
            .margin_end(20)
            .margin_top(10)
            .margin_bottom(10)
            .build();

        let symbol_label = Label::builder()
            .valign(Align::Baseline)
            .margin_end(SYMBOL_LABEL_MARGIN_END)
            .label("--")
            .name("symbol_label")
            .selectable(true)
            .build();
        symbol_label.show();

        let name_label = Label::builder()
            .valign(Align::Baseline)
            .label("--")
            .name("name_label")
            .selectable(true)
            .build();
        name_label.show();

        let info_label = Label::builder()
            .halign(Align::Start)
            .label("--")
            .name("info_label")
            .selectable(true)
            .build();
        info_label.show();

        let latest_quote_label = Label::builder()
            .halign(Align::End)
            .label("--")
            .name("latest_quote_label")
            .selectable(true)
            .build();
        latest_quote_label.show();

        let market_change_label = Label::builder()
            .halign(Align::End)
            .label("--")
            .selectable(true)
            .build();
        market_change_label.show();

        let notebook = Notebook::builder()
            .focusable(true)
            .hexpand(true)
            .height_request(350)
            .margin_top(15)
            .build();

        grid.attach(&symbol_label, 0, 0, 1, 1);
        grid.attach(&name_label, 1, 0, 1, 1);
        grid.attach(&info_label, 0, 1, 3, 1);

        let quote_box = Box::builder()
            .spacing(0)
            .orientation(Orientation::Vertical)
            .halign(Align::End)
            .valign(Align::End)
            .hexpand(true)
            .build();
        quote_box.append(&latest_quote_label);
        quote_box.append(&market_change_label);

        grid.attach(&quote_box, 2, 0, 1, 1);
        grid.attach(&notebook, 0, 2, 3, 2);

        let stats_grid = Grid::builder()
            .margin_top(10)
            .halign(Align::Start)
            .width_request(GRID_WIDTH)
            .hexpand(false)
            .build();

        *self.open_label.borrow_mut() = stat_col!(stats_grid, gettext("Open"), 0, 0);
        *self.high_label.borrow_mut() = stat_col!(stats_grid, gettext("High"), 0, 1);
        *self.low_label.borrow_mut() = stat_col!(stats_grid, gettext("Low"), 0, 2);

        *self.volume_label.borrow_mut() = stat_col!(stats_grid, gettext("Volume"), 1, 0);
        *self.pe_ratio_label.borrow_mut() = stat_col!(stats_grid, gettext("P/E Ratio"), 1, 1);
        *self.market_cap_label.borrow_mut() = stat_col!(stats_grid, gettext("Market Cap"), 1, 2);

        *self.yield_label.borrow_mut() = stat_col!(stats_grid, gettext("Yield"), 2, 0);
        *self.beta_label.borrow_mut() = stat_col!(stats_grid, gettext("Beta"), 2, 1);
        *self.eps_label.borrow_mut() = stat_col!(stats_grid, gettext("EPS"), 2, 2);

        grid.attach(&stats_grid, 0, 6, 3, 3);

        let btns_box = Box::builder()
            .spacing(10)
            .orientation(Orientation::Horizontal)
            .margin_top(10)
            .build();

        {
            let save_btn_box = Box::new(Orientation::Horizontal, 6);

            let star_img = Image::from_icon_name("non-starred");
            save_btn_box.append(&star_img);

            let save_label = Label::new(Some(&gettext("Save")));
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

            let unsave_label = Label::new(Some(&gettext("Unsave")));
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

            let refresh_label = Label::new(Some(&gettext("Refresh")));
            refresh_btn_box.append(&refresh_label);

            let refresh_btn = Button::builder().child(&refresh_btn_box).build();
            btns_box.append(&refresh_btn);

            refresh_btn.connect_clicked(clone!(@weak self as this => move |_| {
                let symbol = this.symbol_label.borrow().label().to_string();
                if symbol == "--" {
                    return;
                }

                let save_btn = this.save_btn.borrow();
                let unsave_btn = this.unsave_btn.borrow();

                this.obj().update(
                    symbol,
                    true,
                    unsave_btn.is_visible(),
                    !save_btn.is_sensitive(),
                );
            }));

            refresh_btn.hide();

            *self.refresh_btn.borrow_mut() = refresh_btn;
        }

        grid.attach(&btns_box, 0, 9, 3, 1);

        grid.show();
        grid.set_parent(&*obj);

        *self.symbol_label.borrow_mut() = symbol_label;
        *self.name_label.borrow_mut() = name_label;
        *self.latest_quote_label.borrow_mut() = latest_quote_label;
        *self.market_change_label.borrow_mut() = market_change_label;
        *self.info_label.borrow_mut() = info_label;
        *self.notebook.borrow_mut() = notebook;
    }
}

impl BoxImpl for StoxDataGrid {}

impl WidgetImpl for StoxDataGrid {}

impl StoxDataGrid {
    pub fn construct_graph(
        &self,
        main_info: MainInfo,
        extended_info: ExtendedInfo,
        mut quotes: Vec<f64>,
    ) {
        let x_axis = stox_get_chart_x_axis(&main_info, "1d");
        if x_axis.is_err() {
            return;
        }
        let x_axis = x_axis.unwrap();

        let y_axis = stox_get_chart_y_axis(&extended_info);
        if y_axis.is_err() {
            return;
        }
        let y_axis = y_axis.unwrap();

        let drawing_area = DrawingArea::new();

        self.notebook
            .borrow_mut()
            .append_page(&drawing_area, Some(&Label::new(Some("1D"))));

        drawing_area.set_draw_func(move |_drawing_area, cr, width, height| {
            let mut x_iter = x_axis.iter();
            let mut y_iter = y_axis.iter();

            // Background color
            #[allow(clippy::eq_op)]
            cr.set_source_rgb(56.0 / 255.0, 56.0 / 255.0, 56.0 / 255.0);
            cr.paint().unwrap();
            cr.set_line_width(1.0);

            cr.set_source_rgb(0.0, 255.0, 0.0);
            if extended_info.market_change_neg() {
                cr.set_source_rgb(255.0, 0.0, 0.0);
            }

            let mut lines_step = quotes.len();
            let new_quotes = stox_scale_quotes(&mut quotes, height);
            let mut quote_iter = new_quotes.iter().rev(); // reverse or else it is reflected

            // Don't panic on a lot of quotes
            if lines_step > width as usize {
                lines_step = width as usize;
            }

            cr.move_to(0.0, *quote_iter.next().unwrap()); // start at the first point

            // Draw the actual chart and change in quote
            for i in (0..=width).step_by(width as usize / lines_step) {
                let next = quote_iter.next();

                // if we hit None, we are done
                if let Some(next) = next {
                    cr.line_to(i as f64, *next);
                    cr.line_to(i as f64, height as f64 - 4.0);
                    cr.stroke().unwrap();
                    cr.line_to(i as f64, *next);
                }
            }

            // Set the grid lines color
            #[allow(clippy::eq_op)]
            cr.set_source_rgb(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0);

            // x-axis (horizontal) lines
            for x_grid_line in (0..width).step_by(width as usize / 8) {
                cr.move_to(x_grid_line as f64, height as f64 - 5.0);
                cr.line_to(x_grid_line as f64, -height as f64);
                cr.stroke().unwrap();

                cr.move_to(x_grid_line as f64 + 2.0, height as f64 - 5.0);

                // Some text can crash on Option Instruments, catch any potential bad unwraps
                let text = x_iter.next();
                if let Some(text) = text {
                    cr.show_text(text).unwrap();
                }
            }

            // y-axis (vertical) lines
            let mut first_yaxis_item_hidden = false;
            for y_grid_line in (0..height).step_by(height as usize / 4).rev() {
                cr.move_to(0.0, y_grid_line as f64);
                cr.line_to(width as f64, y_grid_line as f64);
                cr.stroke().unwrap();

                cr.move_to(2.0, y_grid_line as f64);

                // Don't have the axis text overlap
                if first_yaxis_item_hidden {
                    // Some text can crash on Option Instruments, catch any potential bad unwraps
                    let text = y_iter.next();
                    if let Some(text) = text {
                        cr.show_text(&format!("{:.2}", text)).unwrap();
                    }
                } else {
                    first_yaxis_item_hidden = true;
                }
            }
        });

        drawing_area.show();
    }
}
