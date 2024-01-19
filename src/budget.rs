use crate::structs_utils::*;
use chrono::prelude::*;
use rusqlite::{Connection, Result};

#[derive(Debug, PartialEq)]
enum BudgetSelection<'a> {
    Some(&'a BudgetItem),
    NewCategory,
    NewItem,
    RenameCategory,
    GoBack,
}

pub fn budget_half_entry_point(conn: &Connection, user: &User, which_half: BudgetHalf) {
    initialize_budget(conn, user);
    let (mut categories, mut items) = get_relevant_items(conn, user, &which_half)
        .expect("There was an error accessing the Budget Database");
    loop {
        let response = print_budget_half_get_response(&categories, &items, &which_half);

        match response {
            BudgetSelection::Some(item) => {
                let idx = items
                    .iter()
                    .position(|i| i == item)
                    .expect("Error sending this item for updating");
                update_item(conn, user, &which_half, &mut categories, &mut items, idx);
            }
            BudgetSelection::NewCategory => {
                create_new_category(conn, user, &which_half, &mut categories)
            }
            BudgetSelection::NewItem => {
                create_new_item(conn, user, &which_half, &mut categories, &mut items)
            }
            BudgetSelection::RenameCategory => {
                rename_category(conn, user, &which_half, &mut categories)
            }
            BudgetSelection::GoBack => return,
        }
    }
}

pub fn budget_whole_entry_point(conn: &Connection, user: &User) {
    initialize_budget(conn, user);
    let (income_categories, income_items) = get_relevant_items(conn, user, &BudgetHalf::Income)
        .expect("There was an error accessing the Budget Income from the Database");
    let (expense_categories, expense_items) = get_relevant_items(conn, user, &BudgetHalf::Expenses)
        .expect("There was an error accessing the Budget Expenses from the Database");

    // Response is irrelevant here
    print_budget_get_response(
        &income_categories,
        &income_items,
        &expense_categories,
        &expense_items,
    );
}

/// Print out the half of the budget and find out what the user wants to do
/// It only receives the relevant half categories and items
fn print_budget_half_get_response<'a>(
    categories: &Vec<BudgetCategory>,
    items: &'a Vec<BudgetItem>,
    which_half: &BudgetHalf,
) -> BudgetSelection<'a> {
    println!("\nCurrent list of {}:", which_half.to_str().to_lowercase());
    let mut idx: usize = 1;
    let mut sorted_items: Vec<&BudgetItem> = vec![];

    for category in categories {
        // Check if any of the items are in this category
        let mut no_items_found_in_cat = true;
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
            if item.category_lower == category.category_lower {
                print!("    {}. {} ", idx, item.item);
                let mut num_dashes: usize = MAX_CHARACTERS_ITEM_NAME + 1 - item.item.len();
                // Support prooper formatting up to 999 items
                if idx >= 10 {
                    num_dashes -= 1;
                }
                if idx >= 100 {
                    num_dashes -= 1;
                }
                for _ in 0..num_dashes {
                    print!("-");
                }
                println!(" {}", to_money_string(item.value));
                idx += 1;
                sorted_items.push(item);
            }
        }
    }
    println!("\n{}. NEW CATEGORY", idx);
    idx += 1;
    println!("    {}. NEW ITEM", idx);
    idx += 1;
    println!("{}. RENAME CATEGORY", idx);
    println!("\n 0. GO BACK - Budget Menu");
    let response = print_instr_get_response(0, idx, || {
        println!("\nEnter the number of the item you'd like to update / delete, or one of the other numbers");
    });
    match response {
        0 => BudgetSelection::GoBack,
        x if x > 0 && x <= idx - 3 => BudgetSelection::Some(sorted_items.remove(x - 1)),
        x if x == idx - 2 => BudgetSelection::NewCategory,
        x if x == idx - 1 => BudgetSelection::NewItem,
        x if x == idx => BudgetSelection::RenameCategory,
        x => panic!("Response {} is an error state. Exiting the program.", x),
    }
}

