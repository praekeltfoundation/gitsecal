pub trait RowItem {
    type CmpKey: Ord;
    type DisplayOpts;

    fn cmp_key(&self) -> Self::CmpKey;

    fn table_row(&self, opts: &Self::DisplayOpts) -> prettytable::Row;

    fn sort_vec<T: RowItem>(row_items: &mut Vec<T>) {
        row_items.sort_by(|a, b| a.cmp_key().cmp(&b.cmp_key()))
    }
}
