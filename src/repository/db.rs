use rusqlite::Connection;
use std::fmt;
use thiserror::Error;

use crate::model::dna_sequence::DnaSequence;

pub struct DbHandle {
    connection: Connection,
    name: String,
}

#[derive(Error, Debug, derive_more::From, derive_more::Display)]
pub enum QuerryError {
    RusqliteError(rusqlite::Error),
    EmptyTableErrorW(EmptyTableError),
}

#[derive(Error, Debug)]
pub enum EmptyTableError {
    NoDnaSequences,
}

impl fmt::Display for EmptyTableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EmptyTableError::NoDnaSequences => write!(f, "No Agents in the database.\n Consider creating dna_sequences with Agent::new()"),
        }
    }
}

fn create_tables(connection: Connection) -> Result<Connection, rusqlite::Error> {
    connection.execute(
        "CREATE TABLE IF NOT EXISTS dna(
            id TEXT PRIMARY KEY,
            dna_sequence TEXT
        );",
        []
    )?;
    Ok(connection)
}

impl DbHandle {
    pub fn new(name: String) -> Result<Self, rusqlite::Error> {
        let mut connection = Connection::open(&name)?;
        connection = create_tables(connection)?;
        Ok(DbHandle {
            connection,
            name,
        })
    }

    //pub fn push_item(&self, id: String, name: String, price: f64) -> Result<String, rusqlite::Error> {
    //    let request = &format!("INSERT INTO item(id, name, price) VALUES(\"{}\", \"{}\", {});",
    //            id.clone(),
    //            name,
    //            price,
    //        );
    //    println!("request:{request}");
    //    
    //    self.connection.execute(request,
    //        [],
    //    )?;
    //    Ok(id)
    //}

    //pub fn get_item(&self, id: String) -> Result<Item, QuerryError> {
    //    println!("querying id: {}", id);
    //    let mut query = self.connection.prepare("SELECT name, price FROM item WHERE id = ?1;")?;
    //    let mut rows = query.query(rusqlite::params![id])?;
    //    let maybe_row = rows.next()?;
    //    let row = maybe_row.ok_or(EmptyTableError::NoItems)?;
    //    Ok(Item {
    //        id, 
    //        name: row.get(0)?,
    //        price: row.get(1)?,
    //    })
    //}

    pub fn push_dna_sequence(&self, dna_sequence: &DnaSequence) -> Result<String, rusqlite::Error> {
        self.connection.execute(
            &format!("INSERT OR REPLACE INTO dna_sequence(id, dna_sequence) VALUES(\"{}\", \"{}\")", dna_sequence.id.clone(), dna_sequence.dna_sequence),
            [],
        )?;
        Ok(dna_sequence.id.clone())
    }
        
    pub fn get_dna_sequence(&self, id: String) -> Result<DnaSequence, QuerryError> {
        let mut query = self.connection.prepare("SELECT id, dna_sequence FROM dna_sequence WHERE id = ?1;")?;
        let mut rows = query.query(rusqlite::params![id])?;
        let maybe_row = rows.next()?;
        let row = maybe_row.ok_or(EmptyTableError::NoDnaSequences)?;
        Ok(DnaSequence {
            id: row.get(0)?,
            dna_sequence: row.get(1)?
        })
    }
}

    

