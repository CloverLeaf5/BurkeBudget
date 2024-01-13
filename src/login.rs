use crate::structs_utils::*;
use rusqlite::{Connection, Result};

/// Login a user and return that user as a result
pub fn login(conn: &Connection) -> Result<User> {
    // Create the users table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            username TEXT NOT NULL,
            username_lower TEXT NOT NULL,
            firstname TEXT NOT NULL,
            lastname TEXT NOT NULL,
            is_deleted INTEGER NOT NULL,
            PRIMARY KEY (username_lower)
        )",
        (), // empty list of parameters.
    )
    .expect("Error connecting with the list of users");
    // It would be unrecoverable if it can't connect with the users table

    // Get username from user
    println!("Enter username to login or signup:");
    let username: String = read_or_quit();

    // Check if the username is already in the database
    let mut stmt = conn.prepare("SELECT * FROM users WHERE username_lower = ?1")?;
    let mut rows = stmt.query(rusqlite::params![username.to_lowercase()])?;
    match rows.next()? {
        // Username found, return User to main function
        Some(row) => {
            return Ok(User {
                username: row.get(0)?,
                username_lower: row.get(1)?,
                firstname: row.get(2)?,
                lastname: row.get(3)?,
                is_deleted: row.get(4)?,
            })
        }
        // Username not found, may need to see list or signup
        None => {
            println!("\nThat user was not found.");
            match vieworsignup(&username).as_str() {
                // View list of users and choose one or signup
                "view" => {
                    return chooseorsignup(conn, username);
                }
                // Sign up the user
                "signup" => {
                    return signup(conn, username);
                }
                // Should not get any other responses
                _ => {
                    panic!("Error: Unknown response from view or signup function");
                }
            }
        }
    }
}

/// Figure out if the user would like to view all available users or sign up
fn vieworsignup(username: &String) -> String {
    let mut response: String = String::from("0");
    // Repeat the message until an expected response
    while !(response.as_str() == "1" || response.as_str() == "2") {
        println!("Would you like to:");
        println!("1 - See a list of available users");
        println!("2 - Sign up as user {}", username);
        response = read_or_quit();
        if !(response.as_str() == "1" || response.as_str() == "2") {
            println!("Please enter \"1\" or \"2\"");
        }
    }
    // Return the correct &str to the calling function
    if response.as_str() == "1" {
        String::from("view")
    } else {
        String::from("signup")
    }
}

/// Display all of the users
/// User will choose one or choose to signup instead
fn chooseorsignup(conn: &Connection, username: String) -> Result<User> {
    // Push all of the users into a vector as may need to re-use
    let mut users: Vec<User> = vec![];
    let mut stmt = conn.prepare("SELECT * FROM users")?;
    let mut rows = stmt.query(rusqlite::params![])?;
    while let Some(row) = rows.next()? {
        users.push(User {
            username: row.get(0)?,
            username_lower: row.get(1)?,
            firstname: row.get(2)?,
            lastname: row.get(3)?,
            is_deleted: row.get(4)?,
        })
    }

    // Allow choice of users from that list
    let selection = print_instr_get_response(0, users.len(), || {
        // Print out the list of users
        println!("\nComplete list of users:");
        for (idx, user) in users.iter().enumerate() {
            println!("{}. {}", idx + 1, user.username);
        }
        println!("Enter the number beside the user you would like to login with.");
        println!("Enter 0 for none of these and sign up as a new user instead.");
    });

    // Return the selected user or sign up if they entered 0 and return the new user
    if selection == 0 {
        signup(conn, username)
    } else {
        Ok(users.remove(selection - 1))
    }
}

/// Signup a new user and return that user in a result
fn signup(conn: &Connection, username: String) -> Result<User> {
    println!("Sign up a new user with username {}", username);
    // Get name from user
    println!("What is your first name?");
    let firstname: String = read_or_quit();
    println!("What is your last name?");
    let lastname: String = read_or_quit();

    // Insert the new user into the database
    conn.execute(
        "INSERT INTO users (username, username_lower, firstname, lastname, is_deleted) VALUES (?1, ?2, ?3, ?4, 0)",
        (&username, &username.to_lowercase(), &firstname, &lastname),
    )?;

    // Return the new User to be returned to the main function
    return Ok(User {
        username: username.clone(),
        username_lower: username.to_lowercase(),
        firstname: firstname,
        lastname: lastname,
        is_deleted: false,
    });
}
