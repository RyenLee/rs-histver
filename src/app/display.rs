use comfy_table::{Cell, Table};

use crate::domain::RustRelease;

/// Render releases as a formatted table
pub(super) fn print_releases_table(releases: &[RustRelease], limit: usize) {
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("#"),
        Cell::new("Version"),
        Cell::new("Date"),
        Cell::new("Channel"),
    ]);

    for (i, r) in releases.iter().take(limit).enumerate() {
        table.add_row(vec![
            Cell::new(i + 1),
            Cell::new(&r.version),
            Cell::new(&r.date),
            Cell::new(&r.channel),
        ]);
    }

    println!("{table}");
}
