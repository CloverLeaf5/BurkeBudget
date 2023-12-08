use rusqlite::{Connection, Result};
use text_io::read;

#[derive(Debug)]
pub struct User {
    pub username: String,
    pub firstname: String,
    pub lastname: String,
}

pub fn login(conn: &Connection) -> User {
    // Create the users table if it doesnt exist
    match conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            username TEXT PRIMARY KEY,
            firstname TEXT NOT NULL,
            lastname TEXT NOT NULL
        )",
        (), // empty list of parameters.
    ) {
        Ok(_) => println!("Connected to the users table"),
        Err(error) => println!("Error: {}", error),
    };

    // Get username from user
    println!("What is your username?");
    let username: String = read!();

    // Check if the username is already in the database
    let mut stmt = conn.prepare("SELECT username, firstname, lastname FROM users")?;
    let person_iter = match stmt.query_map([], |row| {
        Ok(User {
            username: row.get(0)?,
            firstname: row.get(1)?,
            lastname: row.get(2)?,
        })
    }){
        Ok(_) => println!("Successfully searched the users table"),
        Err(error) => println!("Error: {}", error),
    }

    person_iter[0].unwrap()
}
