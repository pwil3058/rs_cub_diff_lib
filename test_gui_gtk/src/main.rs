// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use pw_gix::gtk::{self, prelude::*};

use cub_diff_lib_gtk::{
    cub_diff_lib::{
        diff::DiffPlusParser,
        lines::{Lines, LinesIfce},
    },
    diff::DiffPlusNotebook,
    pw_gix::wrapper::*,
};

fn main() {
    gtk::init().expect("nowhere to go if Gtk++ initialization fails");
    let win = gtk::Window::new(gtk::WindowType::Toplevel);
    let diff_notebook = DiffPlusNotebook::new(1);
    let file = std::fs::File::open("./test_diffs/test_1.diff").unwrap();
    let lines = Lines::read(&file).unwrap();
    let diff_plus_parser = DiffPlusParser::new();
    let diff_pluses = diff_plus_parser.parse_lines(&lines).unwrap();
    diff_notebook.repopulate(&diff_pluses);
    diff_notebook.pwo().show_all();
    win.add(&diff_notebook.pwo());
    win.connect_destroy(|_| gtk::main_quit());
    win.show();
    gtk::main()
}
