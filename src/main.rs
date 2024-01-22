use rusqlite::Connection;

mod login;
mod menu;
mod structs_utils;

fn write_welcome() {
    println!("\n||PBPB\\\\");
    println!("||     ||");
    println!("||    //");
    println!("||LKLK");
    println!("||    \\\\");
    println!("||     ||");
    println!("||     ||");
    println!("||TBTB//");
    println!("\nWelcome to the Burke Budget App!");
    println!("\nEnter all numbers without commas or dollar signs");
    println!("\nType \"quit\" to quit at any time. This will also logout the current user\n");
}

fn main() {
    const DB_PATH: &str = "burkebudgetDB.db";

    write_welcome();

    let conn = Connection::open(DB_PATH).expect("There was an error connecting to the database");

    // Turn on foreign keys
    conn.execute("PRAGMA foreign_keys = ON", ())
        .expect("Error turning on foreign keys");

    // Login the user
    let user_result = login::login(&conn);
    match user_result {
        Err(error) => println!("There was an error with login: {}", error),
        Ok(user) => {
            // Display the menu
            // The menu will handle the rest of the functionality until the user quits
            menu::main_menu(&conn, &user);
        }
    }

    // Close the application by returning a Result
    println!("\nYour budget is saved, and you have been logged out. See you next time!\n");
}
