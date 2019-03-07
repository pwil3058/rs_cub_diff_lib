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

use std::rc::Rc;

use gtk;
use gtk::prelude::*;

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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
