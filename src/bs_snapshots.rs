use crate::structs_utils::*;
use chrono::prelude::*;
use rusqlite::{Connection, Result};
use rusty_money::{iso, Money};

/// Store a snapshot in the database (not that it mainly stores a timestamp and sparse details)
/// The Balance Sheet can be reconstructed by accessing the database with this information
pub fn create_snapshot(conn: &Connection, user: &User, net_worth: f64) -> Result<()> {
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
pub fn view_snapshot_menu(conn: &Connection, user: &User) -> Result<()> {
    // Push all of the snapshots to a vector first
    let mut snapshots: Vec<Snapshot> = vec![];
    let mut stmt = conn
        .prepare("SELECT * FROM balance_snapshots WHERE username_lower = ?1 AND is_deleted = 0")?;
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

    // Sort the snapshots in chronological order
    snapshots.sort_by(|a, b| a.timeline.cmp(&b.timeline));

    // Print out listing of snapshots
    loop {
        println!("\n\nSaved Snapshots:");
        let mut idx: usize = 0;
        for snapshot in &snapshots {
            idx += 1;
            println!(
                "{}.  {}: Net Worth {}",
                idx,
                snapshot.date_today,
                Money::from_str(snapshot.net_worth.to_string().as_str(), iso::USD).unwrap()
            );
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
                view_single_snapshot(conn, user, &mut snapshots, x - 1);
            }
            x => panic!("Response {} is an error state. Exiting the program.", x),
        }
    }
}

/// Display the Balance Sheet represented by the snapshot
fn view_single_snapshot(
    conn: &Connection,
    user: &User,
    snapshots: &mut Vec<Snapshot>,
    snapshot_idx: usize,
) {
    let relevant_snapshot = snapshots
        .get(snapshot_idx)
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
                println!(
                    " {}",
                    Money::from_str(item.value.to_string().as_str(), iso::USD).unwrap()
                );
                idx += 1;
                asset_total += item.value;
            }
        }
    }
    // Print sum
    for _ in 0..(MAX_CHARACTERS_ITEM_NAME + 24) {
        print!("_");
    }
    print!("\nTotal Assets");
    for _ in 0..(MAX_CHARACTERS_ITEM_NAME - 2) {
        print!(" ");
    }
    println!(
        "{}",
        Money::from_str(asset_total.to_string().as_str(), iso::USD).unwrap()
    );

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
                println!(
                    " {}",
                    Money::from_str(item.value.to_string().as_str(), iso::USD).unwrap()
                );
                idx += 1;
                liability_total += item.value;
            }
        }
    }
    // Print sum
    for _ in 0..(MAX_CHARACTERS_ITEM_NAME + 24) {
        print!("_");
    }
    print!("\nTotal Liabilities");
    for _ in 0..(MAX_CHARACTERS_ITEM_NAME - 7) {
        print!(" ");
    }
    println!(
        "{}",
        Money::from_str(liability_total.to_string().as_str(), iso::USD).unwrap()
    );

    // Grand total
    let total = asset_total - liability_total;
    println!(
        "\n\nTOTAL NET WORTH -------------------  {}",
        Money::from_str(total.to_string().as_str(), iso::USD).unwrap()
    );

    // Print out the comment under the snapshot
    if relevant_snapshot.comment.len() > 0 {
        println!("\nComment: \"{}\"", relevant_snapshot.comment);
    }

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
                    // Delete the item from the database and from the mutable vector
                    conn.execute(
                        "UPDATE balance_snapshots 
                        SET is_deleted = 1
                        WHERE timestamp = ?1 AND username_lower = ?2",
                        (relevant_snapshot.timeline, &user.username_lower),
                    )
                    .expect("Error deleting the item");
                    // All of the items and categories should remain unchanged
                    // Remove the snapshot from the Vector of Snapshots
                    // The remove function should maintain the sorted order of the snapshots
                    snapshots.remove(snapshot_idx);
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
/// This will return either the assets or liabilities (call once for each)
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
