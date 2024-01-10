use crate::structs_utils::*;
use chrono::prelude::*;
use rusqlite::{Connection, Result};

mod bs_items_cats_timeline;
use bs_items_cats_timeline::*;

#[derive(Debug, PartialEq)]
enum BalanceSheetSelection<'a> {
    Some(&'a Item),
    NewCategory,
    NewItem,
    RenameCategory,
    GoBack,
}

pub fn balance_sheet_half_entry_point(
    conn: &Connection,
    user: &User,
    which_half: BalanceSheetHalf,
) {
    initialize_balance_sheet(conn, user);
    let (mut categories, mut items) = get_relevant_items_cats(conn, user, &which_half)
        .expect("There was an error accessing the Balance Sheet Database");
    loop {
        let response = print_balance_sheet_half_get_response(&categories, &items, &which_half);

        match response {
            BalanceSheetSelection::Some(item) => {
                let idx = items
                    .iter()
                    .position(|i| i == item)
                    .expect("Error sending this item for updating");
                update_item(conn, user, &which_half, &mut categories, &mut items, idx);
            }
            BalanceSheetSelection::NewCategory => {
                create_new_category(conn, user, &which_half, &mut categories)
            }
            BalanceSheetSelection::NewItem => {
                create_new_item(conn, user, &which_half, &mut categories, &mut items)
            }
            BalanceSheetSelection::RenameCategory => {
                rename_category(conn, user, &which_half, &mut categories)
            }
            BalanceSheetSelection::GoBack => return,
        }
    }
}

pub fn balance_sheet_whole_entry_point(conn: &Connection, user: &User) {
    initialize_balance_sheet(conn, user);
    let (asset_categories, asset_items) =
        get_relevant_items_cats(conn, user, &BalanceSheetHalf::Assets)
            .expect("There was an error accessing the Balance Sheet Assets from the Database");
    let (liability_categories, liability_items) =
        get_relevant_items_cats(conn, user, &BalanceSheetHalf::Liabilities)
            .expect("There was an error accessing the Balance Sheet Liabilities from the Database");
    loop {
        let (response, net_worth) = print_balance_sheet_get_response(
            &asset_categories,
            &asset_items,
            &liability_categories,
            &liability_items,
        );

        match response {
            0 => return,
            1 => create_snapshot(conn, user, net_worth).expect("Error creating snapshot"),
            2 => view_snapshot_menu(conn, user).expect("Error accessing snapshots"),
            3 => println!("Coming soon"),
            x => panic!("Response {} is an error state. Exiting the program.", x),
        }
    }
}

// TODO: Don't need to check here if items / categories match which half
/// Print out the half of the balance sheet and find out what the user wants to do
fn print_balance_sheet_half_get_response<'a, 'b, 'c>(
    categories: &'a Vec<Category>,
    items: &'b Vec<Item>,
    which_half: &'c BalanceSheetHalf,
) -> BalanceSheetSelection<'b> {
    println!("\nCurrent list of {}:", which_half.to_str().to_lowercase());
    let mut idx: usize = 1;
    let mut sorted_items: Vec<&Item> = vec![];

    for category in categories {
        if category.is_asset == which_half.to_bool() {
            let mut no_items_found_in_cat = true;
            // Check if any of the items are in this category
            for item in items {
                if item.category_lower == category.category_lower {
                    no_items_found_in_cat = false;
                }
            }
            if no_items_found_in_cat {
                continue; // Don't need to print this category if it has no items
            }
            println!("{}", category.category);
            for item in items {
                if item.is_asset == which_half.to_bool()
                    && item.category_lower == category.category_lower
                {
                    print!("    {}. {} ", idx, item.item);
                    let num_dashes: usize = MAX_CHARACTERS_ITEM_NAME + 1 - item.item.len();
                    for _ in 0..num_dashes {
                        print!("-");
                    }
                    println!(" {}", item.value);
                    idx += 1;
                    sorted_items.push(item);
                }
            }
        }
    }
    println!("\n{}. NEW CATEGORY", idx);
    idx += 1;
    println!("  {}. NEW ITEM", idx);
    idx += 1;
    println!("{}. RENAME CATEGORY", idx);
    println!("\n 0. GO BACK - Balance Sheet Menu");
    let response = print_instr_get_response(0, idx, || {
        println!("\nEnter the number of the item you'd like to update / delete, or one of the other numbers");
    });
    match response {
        0 => BalanceSheetSelection::GoBack,
        x if x > 0 && x <= idx - 3 => BalanceSheetSelection::Some(sorted_items.remove(x - 1)),
        x if x == idx - 2 => BalanceSheetSelection::NewCategory,
        x if x == idx - 1 => BalanceSheetSelection::NewItem,
        x if x == idx => BalanceSheetSelection::RenameCategory,
        x => panic!("Response {} is an error state. Exiting the program.", x),
    }
}

