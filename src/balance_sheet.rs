use crate::structs_utils::*;
use rusqlite::Connection;

pub struct category {
    pub category: String,
    pub username_lower: String,
    pub is_asset: bool,
    pub is_deleted: bool,
}

pub struct item {
    pub item: String,
    pub value: f64,
    pub category: String,
    pub username_lower: String,
    pub is_asset: bool,
    pub is_deleted: bool,
    pub timeline_created: usize,
    pub timeline_deleted: usize,
}

pub fn assets(conn: &Connection, user: &User) {
    let user_clone = user.clone();
    let handle = std::thread::spawn(|| {
        initialize_balance_sheet_tables(user_clone);
    });
    println!("Made it to assets");
    println!("{:?}", user);

    handle.join().expect("Error joining threads.");
}

pub fn liabilities(conn: &Connection, user: &User) {
    let user_clone = user.clone();
    let handle = std::thread::spawn(|| {
        initialize_balance_sheet_tables(user_clone);
    });

    println!("Made it to liabilities");
    println!("{:?}", user);

    handle.join().expect("Error joining threads.");
}

/// Set up the tables for the balance sheet for this user
/// This should finish before the user has a chance to make changes
/// Monitor for SQLITE_BUSY errors
fn initialize_balance_sheet_tables(user: User) {
    let conn2 = Connection::open("budgetDb.db").expect("Error with db connection");

    // conn2
    //     .execute_batch(
    //         "DROP TABLE balance_categories;
    //         DROP TABLE balance_items;",
    //     )
    //     .expect("SKIP");

    // Create the balance_categories table if it doesn't exist
    conn2
        .execute(
            "CREATE TABLE IF NOT EXISTS balance_categories (
            category TEXT NOT NULL,
            username_lower TEXT NOT NULL,
            is_asset INTEGER NOT NULL,
            is_deleted INTEGER NOT NULL,
            PRIMARY KEY (category, username_lower, is_asset),
            FOREIGN KEY (username_lower) REFERENCES users (username_lower)
        )",
            (),
        )
        .expect("Error connecting with the balance sheet categories table");

    // Add a None type to the table for assets if it's not there yet
    conn2.execute(
        "INSERT OR IGNORE INTO balance_categories (category, username_lower, is_asset, is_deleted) VALUES (\"None\", ?1, 1, 0)",
        rusqlite::params![&user.username_lower],
    )
    .expect("Error initializing the balance_categories table");

    // Add a None type to the table for liabilities if it's not there yet
    conn2.execute(
        "INSERT OR IGNORE INTO balance_categories (category, username_lower, is_asset, is_deleted) VALUES (\"None\", ?1, 0, 0)",
        rusqlite::params![&user.username_lower],
    )
    .expect("Error initializing the balance_categories table");

    // Create the balance_items table if it doesn't exist
    conn2
        .execute(
            "CREATE TABLE IF NOT EXISTS balance_items (
                item TEXT NOT NULL,
                value REAL NOT NULL,
                category TEXT NOT NULL,
                username_lower TEXT NOT NULL,
                is_asset INTEGER NOT NULL,
                timeline_created INTEGER NOT NULL,
                is_deleted INTEGER NOT NULL,
                timeline_deleted INTEGER,
                PRIMARY KEY (item, username_lower),
                FOREIGN KEY (username_lower) REFERENCES users (username_lower),
                FOREIGN KEY (category) REFERENCES categories (category)
            );",
            (),
        )
        .expect("Error connecting with the balance sheet items table");

    // Create the timeline table to persist a timeline value per user
    // This stores an incrementing integer to demarcate a timeline
    // The timeline helps differentiate current a past values while avoiding unneccessary dates
    conn2
        .execute(
            "CREATE TABLE IF NOT EXISTS balance_timeline (
                timestamp INTEGER NOT NULL,
                username_lower TEXT NOT NULL,
                PRIMARY KEY (timestamp, username_lower)
            );",
            (),
        )
        .expect("Error connecting with the balance sheet timeline table");

    // Initialize the timeline to 0 for this user if it's not there yet
    conn2
        .execute(
            "INSERT OR IGNORE INTO balance_timeline (timestamp, username_lower) VALUES (0, ?1)",
            rusqlite::params![&user.username_lower],
        )
        .expect("Error initializing the balance_timeline table");
}
