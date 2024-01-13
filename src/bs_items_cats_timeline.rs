use crate::structs_utils::*;
use rusqlite::Connection;

/// Category Creator
/// Mutates the categories Vector and updates the DB
pub fn create_new_category(
    conn: &Connection,
    user: &User,
    which_half: &BalanceSheetHalf,
    categories: &mut Vec<Category>,
) {
    println!("What would you like to call the new category?");
    let cat_name = read_or_quit();
    // Can't have zero length
    if cat_name.len() == 0 {
        println!("Cannot have an empty name. Press Enter to go back");
        read_or_quit(); // Allow user to acknowledge
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
/// Mutates that categories and items Vectors and updates the DB
pub fn create_new_item(
    conn: &Connection,
    user: &User,
    which_half: &BalanceSheetHalf,
    categories: &mut Vec<Category>,
    items: &mut Vec<Item>,
) {
    println!("What would you like to name the new item?");
    // Get The new item's name
    let item_name = read_or_quit();
    // Can't have zero length
    if item_name.len() == 0 {
        println!("Cannot have an empty name. Press Enter to go back");
        read_or_quit(); // Allow user to acknowledge
        return;
    }
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
    let timeline: usize = get_and_update_timeline(conn, user);

    // Insert the new item into the database
    conn.execute(
        "INSERT INTO balance_items 
        (item, item_lower, value, category, category_lower, username_lower, 
            is_asset, timeline_created, timeline_original, is_deleted, timeline_deleted) 
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
    items.push(Item {
        item: item_name.clone(),
        item_lower: String::from(item_name.to_lowercase()),
        value: value,
        category: chosen_cat.clone(),
        category_lower: String::from(chosen_cat.to_lowercase()),
        username_lower: String::from(&user.username_lower),
        is_asset: which_half.to_bool(),
        timeline_created: timeline,
        timeline_original: timeline,
        is_deleted: false,
        timeline_deleted: usize::MAX / 4,
    });
}

/// Rename a category
/// Mutates the categories Vector and updates the DB
pub fn rename_category(
    conn: &Connection,
    user: &User,
    which_half: &BalanceSheetHalf,
    categories: &mut Vec<Category>,
) {
    println!("\nRename a category ({}):", which_half.to_str());
    let mut idx: usize = 0;

    for category in &mut *categories {
        if category.is_asset == which_half.to_bool() {
            idx += 1;
            println!("{}. {}", idx, category.category);
        }
    }
    println!("\n0. GO BACK");
    let response = print_instr_get_response(0, idx, || {
        println!("Enter the number of the category you'd like to rename.");
    });
    match response {
        0 => return,
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
                "UPDATE balance_categories 
                SET category = ?1, category_lower = ?2 
                WHERE username_lower = ?3 AND category = ?4 AND is_asset = ?5",
                (
                    &new_name,
                    &new_name.to_ascii_lowercase(),
                    &user.username_lower,
                    old_cat_name_lower,
                    &which_half.to_bool_int(),
                ),
            )
            .expect("Error updating the timeline database");
        }
        x => panic!("Response {} is an error state. Exiting the program.", x),
    }
}

/// Item Update or Delete
/// Mutates that categories and items Vectors and updates the DB
pub fn update_item(
    conn: &Connection,
    user: &User,
    which_half: &BalanceSheetHalf,
    categories: &mut Vec<Category>,
    items: &mut Vec<Item>,
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
            return;
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
                    items.push(item_chosen);
                    return;
                }
                x => panic!("Response {} is an error state. Exiting the program.", x),
            }
        }
        1 => {
            // Mark current one as deleted with proper timestamp
            // Create new one with proper timestamp
            // Get The new item's name
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
            if item_name.len() == 0 {
                // They would like to not change the item name
                item_name = String::from(&item_chosen.item);
            }

            // Get the new item's value
            if which_half.to_bool() {
                // is_asset
                println!(
                    "The current value of {} is listed as {}. Enter a new value to change it.",
                    item_name, item_chosen.value
                );
            } else {
                println!(
                    "The current cost of {} is listed as {}. Enter a new liability cost to change it (positive number or 0).",
                    item_name, item_chosen.value
                );
            }
            println!("Or just leave it blank and hit Enter to keep the value the same.");
            let mut value: f64 = -1.0;
            while value < 0.0 {
                let val_response = read_or_quit();
                if val_response.len() == 0 {
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
            .expect("Error marking the former version of the item as deleted");

            // Get the timeline_created value for the fupdated version of the item
            let timeline: usize = get_and_update_timeline(conn, user);

            // Insert the new item into the database
            conn.execute(
                "INSERT INTO balance_items 
                (item, item_lower, value, category, category_lower, username_lower, 
                    is_asset, timeline_created, timeline_original, is_deleted, timeline_deleted) 
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
            items.push(Item {
                item: item_name.clone(),
                item_lower: String::from(item_name.to_lowercase()),
                value: value,
                category: chosen_cat.clone(),
                category_lower: String::from(chosen_cat.to_lowercase()),
                username_lower: String::from(&user.username_lower),
                is_asset: which_half.to_bool(),
                timeline_created: timeline,
                timeline_original: item_chosen.timeline_original,
                is_deleted: false,
                timeline_deleted: usize::MAX / 4,
            });
        }
        x => panic!("Response {} is an error state. Exiting the program.", x),
    }
}

/// Gets the timeline from the database and returns it ALREADY INCREMENTED and ready to use
/// It also updates the value in the timeline database
pub fn get_and_update_timeline(conn: &Connection, user: &User) -> usize {
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
            timeline
        }
        // Timeline not found. This is an error state as this needs to be created during initialization.
        None => {
            panic!("Timeline query returned empty");
        }
    }
}