fn print_balance_sheet_get_response(
    asset_categories: &Vec<Category>,
    asset_items: &Vec<Item>,
    liability_categories: &Vec<Category>,
    liability_items: &Vec<Item>,
) -> (usize, f64) {
    let today_date = Local::now().format("%Y-%m-%d").to_string();
    println!("\n\nCurrent balance sheet - {}", today_date);
    println!("\nASSETS");
    let mut idx: usize = 1;
    let mut asset_total: f64 = 0.0;
    for category in asset_categories {
        let mut no_items_found_in_cat = true;
        // Check if any of the items are in this category
        for item in asset_items {
            if item.category_lower == category.category_lower {
                no_items_found_in_cat = false;
            }
        }
        if no_items_found_in_cat {
            continue; // Don't need to print this category if it has no items
        }
        println!("{}", category.category);
        for item in asset_items {
            if item.category_lower == category.category_lower {
                print!("    {}. {} ", idx, item.item);
                let num_dashes: usize = MAX_CHARACTERS_ITEM_NAME + 1 - item.item.len();
                for _ in 0..num_dashes {
                    print!("-");
                }
                println!(" {}", item.value);
                idx += 1;
                asset_total += item.value;
            }
        }
    }
    println!("____________________________________________");
    println!("Total Assets                    {}", asset_total);

    println!("\nLIABILITIES");
    let mut idx: usize = 1;
    let mut liability_total: f64 = 0.0;
    for category in liability_categories {
        let mut no_items_found_in_cat = true;
        // Check if any of the items are in this category
        for item in liability_items {
            if item.category_lower == category.category_lower {
                no_items_found_in_cat = false;
            }
        }
        if no_items_found_in_cat {
            continue; // Don't need to print this category if it has no items
        }
        println!("{}", category.category);
        for item in liability_items {
            if item.category_lower == category.category_lower {
                print!("    {}. {} ", idx, item.item);
                let num_dashes: usize = MAX_CHARACTERS_ITEM_NAME + 1 - item.item.len();
                for _ in 0..num_dashes {
                    print!("-");
                }
                println!(" {}", item.value);
                idx += 1;
                liability_total += item.value;
            }
        }
    }
    println!("____________________________________________");
    println!("Total Liabilities               {}", liability_total);

    println!(
        "\n\nTOTAL NET WORTH --------------- {}",
        asset_total - liability_total
    );

    println!("\n\nWith Balance Sheet Snapshots, you can store this current version of the Balance Sheet.");
    println!("It can then later be viewed or analyzed in aggregate for trends.");
    println!("It is recommended to do this periodically (such as monthly or quarterly).\n");
    let response = print_instr_get_response(0, 3, || {
        println!("1. Take a snapshot");
        println!("2. View or edit a snapshots");
        println!("3. Trend Analysis");
        println!("\n0. Go Back - Balance Sheet Menu");
    });
    (response, asset_total - liability_total)
}

/// Get the relevant half of the balance sheet (items and categories)
fn get_relevant_items_cats(
    conn: &Connection,
    user: &User,
    which_half: &BalanceSheetHalf,
) -> Result<(Vec<Category>, Vec<Item>)> {
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
            category_lower: row.get(1)?,
            username_lower: row.get(2)?,
            is_asset: row.get(3)?,
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
            item_lower: row.get(1)?,
            value: row.get(2)?,
            category: row.get(3)?,
            category_lower: row.get(4)?,
            username_lower: row.get(5)?,
            is_asset: row.get(6)?,
            timeline_created: row.get(7)?,
            timeline_original: row.get(8)?,
            is_deleted: row.get(9)?,
            timeline_deleted: row.get(10)?,
        })
    }
    Ok((categories, items))
}

