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
use std::collections::HashMap;
use std::rc::Rc;

use lazy_static;
use regex::Regex;

use glib;
use gtk;
use gtk::prelude::*;

use cub_diff_lib::diff::{Diff, DiffPlus};
use cub_diff_lib::lines::*;
use cub_diff_lib::preamble::*;
use cub_diff_lib::text_diff::TextDiffHunk;
use cub_diff_lib::unified_diff::UnifiedDiff;

use pw_gix::wrapper::*;

use crate::icons;

pub struct TwsCountDisplay {
    h_box: gtk::Box,
    entry: gtk::Entry,
}

impl TwsCountDisplay {
    pub fn new(label: &str) -> Rc<Self> {
        let h_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        h_box.pack_start(&gtk::Label::new(label), false, false, 0);
        let entry = gtk::Entry::new();
        entry.set_width_chars(1);
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

impl_widget_wrapper!(h_box: gtk::Box, TwsCountDisplay);

lazy_static! {
    static ref TWS_CHECK_CRE: Regex = Regex::new(r"^([\+!].*\S)(\s+\n?)$").unwrap();
}

enum MarkupType {
    Header,
    Ante,
    Post,
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
    ( $mut:expr, $text:expr ) => {{
        let e_text = glib::markup_escape_text($text);
        match $mut {
            MarkupType::Header => format!("<span weight=\"bold\" foreground=\"#0000AA\" face=\"monospace\">{}</span>", e_text),
            MarkupType::Ante => format!("<span foreground=\"#AA0000\" face=\"monospace\">{}</span>", e_text),
            MarkupType::Post => format!("<span foreground=\"#008800\" face=\"monospace\">{}</span>", e_text),
            MarkupType::Removed => format!("<span foreground=\"#AA0000\" face=\"monospace\">{}</span>", e_text),
            MarkupType::Added => format!("<span foreground=\"#008800\" face=\"monospace\">{}</span>", e_text),
            MarkupType::Changed => format!("<span foreground=\"#AA6600\" face=\"monospace\">{}</span>", e_text),
            MarkupType::Unchanged => format!("<span foreground=\"#000000\" face=\"monospace\">{}</span>", e_text),
            MarkupType::AddedTWS => format!("<span background=\"#008800\" face=\"monospace\">{}</span>", e_text),
            MarkupType::Stats => format!("<span foreground=\"#AA00AA\" face=\"monospace\">{}</span>", e_text),
            MarkupType::Separator => format!("<span weight=\"bold\" foreground=\"#0000AA\" face=\"monospace\">{}</span>", e_text),
            MarkupType::ContextAid => format!("<span foreground=\"#00AAAA\" face=\"monospace\">{}</span>", e_text),
        }
    }}
}

        //self.index_tag = self.create_tag("INDEX", weight=Pango.Weight.BOLD, foreground="#0000AA", face="monospace")
        //self.sep_tag = self.create_tag("SEP", weight=Pango.Weight.BOLD, foreground="#0000AA", face="monospace")
        //self.minus_tag = self.create_tag("MINUS", foreground="#AA0000", face="monospace")
        //self.lab_tag = self.create_tag("LAB", foreground="#AA0000", face="monospace")
        //self.plus_tag = self.create_tag("PLUS", foreground="#006600", face="monospace")
        //self.added_tws_tag = self.create_tag("ADDED_TWS", background="#006600", face="monospace")
        //self.star_tag = self.create_tag("STAR", foreground="#006600", face="monospace")
        //self.rab_tag = self.create_tag("RAB", foreground="#006600", face="monospace")
        //self.change_tag = self.create_tag("CHANGED", foreground="#AA6600", face="monospace")
        //self.stats_tag = self.create_tag("STATS", foreground="#AA00AA", face="monospace")
        //self.func_tag = self.create_tag("FUNC", foreground="#00AAAA", face="monospace")
        //self.unchanged_tag = self.create_tag("UNCHANGED", foreground="black", face="monospace")


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

    fn append_preamble(&mut self, preamble: &Preamble) {
        for line in preamble.iter() {
            let markup = markup_as!(MarkupType::Header, line);
            self.append_markup(&markup);
        }
    }

    fn append_git_preamble(&mut self, preamble: &GitPreamble) {
        for line in preamble.iter() {
            let markup = markup_as!(MarkupType::Header, line);
            self.append_markup(&markup);
        }
    }

    fn append_unified_diff(&mut self, unified_diff: &UnifiedDiff) {
        let ante_header_line = &unified_diff.header().lines[0];
        self.append_markup(&markup_as!(MarkupType::Ante, ante_header_line));
        let post_header_line = &unified_diff.header().lines[1];
        self.append_markup(&markup_as!(MarkupType::Post, post_header_line));
        for hunk in unified_diff.hunks().iter() {
            let mut iter = hunk.iter();
            let first_line = iter.next().unwrap();
            let i = first_line[2..].find("@@").unwrap();
            self.append_markup(&markup_as!(MarkupType::Stats, &first_line[..i + 4]));
            self.append_markup(&markup_as!(MarkupType::ContextAid, &first_line[i + 4..]));
        }
    }
}

impl DiffPlusTextBuffer for gtk::TextBuffer {}

pub struct DiffPlusDisplay {
    v_box: gtk::Box,
    text_view: gtk::TextView,
}

impl_widget_wrapper!(v_box: gtk::Box, DiffPlusDisplay);

impl DiffPlusDisplay {
    pub fn new(diff_plus: &Arc<DiffPlus>) -> Rc<Self> {
        let dpp = Rc::new(Self {
            v_box: gtk::Box::new(gtk::Orientation::Vertical, 0),
            text_view: gtk::TextView::new(),
        });
        let nadj: Option<&gtk::Adjustment> = None;
        let sw = gtk::ScrolledWindow::new(nadj, nadj);
        sw.add(&dpp.text_view);
        dpp.v_box.pack_start(&sw, true, true, 0);
        let mut buffer: gtk::TextBuffer = dpp.text_view.get_buffer().unwrap();
        if let Some(preamble) = diff_plus.preamble() {
            buffer.append_preamble(&preamble);
        }
        match diff_plus.diff() {
            Diff::Unified(unified_diff) => buffer.append_unified_diff(&unified_diff),
            Diff::Context(context_diff) => buffer.append_markup("<b>not yet implemented</b>"),
            Diff::GitBinary(git_binary_diff) => buffer.append_markup("<b>not yet implemented</b>"),
            Diff::GitPreambleOnly(git_preamble) => buffer.append_git_preamble(&git_preamble),
        }

        dpp.v_box.show_all();

        dpp
    }
}

pub struct DiffPlusNotebook {
    notebook: gtk::Notebook,
    tws_count_display: Rc<TwsCountDisplay>,
    diff_plus_displays: RefCell<HashMap<String, Rc<DiffPlusDisplay>>>,
    strip_level: usize,
}

impl_widget_wrapper!(notebook: gtk::Notebook, DiffPlusNotebook);

fn make_file_label(file_path: &str, adds_tws: bool) -> gtk::Box {
    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let image = if adds_tws {
        icons::bad_file_image(24)
    } else {
        icons::good_file_image(24)
    };
    hbox.pack_start(&image, false, true, 0);
    let label = gtk::Label::new(file_path);
    hbox.pack_start(&label, true, true, 4);
    hbox.show_all();
    hbox
}

impl DiffPlusNotebook {
    pub fn new(strip_level: usize) -> Rc<Self> {
        let dpn = Rc::new(Self{
            notebook: gtk::Notebook::new(),
            tws_count_display: TwsCountDisplay::new("#Files adding TWS:"),
            diff_plus_displays: RefCell::new(HashMap::<String, Rc<DiffPlusDisplay>>::new()),
            strip_level
        });
        dpn.notebook.popup_enable();

        dpn
    }

