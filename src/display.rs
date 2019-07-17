use failure::Fallible;

use prettytable::{Attr, Cell, Row};

use crate::common::{Content, RowItem};

pub trait DisplayOpts {
    fn common_opts(&self) -> CommonOpts;

    fn joinstrs(&self, lines: &[String]) -> String {
        lines.join(if self.common_opts().multiline { "\n" } else { ", " })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CommonOpts {
    pub multiline: bool,
    pub borders: bool,
    pub csv: bool,
}

fn mkheader(columns: &[String]) -> Row {
    let mut row = Row::empty();
    for col in columns {
        row.add_cell(Cell::new(col).with_style(Attr::Bold));
    }
    row
}

fn mkrow(columns: &[String], item: &RowItem, multiline: bool) -> Row {
    let mut row = Row::empty();
    for col in columns {
        let text = match item.cells.get(col) {
            None => "".to_owned(),
            Some(cellitem) => cellitem.text(multiline),
        };
        row.add_cell(Cell::new(&text));
    }
    row
}

pub fn printstd(content: Content, opts: CommonOpts) -> Fallible<()> {
    let mut table = prettytable::Table::new();
    table.add_row(mkheader(&content.columns));
    for item in &content.rows {
        table.add_row(mkrow(&content.columns, item, opts.multiline));
    }

    if !opts.borders {
        table.set_format(*prettytable::format::consts::FORMAT_CLEAN);
    }

    if opts.csv {
        table.to_csv(std::io::stdout())?;
    } else {
        table.printstd();
    }
    Ok(())
}
