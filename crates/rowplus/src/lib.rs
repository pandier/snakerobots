use sqlx::{Row, postgres::PgRow};

#[macro_export]
macro_rules! query_plus {
    ($row:ty, $fmt:literal) => {
        sqlx::query_as::<_, $row>(&rowplus::sql_plus!($row, $fmt))
    };
}

#[macro_export]
macro_rules! sql_plus {
    ($row:ty, $fmt:literal) => {
        {
            use rowplus::RowPlus;
            format!($fmt, <$row>::columns())
        }
    };
}

pub trait RowPlus: Sized {
    fn columns() -> RowPlusColumns;
    fn columns_under(table: &str, root: bool) -> RowPlusColumns;
    fn from_row_under(row: &PgRow, table: &str, root: bool) -> Result<Self, sqlx::Error>;
}

pub trait RowPlusNested: Sized {
    fn columns_nested(table: &str) -> RowPlusColumns;
    fn from_row_nested(row: &PgRow, table: &str) -> Result<Self, sqlx::Error>;
}

impl<T: RowPlus> RowPlusNested for T {
    fn columns_nested(table: &str) -> RowPlusColumns {
        Self::columns_under(table, false)
    }

    fn from_row_nested(row: &PgRow, table: &str) -> Result<Self, sqlx::Error> {
        Self::from_row_under(row, table, false)
    }
}

impl<T: RowPlus> RowPlusNested for Option<T> {
    fn columns_nested(table: &str) -> RowPlusColumns {
        let mut cols = RowPlusColumns::new();
        // .id is a "temporary" fix...
        cols.add_raw(&format!("\"{}\".id IS NOT NULL", table), (String::from(table) + "$Option").as_str());
        cols.append(T::columns_nested(table));
        cols
    }

    fn from_row_nested(row: &PgRow, table: &str) -> Result<Self, sqlx::Error> {
        let is_not_null: bool = row.try_get((String::from(table) + "$Option").as_str())?;
        if is_not_null {
            let value = T::from_row_nested(row, table)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone)]
pub struct RowPlusColumns(pub Vec<String>);

impl RowPlusColumns {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, table: &str, name: &str, alias: &str) {
        let col = format!("\"{}\".\"{}\" AS \"{}\"", table, name, alias);
        self.0.push(col);
    }

    pub fn add_raw(&mut self, value: &str, alias: &str) {
        let col = format!("{} AS \"{}\"", value, alias);
        self.0.push(col);
    }

    pub fn nest<T>(&mut self, new_table: &str) where T: RowPlusNested {
        let cols = T::columns_nested(&new_table);
        self.append(cols);
    }

    pub fn flat<T>(&mut self, table: &str, root: bool) where T: RowPlus {
        let child = T::columns_under(table, root);
        self.append(child);
    }

    pub fn append(&mut self, mut other: RowPlusColumns) {
        self.0.append(&mut other.0);
    }
}

impl std::fmt::Display for RowPlusColumns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, col) in self.0.iter().enumerate() {
            write!(f, "{}", col)?;
            if i < self.0.len() - 1 {
                write!(f, ", ")?;
            }
        }
        Ok(())
    }
}
