use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};

use crate::serde::MyPath;
pub fn add_to_waitlist() {}

const ORANGE: Color = Color::Rgb {
    r: 254,
    g: 138,
    b: 24,
};

/// Create terminal table with header
pub fn get_table() -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Name").add_attribute(Attribute::Bold),
            Cell::new("Tag").add_attribute(Attribute::Bold),
            Cell::new("Status").add_attribute(Attribute::Bold),
            Cell::new("Target").add_attribute(Attribute::Bold),
        ]);
    return table;
}

pub fn add_row(table: &mut Table, key: &String, tag: &String, status: &String, source: &MyPath) {
    table.add_row(vec![
        Cell::new(key).add_attribute(Attribute::Bold),
        Cell::new(tag).fg(ORANGE),
        Cell::new(status).fg(Color::Green),
        Cell::new(source)
            .add_attribute(Attribute::Italic)
            .add_attribute(Attribute::Dim),
    ]);
}
