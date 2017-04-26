extern crate cairo;
extern crate gdk;
extern crate gtk;
extern crate pango;
extern crate pangocairo;

use std::cell::RefCell;
use std::env;
use std::io::Read;
use std::fs::File;

use cairo::Context;
use gtk::prelude::*;
use gtk::DrawingArea;
use pangocairo::CairoContextExt;

struct Point {
    x: f64,
    y: f64,
}

struct Attributes {
    text: String,
    mouse: Option<Point>,
    //cursor: usize,
}

thread_local!(
    static GLOBAL: RefCell<Option<Attributes>> = RefCell::new(None);
);

fn p2c(x: i32) -> f64 {
    x as f64 / pango::SCALE as f64
}

fn c2p(x: f64) -> i32 {
    (x * pango::SCALE as f64) as i32
}

fn draw(darea: &DrawingArea, cr: &Context) -> Inhibit {
    let font = pango::FontDescription::from_string("Sans Bold 16");

    let layout = cr.create_pango_layout();
    let mut text_out = String::from("NO");
    let mut mouse_out = None;

    GLOBAL.with(|global| {
        if let Some(Attributes { ref text, ref mut mouse }) = *global.borrow_mut() {
            mouse_out = mouse.take();
            text_out = String::from(&**text);
        }
    });

    layout.set_text(text_out.as_ref(), text_out.len() as i32);
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

    cr.show_pango_layout(&layout);

    // Handle mouse
    if let Some(Point { x, y }) = mouse_out {
        let m_x = c2p(x - c_x);
        let m_y = c2p(y - c_y);
        let (inside, index, trailing) = layout.xy_to_index(m_x, m_y);
        println!("{}, {}i, {}t", inside, index, trailing);
        let pango::Rectangle { x, y, width, height } = layout.index_to_pos(index);
        cr.rectangle(p2c(x) + c_x, p2c(y) + c_y, p2c(width), p2c(height));
        cr.set_line_width(1.0);
        cr.stroke();
    }

    Inhibit(false)
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <text-file>", args[0]);
        return;
    }

    let text = {
        let mut file = File::open(&args[1]).unwrap();
        let mut text = String::new();
        file.read_to_string(&mut text).unwrap();
        text
    };

    GLOBAL.with(move |global| {
        *global.borrow_mut() = Some(Attributes {
            text: text,
            mouse: None,
        })
    });

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
    let drawing_area = DrawingArea::new();
    drawing_area.connect_draw(draw_fn);
    drawing_area.add_events(gdk::BUTTON_PRESS_MASK.bits() as i32);
    drawing_area.connect_button_press_event(|darea, ev| {
        let (x, y) = ev.get_position();
        GLOBAL.with(move |global| {
            global.borrow_mut().as_mut().unwrap().mouse = Some(Point { x, y })
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