/// Store a snapshot in the database (not that it mainly stores a timestamp and sparse details)
/// The Balance Sheet can be reconstructed by accessing the database with this information
fn create_snapshot(conn: &Connection, user: &User, net_worth: f64) -> Result<()> {
    // Get current timestamp without updating it
    // This should be the last used timestamp on the timeline
    let timestamp: usize;
    let mut stmt =
        conn.prepare("SELECT timestamp FROM balance_timeline WHERE username_lower = ?1")?;
    let mut rows = stmt.query(rusqlite::params![user.username_lower])?;
    timestamp = rows
        .next()?
        .expect("Timeline query returned empty")
        .get(0)?;

    // Check if a snapshot has already been made for this timestamp
    let mut stmt = conn.prepare(
        "SELECT timestamp FROM balance_snapshots WHERE timestamp = ?1 AND username_lower = ?2",
    )?;
    let mut rows = stmt.query(rusqlite::params![timestamp, user.username_lower])?;
    let row = rows.next()?;
    match row {
        // Already created
        Some(_same_timestamp) => {
            println!("A snapshot has already been created for the current balance sheet state.");
            return Ok(());
        }
        // New timestamp
        None => {} // Code below
    }

    let date_today = Local::now().format("%Y-%m-%d").to_string(); // YYYY-MM-DD

    println!("\nEnter an optional comment about this snapshot (Just hit Enter to skip):");
    let comment: String = read_or_quit();

    // Insert the snapshot into the table
    conn.execute(
        "INSERT INTO balance_snapshots
        (timestamp, username_lower, date_text, net_worth, comment, is_deleted)
        VALUES (?1, ?2, ?3, ?4, ?5, 0)",
        (
            timestamp,
            user.username.to_ascii_lowercase(),
            date_today,
            net_worth,
            comment,
        ),
    )?;

    Ok(())
}

/// List the snapshots and offer to open one of them up
fn view_snapshot_menu(conn: &Connection, user: &User) -> Result<()> {
    println!("\n\nSaved Snapshots:");
    // Push all of the snapshots to a vector first
    let mut snapshots: Vec<Snapshot> = vec![];
    let mut stmt = conn.prepare("SELECT * FROM balance_snapshots WHERE username_lower = ?1")?;
    let mut rows = stmt.query(rusqlite::params![user.username_lower])?;
    while let Some(row) = rows.next()? {
        snapshots.push(Snapshot {
            timeline: row.get(0)?,
            username_lower: row.get(1)?,
            date_today: row.get(2)?,
            net_worth: row.get(3)?,
            comment: row.get(4)?,
            is_deleted: row.get(5)?,
        })
    }
    // Print out listing of snapshots
    snapshots.sort_by(|a, b| a.timeline.cmp(&b.timeline));
    let mut idx: usize = 1;
    for snapshot in &snapshots {
        println!(
            "{}.  {}: Net Worth ${}",
            idx, snapshot.date_today, snapshot.net_worth
        );
        idx += 1;
    }
    println!("\n0. GO BACK");
    let response = print_instr_get_response(0, idx, || {
        println!("Which snapshot would you like to view");
    });
    // Go back or go to snapshot viewer
    match response {
        0 => {
            return Ok(());
        }
        x if x > 0 && x <= idx => {
            view_single_snapshot(conn, user, &mut snapshots, idx - 1);
        }
        x => panic!("Response {} is an error state. Exiting the program.", x),
    }

    Ok(())
}

