use crate::structs_utils::*;
use rusqlite::{Connection, Result};

#[derive(Debug, PartialEq)]
struct Category {
    category: String,
    category_lower: String,
    username_lower: String,
    is_asset: bool,
}

#[derive(Debug, PartialEq)]
struct Item {
    item: String,
    item_lower: String,
    value: f64,
    category: String,
    category_lower: String,
    username_lower: String,
    is_asset: bool,
    is_deleted: bool,
    timeline_created: usize,
    timeline_deleted: usize,
}

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
    let (mut categories, mut items) = get_relevant_items(conn, user, &which_half)
        .expect("There was an error accessing the Balance Sheet Database");
    loop {
        let response = print_balance_sheet_half_get_response(&categories, &items, &which_half);

        match response {
            BalanceSheetSelection::Some(item) => println!("Coming soon {:?}", item),
            BalanceSheetSelection::NewCategory => {
                create_new_category(conn, user, &which_half, &mut categories)
            }
            BalanceSheetSelection::NewItem => {
                create_new_item(conn, user, &which_half, &mut categories, &mut items)
            }
            BalanceSheetSelection::RenameCategory => println!("Coming soon"),
            BalanceSheetSelection::GoBack => return,
        }
    }
}

/// Print out the half of the balance sheet and find out what the user wants to do
fn print_balance_sheet_half_get_response<'a, 'b, 'c>(
    categories: &'a Vec<Category>,
    items: &'b Vec<Item>,
    which_half: &'c BalanceSheetHalf,
) -> BalanceSheetSelection<'b> {
    println!("\nCurrent list of {}:", which_half.to_str().to_lowercase());
    let mut idx: usize = 1;
    let mut sorted_items: Vec<&Item> = vec![];

    //TODO: Only print categories that have items
    for category in categories {
        if category.is_asset == which_half.to_bool() {
            println!("{}", category.category);
            for item in items {
                if item.is_asset == which_half.to_bool() && item.category == category.category {
                    println!("  {}. {}      {}", idx, item.item, item.value);
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
    println!("\n0. GO BACK");
    let response = print_instr_get_response(0, idx, || {
        println!("Enter the number of the item you'd like to update / delete, or one of the other numbers");
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

/// Category Creator
fn create_new_category(
    conn: &Connection,
    user: &User,
    which_half: &BalanceSheetHalf,
    categories: &mut Vec<Category>,
) {
    println!("What would you like to call the new category?");
    let cat_name = read_or_quit();
    // Check if the category already exists
    for category in &mut *categories {
        if cat_name.to_lowercase() == category.category_lower {
            println!("This category already exists as {}.", category.category);
            println!("It can already be used to create new items.");
            println!(
                "If you would like to edit the name, select \"Rename Category\" on the {} listing page.", which_half.to_str()
            );
            return;
        }
    }
    // Insert the new category into the database
    conn.execute(
        "INSERT INTO balance_categories (category, category_lower, username_lower, is_asset) VALUES (?1, ?2, ?3, ?4)",
        (&cat_name, &cat_name.to_lowercase(), &user.username_lower, &which_half.to_bool_int()),
    ).expect("Error creating new category");
    // Add the category into the categories vector
    categories.push(Category {
        category: cat_name.clone(),
        category_lower: String::from(cat_name.to_lowercase()),
        username_lower: String::from(&user.username_lower),
        is_asset: which_half.to_bool(),
    });
}

/// Item Creator
fn create_new_item(
    conn: &Connection,
    user: &User,
    which_half: &BalanceSheetHalf,
    categories: &mut Vec<Category>,
    items: &mut Vec<Item>,
) {
    println!("What would you like to name the new item?");
    // Get The new item's name
    let item_name = read_or_quit();
    // Check if the item already exists
    for item in &mut *items {
        if item_name.to_lowercase() == item.item_lower {
            println!("This item already exists as {}.", item.item);
            println!("If you would like to edit the item, select it on the next page.");
            return;
        }
    }
    // Get the new item's value
    if which_half.to_bool() {
        // is_asset
        println!("What value does this item have currently?");
    } else {
        println!("What is the current total cost of this liability? (positive number or 0)");
    }
    let mut value: f64 = -1.0;
    while value < 0.0 {
        let val_response = read_or_quit();
        // Make sure the input is valid
        match val_response.parse::<f64>() {
            Ok(poss_value) => {
                if poss_value < 0.0 {
                    println!("\nPlease enter a positive number.");
                } else {
                    value = poss_value;
                }
            }
            Err(_err) => {
                println!("\nPlease enter a valid number.");
            }
        }
    }
    // Get the new item's category
    // Can also make a new category during this process
    let mut still_need_category: bool = true;
    let mut chosen_cat: String = String::new();
    while still_need_category {
        println!("\nWhich category would you like to use for this item?");
        let mut idx: usize = 1;
        println!("0. GO BACK");
        for category in &mut *categories {
            println!("{}. {}", idx, category.category);
            idx += 1;
        }
        println!("\n{}. NEW CATEGORY", idx);
        let response = print_instr_get_response(0, idx, || {});
        match response {
            0 => return,
            x if x > 0 && x < idx => {
                // Selected category
                chosen_cat = categories
                    .get(x - 1)
                    .expect("Unable to access chosen category")
                    .category
                    .clone();
                still_need_category = false;
            }
            x if x == idx => {
                // New category
                let num_cats_before: usize = categories.len();
                create_new_category(conn, user, which_half, categories);
                let num_cats_after: usize = categories.len();
                if num_cats_before == num_cats_after {
                    // This only happens if there was an error in category creation
                    // Must loop again
                    continue;
                } else {
                    chosen_cat = categories
                        .get(num_cats_after - 1)
                        .expect("Unable to access new category")
                        .category
                        .clone();
                    still_need_category = false;
                }
            }
            x => panic!("Response {} is an error state. Exiting the program.", x),
        }
    }

    // Get the new item's timeline_created value and increment it
    let timeline: usize;
    let mut stmt = conn
        .prepare("SELECT timestamp FROM balance_timeline WHERE username_lower = ?1")
        .expect("Error preparing timeline statement");
    let mut rows = stmt
        .query(rusqlite::params![user.username.to_lowercase()])
        .expect("Error accessing timeline");
    match rows
        .next()
        .expect("Error accessing timeline query response")
    {
        // Timeline found
        Some(row) => {
            let timeline_returned: usize = row.get(0).expect("Unable to get timeline from query");
            timeline = timeline_returned + 1;
            // Reinsert the new timeline value into the database
            conn.execute(
                "UPDATE balance_timeline SET timestamp = ?1 WHERE username_lower = ?2",
                (timeline, &user.username_lower),
            )
            .expect("Error updating the timeline database");
        }
        // Timeline not found. This is an error state as this needs to be created during initialization.
        None => {
            panic!("Timeline query returned empty");
        }
    }

    // Insert the new item into the database
    conn.execute(
        "INSERT INTO balance_items 
        (item, item_lower, value, category, category_lower, username_lower, is_asset, timeline_created, is_deleted, timeline_deleted) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        (
            &item_name,
            &item_name.to_lowercase(),
            &value,
            &chosen_cat,
            &chosen_cat.to_lowercase(),
            &user.username_lower,
            &which_half.to_bool_int(),
            &timeline,
            0,
            usize::MAX/4,
        ),
    )
    .expect("Error creating new item");
    // Add the category into the categories vector
    items.push(Item {
        item: item_name.clone(),
        item_lower: String::from(item_name.to_lowercase()),
        value: value,
        category: chosen_cat.clone(),
        category_lower: String::from(chosen_cat.to_lowercase()),
        username_lower: String::from(&user.username_lower),
        is_asset: which_half.to_bool(),
        is_deleted: false,
        timeline_created: timeline,
        timeline_deleted: usize::MAX / 4,
    });
}

/// Get the relevant half of the balance sheet
fn get_relevant_items(
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
            is_deleted: row.get(7)?,
            timeline_created: row.get(8)?,
            timeline_deleted: row.get(9)?,
        })
    }
    Ok((categories, items))
}

/// Set up the tables for the balance sheet for this user
fn initialize_balance_sheet(conn: &Connection, user: &User) {
    conn.execute_batch(
        "DROP TABLE IF EXISTS balance_items;
        DROP TABLE IF EXISTS balance_categories;
        DROP TABLE IF EXISTS balance_timeline",
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
    conn.execute(
        "CREATE TABLE IF NOT EXISTS balance_timeline (
                timestamp INTEGER NOT NULL,
                username_lower TEXT NOT NULL,
                PRIMARY KEY (username_lower)
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
