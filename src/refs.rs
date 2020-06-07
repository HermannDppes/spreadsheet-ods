use std::convert::TryFrom;
use std::fmt::{Display, Formatter};

use crate::{OdsError, ucell};

/// Parse the colname.
/// Stops when the colname ends and returns the byte position in end.
pub fn parse_colname(buf: &str, pos: &mut usize) -> Option<ucell> {
    let mut col = 0u32;

    let mut loop_break = false;
    for (p, c) in buf[*pos..].char_indices() {
        if c < 'A' || c > 'Z' {
            loop_break = true;
            *pos += p;
            break;
        }

        let mut v = c as u32 - b'A' as u32;
        if v == 25 {
            v = 0;
            col = (col + 1) * 26;
        } else {
            v += 1;
            col = col * 26;
        }
        col += v as u32;
    }
    // consumed all chars
    if !loop_break {
        *pos = buf.len();
    }

    if col == 0 {
        None
    } else {
        Some(col - 1)
    }
}

/// Parse the rowname.
/// Stops when the rowname ends and returns the byte position in end.
pub fn parse_rowname(buf: &str, pos: &mut usize) -> Option<ucell> {
    let mut row = 0u32;

    let mut loop_break = false;
    for (p, c) in buf[*pos..].char_indices() {
        if c < '0' || c > '9' {
            loop_break = true;
            *pos += p;
            break;
        }

        row *= 10;
        row += c as u32 - '0' as u32;
    }

    // consumed all chars
    if !loop_break {
        *pos = buf.len();
    }

    if row == 0 {
        None
    } else {
        Some(row - 1)
    }
}

/// Parse a cell reference.
pub fn parse_cellref(buf: &str, pos: &mut usize) -> Result<CellRef, OdsError> {
    let mut dot_idx = None;
    let mut any_quote = false;
    let mut state_quote = false;

    for (p, c) in buf[*pos..].char_indices() {
        if !state_quote {
            if c == '\'' {
                state_quote = true;
                any_quote = true;
            }
            if c == '.' {
                dot_idx = Some(*pos + p);
                break;
            }
        } else {
            if c == '\'' {
                state_quote = false;
            }
        }
    }
    if dot_idx.is_none() {
        return Err(OdsError::Ods(format!("No '.' in the cell reference {}", &buf[*pos..])));
    }
    let dot_idx = dot_idx.unwrap();

    // Tablename
    let table = if dot_idx > *pos {
        if any_quote {
            // quoting rules: enclose with ' and double contained ''
            Some(buf[*pos..dot_idx].trim_matches('\'').replace("''", "'"))
        } else {
            Some(buf[*pos..dot_idx].to_string())
        }
    } else {
        None
    };

    *pos = dot_idx + 1;
    let abs_col = buf[*pos..].starts_with('$');
    if abs_col {
        *pos += 1;
    }

    let col = parse_colname(buf, pos);
    if col.is_none() {
        return Err(OdsError::Ods(format!("No colname in the cell reference {}", &buf[*pos..])));
    }

    let abs_row = buf[*pos..].starts_with('$');
    if abs_row {
        *pos += 1;
    }

    let row = parse_rowname(buf, pos);
    if row.is_none() {
        return Err(OdsError::Ods(format!("No rowname in the cell reference {}", &buf[*pos..])));
    }

    Ok(CellRef {
        table,
        row: row.unwrap(),
        abs_row,
        col: col.unwrap(),
        abs_col,
    })
}

/// Parse a range ref.
pub fn parse_cellrange(buf: &str, pos: &mut usize) -> Result<CellRange, OdsError> {
    let ffrom = parse_cellref(buf, pos)?;

    let colon = buf[*pos..].starts_with(':');
    if !colon {
        return Err(OdsError::Ods(format!("No colon in cellrange {}", &buf[*pos..])));
    } else {
        *pos += 1;
    }

    let tto = parse_cellref(buf, pos)?;

    Ok(CellRange {
        from: ffrom,
        to: tto,
    })
}

