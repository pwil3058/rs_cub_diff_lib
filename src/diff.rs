//Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
//
//Licensed under the Apache License, Version 2.0 (the "License");
//you may not use this file except in compliance with the License.
//You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
//Unless required by applicable law or agreed to in writing, software
//distributed under the License is distributed on an "AS IS" BASIS,
//WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//See the License for the specific language governing permissions and
//limitations under the License.

use std::cell::RefCell;
use std::rc::Rc;

use lazy_static;
use regex::Regex;

use gtk;
use gtk::prelude::*;

use cub_diff_lib::diff::DiffPlus;
use cub_diff_lib::lines::*;

use pw_gix::wrapper::*;

pub struct TwsLineCountDisplay {
    h_box: gtk::Box,
    entry: gtk::Entry,
}

impl_widget_wrapper!(h_box: gtk::Box, TwsLineCountDisplay);

impl TwsLineCountDisplay {
    pub fn new() -> Rc<Self> {
        let h_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        h_box.pack_start(&gtk::Label::new("Added TWS lines:"), false, false, 0);
        let entry = gtk::Entry::new();
        entry.set_width_chars(1);
        entry.set_text("0");
        entry.set_editable(false);
        h_box.pack_start(&entry, false, false, 0);
        h_box.show_all();
        let twslcd = Rc::new(Self{ h_box, entry });
        twslcd.set_value(0);

        twslcd
    }

    pub fn set_value(&self, val: usize) {
        let sval = val.to_string();
        self.entry.set_width_chars(sval.len() as i32);
        self.entry.set_text(&sval);
        // TODO: set TWS background colours
        //if val > 0 {
        //    set background red ("#FF0000")
        //} else {
        //    set background green ("#00FF00")
        //}
    }
}

lazy_static! {
    static ref TWS_CHECK_CRE: Regex = Regex::new(r"^([\+!].*\S)(\s+\n?)$").unwrap();
}

enum MarkupType {
    Header,
    Before,
    After,
    Added,
    Removed,
    Changed,
    Unchanged,
    AddedTWS,
    Stats,
    Separator,
    ContextAid,
}

macro_rules! markup_as {
    ( $mut:expr, $text:expr ) => {
        match $mut {
            MarkupType::Header => format!("<scan weight=Pango::Weight::BOLD, foreground=\"#0000AA\", family=\"monospace\">{}</scan>", $text),
            MarkupType::Before => format!("<scan foreground=\"#AA0000\", family=\"monospace\">{}</scan>", $text),
            MarkupType::After => format!("<scan foreground=\"#008800\", family=\"monospace\">{}</scan>", $text),
            MarkupType::Removed => format!("<scan foreground=\"#AA0000\", family=\"monospace\">{}</scan>", $text),
            MarkupType::Added => format!("<scan foreground=\"#008800\", family=\"monospace\">{}</scan>", $text),
            MarkupType::Changed => format!("<scan foreground=\"#AA6600\", family=\"monospace\">{}</scan>", $text),
            MarkupType::Unchanged => format!("<scan foreground=\"#000000\", family=\"monospace\">{}</scan>", $text),
            MarkupType::AddedTWS => format!("<scan background=\"#008800\", family=\"monospace\">{}</scan>", $text),
            MarkupType::Stats => format!("<scan foreground=\"#AA00AA\", family=\"monospace\">{}</scan>", $text),
            MarkupType::Separator => format!("<scan weight=Pango::Weight::BOLD, foreground=\"#0000AA\", family=\"monospace\">{}</scan>", $text),
            MarkupType::ContextAid => format!("<scan foreground=\"#00AAAA\", family=\"monospace\">{}</scan>", $text),
        }
    }
}

        //self.index_tag = self.create_tag("INDEX", weight=Pango.Weight.BOLD, foreground="#0000AA", family="monospace")
        //self.sep_tag = self.create_tag("SEP", weight=Pango.Weight.BOLD, foreground="#0000AA", family="monospace")
        //self.minus_tag = self.create_tag("MINUS", foreground="#AA0000", family="monospace")
        //self.lab_tag = self.create_tag("LAB", foreground="#AA0000", family="monospace")
        //self.plus_tag = self.create_tag("PLUS", foreground="#006600", family="monospace")
        //self.added_tws_tag = self.create_tag("ADDED_TWS", background="#006600", family="monospace")
        //self.star_tag = self.create_tag("STAR", foreground="#006600", family="monospace")
        //self.rab_tag = self.create_tag("RAB", foreground="#006600", family="monospace")
        //self.change_tag = self.create_tag("CHANGED", foreground="#AA6600", family="monospace")
        //self.stats_tag = self.create_tag("STATS", foreground="#AA00AA", family="monospace")
        //self.func_tag = self.create_tag("FUNC", foreground="#00AAAA", family="monospace")
        //self.unchanged_tag = self.create_tag("UNCHANGED", foreground="black", family="monospace")