/// Display the Balance Sheet represented by the snapshot
fn view_single_snapshot(conn: &Connection, user: &User, snapshots: &mut Vec<Snapshot>, idx: usize) {
    let relevant_snapshot = snapshots
        .get(idx)
        .expect("Error accessing requested snapshot");
    let (asset_categories, asset_items) = get_snapshot_items_cats(
        conn,
        user,
        &BalanceSheetHalf::Assets,
        relevant_snapshot.timeline,
    )
    .expect("There was an error accessing the Balance Sheet Assets from the Database");
    let (liability_categories, liability_items) = get_snapshot_items_cats(
        conn,
        user,
        &BalanceSheetHalf::Liabilities,
        relevant_snapshot.timeline,
    )
    .expect("There was an error accessing the Balance Sheet Assets from the Database");
    println!(
        "\n\nSNAPSHOT of Balance Sheet - {}",
        relevant_snapshot.date_today
    );

    println!("\nASSETS");
    let mut idx: usize = 1;
    let mut asset_total: f64 = 0.0;
    for category in &asset_categories {
        let mut no_items_found_in_cat = true;
        // Check if any of the items are in this category
        for item in &asset_items {
            if item.category_lower == category.category_lower {
                no_items_found_in_cat = false;
            }
        }
        if no_items_found_in_cat {
            continue; // Don't need to print this category if it has no items
        }
        println!("{}", category.category);
        for item in &asset_items {
            if item.category_lower == category.category_lower {
                print!("    {}. {} ", idx, item.item);
                let num_dashes: usize = MAX_CHARACTERS_ITEM_NAME + 1 - item.item.len();
                for _ in 0..num_dashes {
                    print!("-");
                }
                println!(" {}", item.value);
                idx += 1;
                asset_total += item.value;
            }
        }
    }
    println!("____________________________________________");
    println!("Total Assets                    {}", asset_total);

    println!("\nLIABILITIES");
    let mut idx: usize = 1;
    let mut liability_total: f64 = 0.0;
    for category in &liability_categories {
        let mut no_items_found_in_cat = true;
        // Check if any of the items are in this category
        for item in &liability_items {
            if item.category_lower == category.category_lower {
                no_items_found_in_cat = false;
            }
        }
        if no_items_found_in_cat {
            continue; // Don't need to print this category if it has no items
        }
        println!("{}", category.category);
        for item in &liability_items {
            if item.category_lower == category.category_lower {
                print!("    {}. {} ", idx, item.item);
                let num_dashes: usize = MAX_CHARACTERS_ITEM_NAME + 1 - item.item.len();
                for _ in 0..num_dashes {
                    print!("-");
                }
                println!(" {}", item.value);
                idx += 1;
                liability_total += item.value;
            }
        }
    }
    println!("____________________________________________");
    println!("Total Liabilities               {}", liability_total);

    println!(
        "\n\nTOTAL NET WORTH --------------- {}",
        asset_total - liability_total
    );

    // Get response
    println!("\n\nWhat would you like to do next?");
    println!("1. Go Back");
    println!("2. Delete this Snapshot");
    let response = print_instr_get_response(1, 2, || {});
    match response {
        1 => {
            return;
        }
        2 => {
            println!("Are you sure you'd like to delete this Snapshot? This cannot be undone.");
            println!("1. Yes");
            println!("2. No (Go back)");
            match print_instr_get_response(1, 2, || {}) {
                1 => {
                    // THIS IS WHERE I AM
                    // Delete the item from the database and from the mutable vector
                    let timeline: usize = get_and_update_timeline(conn, user);
                    conn.execute(
                        "UPDATE balance_items 
                        SET is_deleted = 1, timeline_deleted = ?1
                        WHERE item_lower = ?2 AND username_lower = ?3 AND timeline_created = ?4",
                        (
                            &timeline,
                            &item_chosen.item_lower,
                            &user.username_lower,
                            &item_chosen.timeline_created,
                        ),
                    )
                    .expect("Error deleting the item");
                    // The item is already removed from the vector and will go out of scope here
                    return;
                }
                2 => {
                    // Must push the item back into the vector
                    return;
                }
                x => panic!("Response {} is an error state. Exiting the program.", x),
            }
        }
        x => panic!("Response {} is an error state. Exiting the program.", x),
    }
}

/// Get the items and categories relevant to a snapshot timestamp
/// This will return either the assets or liabilities (call twice for both)
fn get_snapshot_items_cats(
    conn: &Connection,
    user: &User,
    which_half: &BalanceSheetHalf,
    timeline: usize,
) -> Result<(Vec<Category>, Vec<Item>)> {
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
            category_lower: row.get(1)?,
            username_lower: row.get(2)?,
            is_asset: row.get(3)?,
        })
    }
    // Next push all of the items that fit the snapshot into a vector
    let mut items: Vec<Item> = vec![];
    let mut stmt = conn.prepare(
        "SELECT * FROM balance_items 
        WHERE is_asset=?1 AND username_lower=?2 AND timeline_created<=?3 AND timeline_deleted>?3",
    )?;
    let mut rows = stmt.query(rusqlite::params![
        which_half.to_bool_int(),
        user.username_lower,
        timeline
    ])?;
    while let Some(row) = rows.next()? {
        items.push(Item {
            item: row.get(0)?,
            item_lower: row.get(1)?,
            value: row.get(2)?,
            category: row.get(3)?,
            category_lower: row.get(4)?,
            username_lower: row.get(5)?,
            is_asset: row.get(6)?,
            timeline_created: row.get(7)?,
            timeline_original: row.get(8)?,
            is_deleted: row.get(9)?,
            timeline_deleted: row.get(10)?,
        })
    }
    Ok((categories, items))
}

