use std::collections::HashMap;

use failure::{format_err, Fallible};

#[derive(Debug, Clone)]
pub enum CellItem {
    Line(String),
    Lines(Vec<String>),
}


impl CellItem {
    pub fn text(&self, multiline: bool) -> String {
        let joinstr = if multiline { "\n" } else { ", " };
        match self {
            CellItem::Line(line) => line.clone(),
            CellItem::Lines(lines) => lines.join(joinstr),
        }
    }
}


#[derive(Debug, Default, Clone)]
pub struct RowItem {
    pub cells: HashMap<String, CellItem>,
}


impl RowItem {
    pub fn add_line(&mut self, field: impl ToString, line: impl ToString) {
        self.cells.insert(field.to_string(), CellItem::Line(line.to_string()));
    }

    pub fn add_lines(&mut self, field: impl ToString, lines: Vec<impl ToString>) {
        let lines = lines.iter().map(|l| l.to_string()).collect();
        self.cells.insert(field.to_string(), CellItem::Lines(lines));
    }

    pub fn append_line(&mut self, field: impl ToString, line: impl ToString)  -> Fallible<()> {
        let cell = self.cells.entry(field.to_string()).or_insert_with(|| CellItem::Lines(vec![]));
        if let CellItem::Lines(lines) = cell {
            lines.push(line.to_string());
            Ok(())
        } else {
            Err(format_err!("can only append to a multiline cell"))
        }
    }

    pub fn cmp_key(&self, field: &str) -> String {
        let empty = "".to_owned();
        match self.cells.get(field) {
            None => empty,
            Some(CellItem::Lines(lines)) => lines.first().cloned().unwrap_or(empty),
            Some(CellItem::Line(line)) => line.clone(),
        }
    }
}


#[derive(Debug, Default, Clone)]
pub struct Content {
    pub columns: Vec<String>,
    pub rows: Vec<RowItem>,
}


impl Content {
    pub fn sort_on(&mut self, field: &str) {
        self.rows.sort_by_key(|row| row.cmp_key(field));
    }
}
