extern crate rusqlite;

mod migrations;

use std::path::Path;
use rusqlite::{Connection, Error, NO_PARAMS};
use rusqlite::types::{ToSql, ToSqlOutput, Value};
use crate::data::{File, Rating};
use crate::support::ErrToString;

#[derive(Debug)]
pub struct PersistenceManager {
    conn: Connection,
}

// lifetime
impl PersistenceManager {
    pub fn open_dir(dir: &Path) -> Result<Self, String> {
        let db_file = dir.join("aspect.sqlite");

        migrations::migrate(&db_file)?;
        let conn = Connection::open(db_file).err_to_string()?;

        Ok(PersistenceManager {
            conn
        })
    }

    pub fn close(self) -> Result<(), String> {
        self.conn.close().map_err(|(_, e)| format!("{}", e))
    }
}

// file updates
impl PersistenceManager {
    pub fn set_rating(&self, file: &File, rating: &Option<Rating>) -> Result<(), String> {
        self.conn.execute(
            "INSERT OR REPLACE INTO File (name, rating) VALUES (?1, ?2)",
            &[&file.name() as &ToSql, rating])
            .map(|_| ())
            .err_to_string()
    }

    pub fn populate_files(&self, files: &mut Vec<File>) -> Result<(), String> {
        use std::collections::HashMap;
        let results: HashMap<String, _> = self.conn
            .prepare("SELECT name, rating FROM File").err_to_string()?
            .query_map(NO_PARAMS, |row| (row.get::<_, String>(0), row.get::<_, Option<i64>>(1))).err_to_string()?
            .filter_map(|result| result.ok())
            .collect();

        for file in files {
            if let Some(rating) = results.get(&file.name()) {
                file.rating = rating.map(&Rating::from);
            }
        }

        Ok(())
    }
}

impl ToSql for Rating {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, Error> {
        Ok(ToSqlOutput::Owned(Value::Integer(self.as_i64())))
    }
}