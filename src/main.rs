extern crate cairo;
extern crate gdk;
extern crate gtk;
extern crate pango;
extern crate pangocairo;

use std::cell::RefCell;

use cairo::Context;
use gtk::prelude::*;
use gtk::DrawingArea;
use pangocairo::CairoContextExt;

struct Point {
    x: f64,
    y: f64,
}

const TEXT: &'static str = "Hello, World!\nThis is the end!";

thread_local!(
    static GLOBAL: RefCell<Option<Point>> = RefCell::new(None);
);

fn p2c(x: i32) -> f64 {
    x as f64 / pango::SCALE as f64
}

fn c2p(x: f64) -> i32 {
    (x * pango::SCALE as f64) as i32
}

fn draw(darea: &DrawingArea, cr: &Context) -> Inhibit {
    let font = pango::FontDescription::from_string("Sans Bold 27");

    let layout = cr.create_pango_layout();
    layout.set_text(TEXT, TEXT.len() as i32);
    layout.set_font_description(Some(&font));
    let (w_p, h_p) = layout.get_size();
    let (w_c, h_c) = (p2c(w_p), p2c(h_p));
    let w_win = darea.get_allocated_width() as f64;
    let h_win = darea.get_allocated_height() as f64;
    let c_x = (w_win - w_c)/2.;
    let c_y = (h_win - h_c)/2.;

    cr.move_to(c_x, c_y);
    cr.set_source_rgb(0., 0., 0.);
    cr.update_pango_layout(&layout);

    // Handle mouse
    GLOBAL.with(|global| {
        if let Some(Point { x, y }) = global.borrow_mut().take() {
            let m_x = c2p(x - c_x);
            let m_y = c2p(y - c_y);
            let (inside, index, trailing) = layout.xy_to_index(m_x, m_y);
            println!("{}, {}i, {}t", inside, index, trailing);
        }
    });

    cr.show_pango_layout(&layout);

    Inhibit(false)
}

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }
    drawable_window(500, 500, draw);
    gtk::main();
}

pub fn drawable_window<F>(width: i32, height: i32, draw_fn: F)
where F: Fn(&DrawingArea, &Context) -> Inhibit + 'static {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    let drawing_area = Box::new(DrawingArea::new)();
    drawing_area.connect_draw(draw_fn);
    drawing_area.add_events(gdk::BUTTON_PRESS_MASK.bits() as i32);
    drawing_area.connect_button_press_event(|darea, ev| {
        let (x, y) = ev.get_position();
        GLOBAL.with(move |global| {
            *global.borrow_mut() = Some(Point { x, y })
        });
        darea.queue_draw();
        Inhibit(false)
    });

    window.set_default_size(width, height);
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
    window.add(&drawing_area);
    window.show_all();
}
