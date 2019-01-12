extern crate rusqlite;

mod migrations;

use std::path::Path;
use rusqlite::Connection;
use crate::support::ErrToString;

#[derive(Debug)]
pub struct PersistenceManager {
    conn: Connection,
}

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