/// Parse a list of range refs
pub fn parse_cellranges(buf: &str, pos: &mut usize) -> Result<Vec<CellRange>, OdsError> {
    let mut v = Vec::new();

    loop {
        let r = parse_cellrange(buf, pos)?;
        v.push(r);

        if *pos == buf.len() {
            break;
        }

        if !buf[*pos..].starts_with(' ') {
            return Err(OdsError::Ods(format!("No blank between cellranges {}", &buf[*pos..])));
        } else {
            *pos += 1;
        }
    }

    Ok(v)
}

/// Returns the spreadsheet column name.
pub fn push_colname(buf: &mut String, mut col: ucell) {
    let mut i = 0;
    let mut dbuf = [0u8; 7];

    col += 1;
    while col > 0 {
        dbuf[i] = (col % 26) as u8;
        if dbuf[i] == 0 {
            dbuf[i] = 25;
            col = col / 26 - 1;
        } else {
            dbuf[i] -= 1;
            col /= 26;
        }

        i += 1;
    }

    // reverse order
    let mut j = i;
    while j > 0 {
        buf.push((b'A' + dbuf[j - 1]) as char);
        j -= 1;
    }
}

/// Returns the spreadsheet row name
pub fn push_rowname(buf: &mut String, mut row: ucell) {
    let mut i = 0;
    let mut dbuf = [0u8; 10];

    row += 1;
    while row > 0 {
        dbuf[i] = (row % 10) as u8;
        row /= 10;

        i += 1;
    }

    // reverse order
    let mut j = i;
    while j > 0 {
        buf.push((b'0' + dbuf[j - 1]) as char);
        j -= 1;
    }
}

/// Returns the spreadsheet column name.
pub fn colname(col: ucell) -> String {
    let mut col_str = String::new();
    push_colname(&mut col_str, col);
    col_str
}

/// Returns the spreadsheet row name
pub fn rowname(row: ucell) -> String {
    let mut row_str = String::new();
    push_rowname(&mut row_str, row);
    row_str
}

/// Returns a list of ranges as string.
pub fn cellranges_to_string(v: &Vec<CellRange>) -> String {
    let mut buf = String::new();

    let mut first = true;
    for r in v {
        if first {
            first = false;
        } else {
            buf.push(' ');
        }
        buf.push_str(&r.to_string());
    }

    buf
}


/// Reference to a cell.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CellRef {
    // Tablename
    pub table: Option<String>,
    // Row
    pub row: ucell,
    // Column
    pub col: ucell,
    // Absolute ($) reference
    pub abs_row: bool,
    // Absolute ($) reference
    pub abs_col: bool,
}

impl TryFrom<&str> for CellRef {
    type Error = OdsError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut pos = 0usize;
        parse_cellref(s, &mut pos)
    }
}

impl Display for CellRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        if let Some(table) = &self.table {
            write!(f, "{}", table)?;
        }
        write!(f, ".")?;
        if self.abs_row {
            write!(f, "$")?;
        }
        write!(f, "R{}", self.row)?;
        if self.abs_col {
            write!(f, "$")?;
        }
        write!(f, "C{}", self.col)?;
        Ok(())
    }
}

impl CellRef {
    pub fn simple(row: ucell, col: ucell) -> Self {
        Self {
            table: None,
            row,
            abs_row: false,
            col,
            abs_col: false,
        }
    }

    pub fn table<S: Into<String>>(table: S, row: ucell, col: ucell) -> Self {
        Self {
            table: Some(table.into()),
            row,
            abs_row: false,
            col,
            abs_col: false,
        }
    }

    /// Returns the spreadsheet column name.
    pub fn colname(&self) -> String {
        colname(self.col)
    }

    /// Returns the spreadsheet row name.
    pub fn rowname(&self) -> String {
        rowname(self.row)
    }

