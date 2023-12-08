use rusqlite::{Connection, Result};

mod db;
pub use db::*;

fn write_welcome() {
    println!("||PBPB\\\\");
    println!("||     ||");
    println!("||    //");
    println!("||LKLK");
    println!("||    \\\\");
    println!("||     ||");
    println!("||     |||");
    println!("||TBTB///");
    println!("");
    println!("Welcome to the Burke Budget App!");
}

fn main() -> Result<()> {
    const DB_PATH: &str = "budgetDb.db";

    write_welcome();

    let conn = Connection::open(DB_PATH)?;

    let username = db::login(&conn);
    println!("User returned is: {}", username);
    Ok(())

    // let me = Person {
    //     id: 0,
    //     name: "Steven".to_string(),
    //     data: None,
    // };
    // conn.execute(
    //     "INSERT INTO person (name, data) VALUES (?1, ?2)",
    //     (&me.name, &me.data),
    // )?;

    // for person in person_iter {
    //     println!("Found person {:?}", person.unwrap());
    // }
    // Ok(())
}

// fn main() {
//     write_welcome();
//     // Sign in
//     // Connect to DB
//     // Give status
//     // Offer menu
// }