/// Print out the whole budget
fn print_budget_get_response(
    income_categories: &Vec<BudgetCategory>,
    income_items: &Vec<BudgetItem>,
    expense_categories: &Vec<BudgetCategory>,
    expense_items: &Vec<BudgetItem>,
) -> String {
    let today_date = Local::now().format("%Y-%m-%d").to_string();
    println!("\n\nCurrent Monthly Budget - {}", today_date);
    println!("\nINCOME");
    let mut income_total: f64 = 0.0;
    for category in income_categories {
        // Check if any of the items are in this category
        let mut no_items_found_in_cat = true;
        for item in income_items {
            if item.category_lower == category.category_lower {
                no_items_found_in_cat = false;
            }
        }
        if no_items_found_in_cat {
            continue; // Don't need to print this category if it has no items
        }
        println!("{}", category.category);
        for item in income_items {
            if item.category_lower == category.category_lower {
                print!("    {} ", item.item);
                let num_dashes: usize = MAX_CHARACTERS_ITEM_NAME + 4 - item.item.len();
                for _ in 0..num_dashes {
                    print!("-");
                }
                println!(" {}", to_money_string(item.value));
                income_total += item.value;
            }
        }
    }
    // Print sum
    for _ in 0..(MAX_CHARACTERS_ITEM_NAME + 24) {
        print!("_");
    }
    print!("\nTotal Income");
    for _ in 0..(MAX_CHARACTERS_ITEM_NAME - 2) {
        print!(" ");
    }
    println!("{}", to_money_string(income_total));

    println!("\nEXPENSES");
    let mut expense_total: f64 = 0.0;
    for category in expense_categories {
        // Check if any of the items are in this category
        let mut no_items_found_in_cat = true;
        for item in expense_items {
            if item.category_lower == category.category_lower {
                no_items_found_in_cat = false;
            }
        }
        if no_items_found_in_cat {
            continue; // Don't need to print this category if it has no items
        }
        println!("{}", category.category);
        for item in expense_items {
            if item.category_lower == category.category_lower {
                print!("    {} ", item.item);
                let num_dashes: usize = MAX_CHARACTERS_ITEM_NAME + 4 - item.item.len();
                for _ in 0..num_dashes {
                    print!("-");
                }
                println!(" {}", to_money_string(item.value));
                expense_total += item.value;
            }
        }
    }
    // Print sum
    for _ in 0..(MAX_CHARACTERS_ITEM_NAME + 24) {
        print!("_");
    }
    print!("\nTotal Expenses");
    for _ in 0..(MAX_CHARACTERS_ITEM_NAME - 4) {
        print!(" ");
    }
    println!("{}", to_money_string(expense_total));

    // Grand total
    let total = income_total - expense_total;
    println!(
        "\n\nTOTAL MONTHLY NET ------------------  {}",
        to_money_string(total)
    );

    // Return
    println!("\n\nPress enter to return.");
    read_or_quit()
}

/// Category Creator
/// Mutates the categories Vector and updates the DB
fn create_new_category(
    conn: &Connection,
    user: &User,
    which_half: &BudgetHalf,
    categories: &mut Vec<BudgetCategory>,
) {
    println!("What would you like to call the new category?");
    let cat_name = read_or_quit();

    // Return if the new name is empty
    if cat_name.is_empty() {
        println!("The category name cannot be empty. Please try again.");
        println!("Hit Enter to go back.");
        // Give the user a chance to acknowledge
        read_or_quit();
        return;
    }

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
        "INSERT INTO budget_categories (category, category_lower, username_lower, is_income) VALUES (?1, ?2, ?3, ?4)",
        (&cat_name, &cat_name.to_lowercase(), &user.username_lower, &which_half.to_bool_int()),
    ).expect("Error creating new category");
    // Add the category into the categories vector
    categories.push(BudgetCategory {
        category: cat_name.clone(),
        category_lower: cat_name.to_lowercase(),
        username_lower: String::from(&user.username_lower),
        is_income: which_half.to_bool(),
    });
}