pub trait DiffPlusTextBuffer: gtk::TextBufferExt {
    fn append_markup(&mut self, markup: &str) {
        self.insert_markup(&mut self.get_end_iter(), markup);
    }

    fn append_diff_plus_line(&mut self, line: &str) -> usize {
        use MarkupType::*;
        let mut is_context_diff = false;
        if line.starts_with(" ") {
            self.append_markup(&markup_as!(Unchanged, line));
        } else if line.starts_with("+") {
            if let Some(captures) = TWS_CHECK_CRE.captures(line) {
                let text = captures.get(1).unwrap().as_str();
                self.append_markup(&markup_as!(Added, line));
                self.append_markup(&markup_as!(AddedTWS, captures.get(2).unwrap().as_str()));
                return text.len()
            } else {
                self.append_markup(&markup_as!(Added, line));
            }
        } else if line.starts_with("---") {
            self.append_markup(&markup_as!(Removed, line));
        } else if line.starts_with("-") {
            self.append_markup(&markup_as!(Removed, line));
        } else if line.starts_with("!") {
            if let Some(captures) = TWS_CHECK_CRE.captures(line) {
                let text = captures.get(1).unwrap().as_str();
                self.append_markup(&markup_as!(Changed, line));
                self.append_markup(&markup_as!(AddedTWS, captures.get(2).unwrap().as_str()));
                return text.len()
            } else {
                self.append_markup(&markup_as!(Changed, line));
            }
        } else if line.starts_with("@") {
            if let Some(i) = line.rfind("@@") {
                self.append_markup(&markup_as!(Stats, &line[..i + 2]));
                self.append_markup(&markup_as!(ContextAid, &line[i + 2..]));
            } else {
                self.append_markup(&markup_as!(Stats, line));
            }
        } else if line.starts_with("****") {
            self.append_markup(&markup_as!(Separator, line));
        } else if line.starts_with("***") {
            self.append_markup(&markup_as!(Separator, line));
            //self.append_markup(line, self.sep_tag)
        //} else if line.starts_with("*") {
            //self.append_markup(line, self.star_tag)
        //} else {
            //self.append_markup(line, self.index_tag)
        }
        0
    }
}

impl DiffPlusTextBuffer for gtk::TextBuffer {}

pub struct DiffPlusNotebook {
    notebook: gtk::Notebook,
    lines: RefCell<Lines>,
}

impl_widget_wrapper!(notebook: gtk::Notebook, DiffPlusNotebook);

impl DiffPlusNotebook {
    pub fn new() -> Rc<Self> {
        Rc::new(Self{
            notebook: gtk::Notebook::new(),
            lines: RefCell::new(vec![]),
        })
    }

    pub fn update(&self, _diff_pluses: &Vec<DiffPlus>) {
    }

    pub fn repopulate(&self, _diff_pluses: &Vec<DiffPlus>) {
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
