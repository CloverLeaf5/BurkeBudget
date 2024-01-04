use crate::structs_utils::*;
use rusqlite::{Connection, Result};

#[derive(Debug)]
pub struct Category {
    pub category: String,
    pub username_lower: String,
    pub is_asset: bool,
}

pub struct Item {
    pub item: String,
    pub value: f64,
    pub category: String,
    pub username_lower: String,
    pub is_asset: bool,
    pub is_deleted: bool,
    pub timeline_created: usize,
    pub timeline_deleted: usize,
}

pub fn balance_sheet_entry(conn: &Connection, user: &User, which_half: BalanceSheetHalf) {
    initialize_balance_sheet(conn, user);
    get_relevant_items(conn, user, &which_half).unwrap();

    println!("{:?}", user);
}

/// Get the relevant half of the balance sheet
fn get_relevant_items(
    conn: &Connection,
    user: &User,
    which_half: &BalanceSheetHalf,
) -> Result<(Vec<Category>, Vec<Item>)> {
    println!("\nCurrent list of {}", which_half.to_str().to_lowercase());
    // Push all of the categories to a vector first
    let mut categories: Vec<Category> = vec![];
    let mut stmt =
        conn.prepare("SELECT * FROM balance_categories WHERE is_asset=?1 AND username_lower=?2")?;
    let mut rows = stmt.query(rusqlite::params![
        which_half.to_bool_int(),
        user.username_lower
    ])?;
    while let Some(row) = rows.next()? {
        categories.push(Category {
            category: row.get(0)?,
            username_lower: row.get(1)?,
            is_asset: row.get(2)?,
        })
    }
    // Next push all of the active items to a vector
    let mut items: Vec<Item> = vec![];
    let mut stmt = conn.prepare(
        "SELECT * FROM balance_items WHERE is_deleted=0 AND is_asset=?1 AND username_lower=?2",
    )?;
    let mut rows = stmt.query(rusqlite::params![
        which_half.to_bool_int(),
        user.username_lower
    ])?;
    while let Some(row) = rows.next()? {
        items.push(Item {
            item: row.get(0)?,
            value: row.get(1)?,
            category: row.get(2)?,
            username_lower: row.get(23)?,
            is_asset: row.get(4)?,
            is_deleted: row.get(5)?,
            timeline_created: row.get(6)?,
            timeline_deleted: row.get(7)?,
        })
    }
    Ok((categories, items))
}

/// Set up the tables for the balance sheet for this user
fn initialize_balance_sheet(conn: &Connection, user: &User) {
    // conn
    //     .execute_batch(
    //         "DROP TABLE balance_categories;
    //         DROP TABLE balance_items;",
    //     )
    //     .expect("SKIP");

    // Create the balance_categories table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS balance_categories (
            category TEXT NOT NULL,
            username_lower TEXT NOT NULL,
            is_asset INTEGER NOT NULL,
            PRIMARY KEY (category, username_lower, is_asset),
            FOREIGN KEY (username_lower) REFERENCES users (username_lower)
        )",
        (),
    )
    .expect("Error connecting with the balance sheet categories table");

    // Add a None type to the table for assets if it's not there yet
    conn.execute(
        "INSERT OR IGNORE INTO balance_categories (category, username_lower, is_asset, is_deleted) VALUES (\"None\", ?1, 1, 0)",
        rusqlite::params![&user.username_lower],
    )
    .expect("Error initializing the balance_categories table");

    // Add a None type to the table for liabilities if it's not there yet
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
                timeline_created INTEGER NOT NULL,
                is_deleted INTEGER NOT NULL,
                timeline_deleted INTEGER NOT NULL,
                PRIMARY KEY (item, username_lower, timeline_created),
                FOREIGN KEY (username_lower) REFERENCES users (username_lower),
                FOREIGN KEY (category) REFERENCES categories (category)
            );",
        (),
    )
    .expect("Error connecting with the balance sheet items table");

    // Create the timeline table to persist a timeline value per user
    // This stores an incrementing integer to demarcate a timeline
    // The timeline helps differentiate current a past values while avoiding unneccessary dates
    conn.execute(
        "CREATE TABLE IF NOT EXISTS balance_timeline (
                timestamp INTEGER NOT NULL,
                username_lower TEXT NOT NULL,
                PRIMARY KEY (timestamp, username_lower)
            );",
        (),
    )
    .expect("Error connecting with the balance sheet timeline table");

    // Initialize the timeline to 0 for this user if it's not there yet
    conn.execute(
        "INSERT OR IGNORE INTO balance_timeline (timestamp, username_lower) VALUES (0, ?1)",
        rusqlite::params![&user.username_lower],
    )
    .expect("Error initializing the balance_timeline table");
}