/// Item Creator
/// Mutates that categories and items Vectors and updates the DB
fn create_new_item(
    conn: &Connection,
    user: &User,
    which_half: &BudgetHalf,
    categories: &mut Vec<BudgetCategory>,
    items: &mut Vec<BudgetItem>,
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
    // Enforce MAX_CHARACTERS_ITEM_NAME character maximum
    if item_name.len() > MAX_CHARACTERS_ITEM_NAME {
        println!(
            "There is currently a {} character limit on the item name. Please try again.",
            MAX_CHARACTERS_ITEM_NAME
        );
        return;
    }

    // Return if the new name is empty
    if item_name.is_empty() {
        println!("The item name cannot be empty. Please try again.");
        println!("Hit Enter to go back.");
        // Give the user a chance to acknowledge
        read_or_quit();
        return;
    }

    // Get the new item's value
    if which_half.to_bool() {
        // is_income
        println!("What is the average monthly income associated with this item?");
    } else {
        println!(
            "What is the average monthly cost associated with this item? (positive number or 0)"
        );
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
    let timeline: usize = get_and_update_timeline(conn, user);

    // Insert the new item into the database
    conn.execute(
        "INSERT INTO budget_items 
        (item, item_lower, value, category, category_lower, username_lower, 
            is_income, timeline_created, timeline_original, is_deleted, timeline_deleted) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        (
            &item_name,
            &item_name.to_lowercase(),
            &value,
            &chosen_cat,
            &chosen_cat.to_lowercase(),
            &user.username_lower,
            &which_half.to_bool_int(),
            &timeline,
            &timeline,
            0,
            usize::MAX / 4,
        ),
    )
    .expect("Error creating new item");
    // Add the item into the items vector
    items.push(BudgetItem {
        item: item_name.clone(),
        item_lower: item_name.to_lowercase(),
        value,
        category: chosen_cat.clone(),
        category_lower: chosen_cat.to_lowercase(),
        username_lower: String::from(&user.username_lower),
        is_income: which_half.to_bool(),
        timeline_created: timeline,
        timeline_original: timeline,
        is_deleted: false,
        timeline_deleted: usize::MAX / 4,
    });
}

/// Rename a category
/// Mutates the categories Vector and updates the DB
fn rename_category(
    conn: &Connection,
    user: &User,
    which_half: &BudgetHalf,
    categories: &mut Vec<BudgetCategory>,
) {
    println!("\nRename a category ({}):", which_half.to_str());
    let mut idx: usize = 0;

    for category in &mut *categories {
        if category.is_income == which_half.to_bool() {
            idx += 1;
            println!("{}. {}", idx, category.category);
        }
    }
    println!("\n0. GO BACK");
    let response = print_instr_get_response(0, idx, || {
        println!("Enter the number of the category you'd like to rename.");
    });
    match response {
        0 => (), // return
        x if x > 0 && x <= idx => {
            println!(
                "What would you like to rename {}?",
                categories
                    .get(x - 1)
                    .expect("Unable to access chosen category")
                    .category
                    .clone()
            );
            let new_name = read_or_quit();

            // Return if the new name is empty
            if new_name.is_empty() {
                println!("The category name cannot be empty. Please try again.");
                println!("Hit Enter to go back.");
                // Give the user a chance to acknowledge
                read_or_quit();
                return;
            }

            // Check if it exists already, and return if it does
            for category in &mut *categories {
                if new_name.to_ascii_lowercase() == category.category_lower {
                    println!("That name is already in use as {}.", category.category);
                    println!("You cannot rename this category to the same name.");
                    return;
                }
            }
            // Update the Vector and DB
            let old_cat_name_lower = String::from(&mut *categories[x - 1].category_lower);
            categories[x - 1].category = new_name.clone();
            categories[x - 1].category_lower = new_name.to_ascii_lowercase();

            conn.execute(
                "UPDATE budget_categories 
                SET category = ?1, category_lower = ?2 
                WHERE username_lower = ?3 AND category = ?4 AND is_income = ?5",
                (
                    &new_name,
                    &new_name.to_ascii_lowercase(),
                    &user.username_lower,
                    old_cat_name_lower,
                    &which_half.to_bool_int(),
                ),
            )
            .expect("Error updating the budget categories database");
        }
        x => panic!("Response {} is an error state. Exiting the program.", x),
    }
}

/// Item Update or Delete
/// Mutates that categories and items Vectors and updates the DB
fn update_item(
    conn: &Connection,
    user: &User,
    which_half: &BudgetHalf,
    categories: &mut Vec<BudgetCategory>,
    items: &mut Vec<BudgetItem>,
    idx: usize,
) {
    // Need to remove the item from the vector to get ownership
    let item_chosen = items.remove(idx);
    println!("\nUpdate item {}", item_chosen.item);
    println!("\nWould you like to update or delete it?");
    println!("1. Update");
    println!("2. Delete");
    println!("0. GO BACK");
    let response = print_instr_get_response(0, 2, || {});
    match response {
        0 => {
            // Must push the item back into the vector
            items.push(item_chosen);
            // return
        }
        2 => {
            println!("Are you sure you'd like to delete this item? This cannot be undone.");
            println!("1. Yes");
            println!("2. No (Go back)");
            match print_instr_get_response(1, 2, || {}) {
                1 => {
                    // Delete the item from the database and from the mutable vector
                    let timeline: usize = get_and_update_timeline(conn, user);
                    conn.execute(
                        "UPDATE budget_items 
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
                    // return
                }
                2 => {
                    // Must push the item back into the vector
                    items.push(item_chosen);
                    // return
                }
                x => panic!("Response {} is an error state. Exiting the program.", x),
            }
        }
        1 => {
            // Mark current one as deleted with proper timestamp
            // Create new one with proper timestamp
            // First get the new item name
            println!(
                "If you would like to change the item's name from {}, what would you like to change it to?", item_chosen.item
            );
            println!("Or just leave this blank to keep it the same. (Just hit Enter)");
            let mut item_name = read_or_quit();
            // Check if the item already exists
            for item in &mut *items {
                if item_name.to_lowercase() == item.item_lower {
                    println!("This item already exists as {}.", item.item);
                    println!("If you would like to edit that item, select it on the next page.");
                    return;
                }
            }
            if item_name.is_empty() {
                // They would like to not change the item name
                item_name = String::from(&item_chosen.item);
            }

            // Get the new item's value
            if which_half.to_bool() {
                // is_income
                println!(
                    "The current monthly income associated with {} is listed as {}. Enter a new value to change it.",
                    item_name, to_money_string(item_chosen.value)
                );
            } else {
                println!(
                    "The current monthly cost associated with {} is listed as {}. Enter a new cost to change it (positive number or 0).",
                    item_name, to_money_string(item_chosen.value)
                );
            }
            println!("Or just leave it blank and hit Enter to keep the value the same.");
            let mut value: f64 = -1.0;
            while value < 0.0 {
                let val_response = read_or_quit();
                if val_response.is_empty() {
                    // The user would like to not change the value
                    value = item_chosen.value;
                    break;
                }
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
            println!(
                "The current category for {} is {}.",
                item_name, item_chosen.category
            );
            println!("Select a new category, make a new category, or Enter a \"0\" (zero) to keep the same category.");
            while still_need_category {
                println!("\nWhich category would you like to use for this item?");
                let mut idx: usize = 1;
                println!("0. NO CHANGE");
                for category in &mut *categories {
                    println!("{}. {}", idx, category.category);
                    idx += 1;
                }
                println!("\n{}. NEW CATEGORY", idx);
                let response = print_instr_get_response(0, idx, || {});
                match response {
                    0 => {
                        // Same category
                        chosen_cat = String::from(&item_chosen.category);
                        still_need_category = false;
                    }
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

            // Get the timeline_deleted value for the former version of the item
            let timeline: usize = get_and_update_timeline(conn, user);

            // Mark the former version of the item as deleted
            conn.execute(
                "UPDATE budget_items 
                SET is_deleted = 1, timeline_deleted = ?1
                WHERE item_lower = ?2 AND username_lower = ?3 AND timeline_created = ?4",
                (
                    &timeline,
                    &item_chosen.item_lower,
                    &user.username_lower,
                    &item_chosen.timeline_created,
                ),
            )
            .expect("Error marking the former version of the item as deleted");

            // Get the timeline_created value for the fupdated version of the item
            let timeline: usize = get_and_update_timeline(conn, user);

            // Insert the new item into the database
            conn.execute(
                "INSERT INTO budget_items 
                (item, item_lower, value, category, category_lower, username_lower, 
                    is_income, timeline_created, timeline_original, is_deleted, timeline_deleted) 
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                (
                    &item_name,
                    &item_name.to_lowercase(),
                    &value,
                    &chosen_cat,
                    &chosen_cat.to_lowercase(),
                    &user.username_lower,
                    &which_half.to_bool_int(),
                    &timeline,
                    &item_chosen.timeline_original,
                    0,
                    usize::MAX / 4,
                ),
            )
            .expect("Error creating new item");
            // Add the updated item into the items vector (since the old one was already removed)
            items.push(BudgetItem {
                item: item_name.clone(),
                item_lower: item_name.to_lowercase(),
                value,
                category: chosen_cat.clone(),
                category_lower: chosen_cat.to_lowercase(),
                username_lower: String::from(&user.username_lower),
                is_income: which_half.to_bool(),
                timeline_created: timeline,
                timeline_original: item_chosen.timeline_original,
                is_deleted: false,
                timeline_deleted: usize::MAX / 4,
            });
        }
        x => panic!("Response {} is an error state. Exiting the program.", x),
    }
}

/// Get the relevant half of the budget
fn get_relevant_items(
    conn: &Connection,
    user: &User,
    which_half: &BudgetHalf,
) -> Result<(Vec<BudgetCategory>, Vec<BudgetItem>)> {
    // Push all of the categories to a vector first
    let mut categories: Vec<BudgetCategory> = vec![];
    let mut stmt =
        conn.prepare("SELECT * FROM budget_categories WHERE is_income=?1 AND username_lower=?2")?;
    let mut rows = stmt.query(rusqlite::params![
        which_half.to_bool_int(),
        user.username_lower
    ])?;
    while let Some(row) = rows.next()? {
        categories.push(BudgetCategory {
            category: row.get(0)?,
            category_lower: row.get(1)?,
            username_lower: row.get(2)?,
            is_income: row.get(3)?,
        })
    }
    // Next push all of the active items to a vector
    let mut items: Vec<BudgetItem> = vec![];
    let mut stmt = conn.prepare(
        "SELECT * FROM budget_items WHERE is_deleted=0 AND is_income=?1 AND username_lower=?2",
    )?;
    let mut rows = stmt.query(rusqlite::params![
        which_half.to_bool_int(),
        user.username_lower
    ])?;
    while let Some(row) = rows.next()? {
        items.push(BudgetItem {
            item: row.get(0)?,
            item_lower: row.get(1)?,
            value: row.get(2)?,
            category: row.get(3)?,
            category_lower: row.get(4)?,
            username_lower: row.get(5)?,
            is_income: row.get(6)?,
            timeline_created: row.get(7)?,
            timeline_original: row.get(8)?,
            is_deleted: row.get(9)?,
            timeline_deleted: row.get(10)?,
        })
    }
    Ok((categories, items))
}

/// Set up the tables for the budget for this user
fn initialize_budget(conn: &Connection, user: &User) {
    // Create the budget_categories table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS budget_categories (
            category TEXT NOT NULL,
            category_lower TEXT NOT NULL,
            username_lower TEXT NOT NULL,
            is_income INTEGER NOT NULL,
            PRIMARY KEY (category_lower, username_lower, is_income),
            FOREIGN KEY (username_lower) REFERENCES users (username_lower)
        )",
        (),
    )
    .expect("Error connecting with the budget categories table");

    // Add an Uncategorized type to the table for income if it's not there yet
    conn.execute(
        "INSERT OR IGNORE INTO budget_categories 
        (category, category_lower, username_lower, is_income) 
        VALUES (\"Uncategorized\", \"uncategorized\", ?1, 1)",
        rusqlite::params![&user.username_lower],
    )
    .expect("Error initializing the budget_categories table");

    // Add an Uncategorized type to the table for expenses if it's not there yet
    conn.execute(
        "INSERT OR IGNORE INTO budget_categories 
        (category, category_lower, username_lower, is_income) 
        VALUES (\"Uncategorized\", \"uncategorized\", ?1, 0)",
        rusqlite::params![&user.username_lower],
    )
    .expect("Error initializing the budget_categories table");

    // Create the budget_items table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS budget_items (
                item TEXT NOT NULL,
                item_lower TEXT NOT NULL,
                value REAL NOT NULL,
                category TEXT NOT NULL,
                category_lower TEXT NOT NULL,
                username_lower TEXT NOT NULL,
                is_income INTEGER NOT NULL,
                timeline_created INTEGER NOT NULL,
                timeline_original INTEGER NOT NULL,
                is_deleted INTEGER NOT NULL,
                timeline_deleted INTEGER NOT NULL,
                PRIMARY KEY (item_lower, username_lower, timeline_created),
                FOREIGN KEY (username_lower) REFERENCES users (username_lower),
                FOREIGN KEY (category_lower, username_lower, is_income) REFERENCES budget_categories (category_lower, username_lower, is_income)
            );",
        (),
    )
    .expect("Error connecting with the budget items table");

    // Create the timeline table to persist a timeline value per user
    // This stores an incrementing integer to demarcate a timeline
    // The timeline helps differentiate current a past values while avoiding unneccessary dates
    // It stores the most recently used integer. It should be pulled from the DB, incremented, used, then returned.
    // I'm not sure if this will be useful for the budget, but will keep it in place for now
    conn.execute(
        "CREATE TABLE IF NOT EXISTS budget_timeline (
                timestamp INTEGER NOT NULL,
                username_lower TEXT NOT NULL,
                PRIMARY KEY (username_lower)
            );",
        (),
    )
    .expect("Error connecting with the budget timeline table");

    // Initialize the timeline to 0 for this user if it's not there yet
    conn.execute(
        "INSERT OR IGNORE INTO budget_timeline (timestamp, username_lower) VALUES (0, ?1)",
        rusqlite::params![&user.username_lower],
    )
    .expect("Error initializing the budget_timeline table");
}

/// Gets the timeline from the database and returns it ALREADY INCREMENTED and ready to use
/// It also updates the value in the timeline database
fn get_and_update_timeline(conn: &Connection, user: &User) -> usize {
    // Get the new item's timeline_created value and increment it
    let timeline: usize;
    let mut stmt = conn
        .prepare("SELECT timestamp FROM budget_timeline WHERE username_lower = ?1")
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
                "UPDATE budget_timeline SET timestamp = ?1 WHERE username_lower = ?2",
                (timeline, &user.username_lower),
            )
            .expect("Error updating the timeline database");
            timeline
        }
        // Timeline not found. This is an error state as this needs to be created during initialization.
        None => {
            panic!("Timeline query returned empty");
        }
    }
}