/// Set up the tables for the balance sheet for this user
fn initialize_balance_sheet(conn: &Connection, user: &User) {
    conn.execute_batch(
        "DROP TABLE IF EXISTS balance_items;
        DROP TABLE IF EXISTS balance_categories;
        DROP TABLE IF EXISTS balance_timeline;
        DROP TABLE IF EXISTS balance_snapshots",
    )
    .unwrap();
    // Create the balance_categories table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS balance_categories (
            category TEXT NOT NULL,
            category_lower TEXT NOT NULL,
            username_lower TEXT NOT NULL,
            is_asset INTEGER NOT NULL,
            PRIMARY KEY (category_lower, username_lower, is_asset),
            FOREIGN KEY (username_lower) REFERENCES users (username_lower)
        )",
        (),
    )
    .expect("Error connecting with the balance sheet categories table");

    // Add an Uncategorized type to the table for assets if it's not there yet
    conn.execute(
        "INSERT OR IGNORE INTO balance_categories 
        (category, category_lower, username_lower, is_asset) 
        VALUES (\"Uncategorized\", \"uncategorized\", ?1, 1)",
        rusqlite::params![&user.username_lower],
    )
    .expect("Error initializing the balance_categories table");

    // Add an Uncategorized type to the table for liabilities if it's not there yet
    conn.execute(
        "INSERT OR IGNORE INTO balance_categories 
        (category, category_lower, username_lower, is_asset) 
        VALUES (\"Uncategorized\", \"uncategorized\", ?1, 0)",
        rusqlite::params![&user.username_lower],
    )
    .expect("Error initializing the balance_categories table");

    // Create the balance_items table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS balance_items (
                item TEXT NOT NULL,
                item_lower TEXT NOT NULL,
                value REAL NOT NULL,
                category TEXT NOT NULL,
                category_lower TEXT NOT NULL,
                username_lower TEXT NOT NULL,
                is_asset INTEGER NOT NULL,
                timeline_created INTEGER NOT NULL,
                timeline_original INTEGER NOT NULL,
                is_deleted INTEGER NOT NULL,
                timeline_deleted INTEGER NOT NULL,
                PRIMARY KEY (item_lower, username_lower, timeline_created),
                FOREIGN KEY (username_lower) REFERENCES users (username_lower),
                FOREIGN KEY (category_lower, username_lower, is_asset) REFERENCES balance_categories (category_lower, username_lower, is_asset)
            );",
        (),
    )
    .expect("Error connecting with the balance sheet items table");

    // Create the timeline table to persist a timeline value per user
    // This stores an incrementing integer to demarcate a timeline
    // The timeline helps differentiate current a past values while avoiding unneccessary dates
    // It stores the most recently used integer. It should be pulled from the DB, incremented, used, then returned.
    conn.execute(
        "CREATE TABLE IF NOT EXISTS balance_timeline (
                timestamp INTEGER NOT NULL,
                username_lower TEXT NOT NULL,
                PRIMARY KEY (username_lower)
                FOREIGN KEY (username_lower) REFERENCES users (username_lower)
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

    // Create the table to store snapshots
    // Minimal information is stored here
    // The snapshots are recreated as needed by accessing the category and item tables
    conn.execute(
        "CREATE TABLE IF NOT EXISTS balance_snapshots (
                timestamp INTEGER NOT NULL,
                username_lower TEXT NOT NULL,
                date_text TEXT NOT NULL,
                net_worth REAL NOT NULL,
                comment TEXT NOT NULL,
                is_deleted INTEGER NOT NULL,
                PRIMARY KEY (timestamp, username_lower)
                FOREIGN KEY (username_lower) REFERENCES users (username_lower)
            );",
        (),
    )
    .expect("Error connecting with the balance sheet timeline table");
}
