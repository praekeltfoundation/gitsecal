use std::collections::HashMap;

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

    pub fn append_line(&mut self, field: impl ToString, line: impl ToString) {
        // FIXME: This should probably explode or error if we have a
        // CellItem::Line instead.
        let cell = self.cells.entry(field.to_string()).or_insert(CellItem::Lines(vec![]));
        if let CellItem::Lines(lines) = cell {
            lines.push(line.to_string());
        }
    }
}


#[derive(Debug, Default, Clone)]
pub struct Content {
    pub columns: Vec<String>,
    pub rows: Vec<RowItem>,
}
