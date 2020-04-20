use diesel::{Connection, PgConnection};

pub mod schema;

pub type DBConnection = PgConnection;
pub type DbBackend = <DBConnection as Connection>::Backend;
