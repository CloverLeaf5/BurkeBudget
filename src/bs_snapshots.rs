use super::bs_items_cats_timeline::get_and_update_timeline;
use crate::structs_utils::*;
use chrono::prelude::*;
use rusqlite::{Connection, Result};

/// Store a snapshot in the database (not that it mainly stores a timestamp and sparse details)
/// The Balance Sheet can be reconstructed by accessing the database with this information
pub fn create_snapshot(conn: &Connection, user: &User, net_worth: f64) -> Result<()> {
    // The timestamp is incremented with a new timestamp to allow for multiple snapshots for the same balance sheet state
    let timestamp = get_and_update_timeline(conn, user);

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

    println!("\nSnapshot successfully created. Press Enter to continue.");
    read_or_quit(); // Give the user a chance to acknowledge

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
                "{}.  {}:  Net Worth  {}",
                idx,
                snapshot.date_today,
                to_money_string(snapshot.net_worth)
            );
        }
        println!("\n0. GO BACK");
        let response = print_instr_get_response(0, idx, || {
            println!("\nWhich snapshot would you like to view");
        });
        // Go back or go to snapshot viewer
        match response {
            0 => {
                return Ok(());
            }
            x if x > 0 && x <= idx => {
                view_single_snapshot(conn, user, &mut snapshots, x - 1)
                    .expect("Error with the snapshot viewer: Exiting the program");
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
) -> Result<()> {
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
                println!(" {}", to_money_string(item.value));
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
    println!("{}", to_money_string(asset_total));

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
                println!(" {}", to_money_string(item.value));
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
    println!("{}", to_money_string(liability_total));

    // Grand total
    let total = asset_total - liability_total;
    println!(
        "\n\nTOTAL NET WORTH -------------------  {}",
        to_money_string(total)
    );

    // Print out the comment under the snapshot
    if !relevant_snapshot.comment.is_empty() {
        println!("\nComment: \"{}\"", relevant_snapshot.comment);
    }

    // Get response
    println!("\n\nWhat would you like to do next?");
    println!("1. Go Back");
    println!("2. Delete this Snapshot");
    let response = print_instr_get_response(1, 2, || {});
    match response {
        1 => Ok(()),
        2 => {
            // The deletion branch
            println!("\nAre you sure you'd like to delete this Snapshot? This cannot be undone.");
            println!("1. Yes");
            println!("2. No (Go back)");
            match print_instr_get_response(1, 2, || {}) {
                1 => {
                    // Can have multiple deleted snapshots at the same timeline value
                    // Must find the last deleted one and increment the deletion integer for this one
                    let mut stmt = conn.prepare(
                        "SELECT is_deleted FROM balance_snapshots 
                        WHERE username_lower = ?1 AND timestamp = ?2 AND is_deleted > 0",
                    )?;
                    let mut rows = stmt.query(rusqlite::params![
                        user.username_lower,
                        relevant_snapshot.timeline
                    ])?;
                    let mut prev_deleted: Vec<usize> = vec![];
                    while let Some(row) = rows.next()? {
                        prev_deleted.push(row.get(0)?)
                    }

                    prev_deleted.sort(); // Low to high
                    let mut deletion_number: usize = 1;
                    if let Some(last_deleted_timeline) = prev_deleted.last() {
                        if last_deleted_timeline >= &deletion_number {
                            deletion_number = *last_deleted_timeline + 1;
                        }
                    }
                    // Delete the snapshot from the database and from the mutable vector
                    // Set the is_deleted value to be the number calculated above
                    conn.execute(
                        "UPDATE balance_snapshots 
                        SET is_deleted = ?1
                        WHERE timestamp = ?2 AND username_lower = ?3  AND is_deleted = 0",
                        (
                            deletion_number,
                            relevant_snapshot.timeline,
                            &user.username_lower,
                        ),
                    )
                    .expect("Error deleting the item");
                    // All of the items and categories should remain unchanged
                    // Remove the snapshot from the Vector of Snapshots
                    // The remove function should maintain the sorted order of the snapshots
                    snapshots.remove(snapshot_idx);
                    Ok(())
                }
                2 => Ok(()),
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
