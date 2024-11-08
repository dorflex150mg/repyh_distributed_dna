use rusqlite::Connection;
use std::fmt;
use thiserror::Error;

use crate::model::dna_sequence::DnaSequence;
use crate::model::public_key::PublicKey;

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
    NoPublicKeys,
}

impl fmt::Display for EmptyTableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EmptyTableError::NoDnaSequences => write!(f, "No Dna Sequences in the database."),
            EmptyTableError::NoPublicKeys => write!(f, "No PublicKeys in the database."),
        }
    }
}

fn create_tables(connection: Connection) -> Result<Connection, rusqlite::Error> {
    connection.execute(
        "CREATE TABLE IF NOT EXISTS DnaSequence( 
            id TEXT PRIMARY KEY,
            dna_sequence TEXT
        );",
        []
    )?;
    connection.execute(
        "CREATE TABLE IF NOT EXISTS PublicKey(
            id TEXT PRIMARY KEY,
            public_key BLOB
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

    pub fn push_dna_sequence(&self, dna_sequence: &DnaSequence) -> Result<String, rusqlite::Error> {
        self.connection.execute(
            &format!(
                "INSERT OR REPLACE INTO DnaSequence(id, dna_sequence) VALUES(\"{}\", \"{}\")", 
                dna_sequence.id.clone(), 
                dna_sequence.dna_sequence
            ),
            [],
        )?;
        Ok(dna_sequence.id.clone())
    }

    pub fn push_public_key(&self, public_key: &PublicKey) -> Result<String, rusqlite::Error> {
        self.connection.execute(
                "INSERT OR REPLACE INTO PublicKey(id, public_key) VALUES(?1, ?2)", 
                (public_key.id.clone(), 
                public_key.public_key.clone())
        )?;
        Ok(public_key.id.clone())
    }

    pub fn get_public_key(&self, id: &String) -> Result<PublicKey, QuerryError> {
        let mut query = self.connection.prepare("SELECT id, public_key FROM PublicKey WHERE id = ?1;")?;
        let mut rows = query.query(rusqlite::params![id])?;
        let maybe_row = rows.next()?;
        let row = maybe_row.ok_or(EmptyTableError::NoPublicKeys)?;
        Ok(PublicKey {
            id: row.get(0)?,
            public_key: row.get(1)?
        })
    }
        
    pub fn get_dna_sequence(&self, id: String) -> Result<DnaSequence, QuerryError> {
        let mut query = self.connection.prepare("SELECT id, dna_sequence FROM DnaSequence WHERE id = ?1;")?;
        let mut rows = query.query(rusqlite::params![id])?;
        let maybe_row = rows.next()?;
        let row = maybe_row.ok_or(EmptyTableError::NoDnaSequences)?;
        Ok(DnaSequence {
            id: row.get(0)?,
            dna_sequence: row.get(1)?
        })
    }
}

    

