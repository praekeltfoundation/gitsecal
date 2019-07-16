pub trait RowItem {
    type CmpKey: Ord;
    type DisplayOpts: DisplayOpts;

    fn cmp_key(&self) -> Self::CmpKey;

    fn table_row(&self, opts: &Self::DisplayOpts) -> prettytable::Row;

    fn sort_vec<T: RowItem>(row_items: &mut Vec<T>) {
        row_items.sort_by(|a, b| a.cmp_key().cmp(&b.cmp_key()))
    }
}

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