    /// Returns a cell reference.
    pub fn to_string(&self) -> String {
        let mut refstr = String::new();
        if let Some(table) = &self.table {
            refstr.push_str(table);
        }
        refstr.push('.');
        if self.abs_col {
            refstr.push('$');
        }
        push_colname(&mut refstr, self.col);
        if self.abs_row {
            refstr.push('$');
        }
        push_rowname(&mut refstr, self.row);

        refstr
    }

    /// Returns a cell reference for a formula.
    pub fn to_formula(&self) -> String {
        let mut refstr = String::new();
        refstr.push('[');
        if let Some(table) = &self.table {
            refstr.push_str(table);
        }
        refstr.push('.');
        if self.abs_col {
            refstr.push('$');
        }
        push_colname(&mut refstr, self.col);
        if self.abs_row {
            refstr.push('$');
        }
        push_rowname(&mut refstr, self.row);
        refstr.push(']');

        refstr
    }
}

/// A cell-range.
/// As usual for a spreadsheet this is meant as inclusive from and to.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CellRange {
    pub from: CellRef,
    pub to: CellRef,
}

impl CellRange {
    /// Creates the cell range from from + to data.
    pub fn simple(row: ucell, col: ucell, row_to: ucell, col_to: ucell) -> Self {
        assert!(row <= row_to);
        assert!(col <= col_to);
        Self {
            from: CellRef::simple(row, col),
            to: CellRef::simple(row_to, col_to),
        }
    }

    /// Creates the cell range from from + to data.
    pub fn table<S: Into<String>>(table: S, row: ucell, col: ucell, row_to: ucell, col_to: ucell) -> Self {
        assert!(row <= row_to);
        assert!(col <= col_to);
        let table = table.into();
        Self {
            from: CellRef::table(table.to_string(), row, col),
            to: CellRef::table(table.to_string(), row_to, col_to),
        }
    }

    /// Creates the cell range from origin + spanning data.
    pub fn origin_span(row: ucell, col: ucell, span: (ucell, ucell)) -> Self {
        assert!(span.0 > 0);
        assert!(span.1 > 0);
        Self {
            from: CellRef::simple(row, col),
            to: CellRef::simple(row + span.0 - 1, col + span.1 - 1),
        }
    }

    /// Returns a range reference.
    pub fn to_string(&self) -> String {
        let mut refstr = String::new();
        refstr.push_str(&self.from.to_string());
        refstr.push(':');
        refstr.push_str(&self.to.to_string());
        refstr
    }

    /// Returns a range reference for a formula.
    pub fn to_formula(&self) -> String {
        let mut refstr = String::new();
        refstr.push('[');
        refstr.push_str(&self.from.to_string());
        refstr.push(':');
        refstr.push_str(&self.to.to_string());
        refstr.push(']');
        refstr
    }

    /// Does the range contain the cell.
    pub fn contains(&self, row: ucell, col: ucell) -> bool {
        row >= self.from.row && row <= self.to.row
            && col >= self.from.col && col <= self.to.col
    }

    /// Is this range any longer relevant, when looping rows first, then columns?
    pub fn out_looped(&self, row: ucell, col: ucell) -> bool {
        row > self.to.row
            || row == self.to.row && col > self.to.col
    }
}

impl TryFrom<&str> for CellRange {
    type Error = OdsError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut pos = 0usize;
        parse_cellrange(value, &mut pos)
    }
}

impl Display for CellRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{}:{}", &self.from, &self.to)?;
        Ok(())
    }
}

/// A range over columns.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ColRange {
    pub from: ucell,
    pub to: ucell,
}

impl ColRange {
    pub fn new(from: ucell, to: ucell) -> Self {
        assert!(from <= to);
        Self {
            from,
            to,
        }
    }

    pub fn contains(&self, col: ucell) -> bool {
        col >= self.from && col <= self.to
    }
}

/// A range over rows.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RowRange {
    pub from: ucell,
    pub to: ucell,
}

impl RowRange {
    pub fn new(from: ucell, to: ucell) -> Self {
        assert!(from <= to);
        Self {
            from,
            to,
        }
    }

    pub fn contains(&self, row: ucell) -> bool {
        row >= self.from && row <= self.to
    }
}