    pub fn tws_count_display(&self) -> &Rc<TwsCountDisplay> {
        &self.tws_count_display
    }

    pub fn update(&self, _diff_pluses: &Vec<Arc<DiffPlus>>) {
    }

    pub fn repopulate(&self, diff_pluses: &Vec<Arc<DiffPlus>>) {
        use gtk::NotebookExtManual;
        // Clear all existing data/pages
        for child in self.notebook.get_children().iter() {
            self.notebook.remove(child);
        }
        self.diff_plus_displays.borrow_mut().clear();
        // Now create the new pages
        let mut added_tws_count = 0;
        for diff_plus in diff_pluses.iter() {
            let file_path = diff_plus.get_file_path(self.strip_level);
            let adds_tws = diff_plus.adds_trailing_white_space();
            if adds_tws {
                added_tws_count += 1;
            }
            let tab_label = make_file_label(&file_path, adds_tws);
            let menu_label = make_file_label(&file_path, adds_tws);
            let diff_plus_display = DiffPlusDisplay::new(&diff_plus);
            self.notebook.append_page_menu(&diff_plus_display.pwo(), Some(&tab_label), Some(&menu_label));
            self.diff_plus_displays.borrow_mut().insert(file_path, diff_plus_display);
        }
        self.notebook.show_all();
        self.tws_count_display.set_value(added_tws_count);
    }
}
