use crate::ucell;
use crate::refs::{CellRange, CellRef};

/// Creates a cell-reference for use in formulas.
pub fn fcellref(row: ucell, col: ucell) -> String {
    CellRef::simple(row, col).to_formula()
}

/// Creates a cell-reference for use in formulas.
pub fn fcellref_table<S: Into<String>>(table: S, row: ucell, col: ucell) -> String {
    CellRef::table(table, row, col).to_formula()
}

/// Creates a cell-reference for use in formulas.
pub fn frangeref(row: ucell,
                 col: ucell,
                 row_to: ucell,
                 col_to: ucell) -> String {
    CellRange::simple(row, col, row_to, col_to).to_formula()
}

/// Creates a cell-reference for use in formulas.
pub fn frangeref_table<S: Into<String>>(table: S,
                                        row: ucell,
                                        col: ucell,
                                        row_to: ucell,
                                        col_to: ucell) -> String {
    CellRange::table(table, row, col, row_to, col_to).to_formula()
}