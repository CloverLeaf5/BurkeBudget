use crate::structs_utils::*;
use rusqlite::Connection;

pub fn assets(conn: &Connection, user: &User) {
    initialize_balance_sheet_tables(&conn, &user);
    println!("Made it to assets");
}

pub fn liabilities(conn: &Connection, user: &User) {
    initialize_balance_sheet_tables(&conn, &user);

    println!("Made it to liabilities");
}

/// Set up the tables for the balance sheet for this user
fn initialize_balance_sheet_tables(conn: &Connection, user: &User) {
    // conn.execute_batch(
    //     "DROP TABLE balance_categories;
    //         DROP TABLE balance_items;",
    // )
    // .expect("SKIP");
    // Create the balance_categories table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS balance_categories (
            category TEXT NOT NULL,
            username_lower TEXT NOT NULL,
            is_asset INTEGER NOT NULL,
            is_deleted INTEGER NOT NULL,
            PRIMARY KEY (category, username_lower),
            FOREIGN KEY (username_lower) REFERENCES users (username_lower)
        )",
        (),
    )
    .expect("Error connecting with the balance sheet categories table");

    // Add a None type to the table for assets
    conn.execute(
        "INSERT OR IGNORE INTO balance_categories (category, username_lower, is_asset, is_deleted) VALUES (\"None\", ?1, 1, 0)",
        rusqlite::params![&user.username_lower],
    )
    .expect("Error initializing the balance_categories table");

    // Add a None type to the table for liabilities
    conn.execute(
        "INSERT OR IGNORE INTO balance_categories (category, username_lower, is_asset, is_deleted) VALUES (\"None\", ?1, 0, 0)",
        rusqlite::params![&user.username_lower],
    )
    .expect("Error initializing the balance_categories table");

    // Create the balance_items table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS balance_items (
                item TEXT NOT NULL,
                value REAL NOT NULL,
                category TEXT NOT NULL,
                username_lower TEXT NOT NULL,
                is_asset INTEGER NOT NULL,
                date_created INTEGER NOT NULL,
                is_deleted INTEGER NOT NULL,
                date_deleted INTEGER,
                PRIMARY KEY (item, username_lower),
                FOREIGN KEY (category) REFERENCES categories (category)
            );",
        (),
    )
    .expect("Error connecting with the balance sheet items table");
}
