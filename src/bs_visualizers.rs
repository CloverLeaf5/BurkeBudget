use crate::structs_utils::*;
use chrono::Duration;
use rusqlite::{Connection, Result};
use textplots::{Chart, LabelBuilder, LabelFormat, Plot, Shape, TickDisplay, TickDisplayBuilder};

/// Select which type of visualization user would like
pub fn snapshot_visualizer_menu(conn: &Connection, user: &User) {
    loop {
        println!("\n\nTrend Analysis - How would you like to visualize your snapshots?");
        println!("1. Side-By-Side Comparison");
        println!("2. Net Worth Graph Over Time");
        println!("\n0. GO BACK");

        let response = print_instr_get_response(0, 2, || {});
        match response {
            0 => return,
            1 => side_by_side_snapshots(conn, user).expect("Error getting the snapshots"),
            2 => net_worth_graph(conn, user).expect("Error getting the snapshots"),
            x => panic!("Response {} is an error state. Exiting the program.", x),
        }
    }
}

/// Allows the user to select which snapshots they would like to view side-by-side
fn side_by_side_snapshots(conn: &Connection, user: &User) -> Result<()> {
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

    if snapshots.is_empty() {
        println!("\n\nYou don't have any saved snapshots yet. Hit Enter to go back.");
        read_or_quit(); // Just to give the user a chance to acknowledge
        return Ok(());
    }

    // Sort the snapshots in chronological order
    snapshots.sort_by(|a, b| a.timeline.cmp(&b.timeline));

    // Print out listing of snapshots
    println!("\n\nSide-By-Side Comparison");
    println!("Select up to 5 snapshots to view side-by-side");
    println!("\nSaved Snapshots:");
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
    println!("\nSelect snapshots by entering their numbers separated by a space (eg - 1 5 6)");

    let response = read_or_quit();
    if response == "0" {
        return Ok(());
    }
    match parse_space_delim_response_for_int_to_index(response, 1, idx) {
        Some(selected_indices) => print_side_by_side(conn, user, snapshots, selected_indices),
        None => {
            println!("\nNo correct inputs were given. Please try again.");
            return Ok(());
        }
    }

    Ok(())
}

/// Print out the side by side comparison, then let the user return
fn print_side_by_side(
    conn: &Connection,
    user: &User,
    snapshots: Vec<Snapshot>,
    mut selected_indices: Vec<usize>,
) {
    const COL_WIDTH: usize = 20;
    selected_indices.sort();

    // Get all of the items and categories for this user
    let mut asset_items = get_all_items(conn, user, &BalanceSheetHalf::Assets)
        .expect("There was an error accessing the Balance Sheet Assets from the Database");
    let mut liability_items = get_all_items(conn, user, &BalanceSheetHalf::Liabilities)
        .expect("There was an error accessing the Balance Sheet Liabilities from the Database");

    // Sort the assets and liabilities by timeline original then secondarily timeline created
    // This will sort unique items based on the unchanging timeline original then put them in order of updates
    asset_items.sort_unstable_by_key(|a| (a.timeline_original, a.timeline_created));
    liability_items.sort_unstable_by_key(|a| (a.timeline_original, a.timeline_created));

    // Print the dates of the snapshots
    print!("\n\n{}", " ".repeat(MAX_CHARACTERS_ITEM_NAME + 2));
    for selected_index in &selected_indices {
        print!(
            "{}{}",
            snapshots[*selected_index].date_today,
            " ".repeat(COL_WIDTH - 10 + 2)
        );
    }
    println!();

    // This code is difficult to maintain and is not readable
    // This could be improved by breaking logic into functions
    //   and by creating a new struct for items that include timeline timestamps
    //   when they are read from the database

    //////////// ASSETS //////////////////////////////
    println!("ASSETS");
    let asset_totals =
        helper_print_half_side_by_side(&snapshots, &selected_indices, &asset_items, COL_WIDTH);

    // Print Asset totals
    print!("{}", "_".repeat(MAX_CHARACTERS_ITEM_NAME + 2));
    for _ in 0..selected_indices.len() {
        print!("{}", "_".repeat(COL_WIDTH + 2));
    }
    println!();
    print!(
        "TOTAL ASSETS {} ",
        " ".repeat(MAX_CHARACTERS_ITEM_NAME - 12)
    );
    for asset_total in &asset_totals {
        let money_len = to_money_string(*asset_total).len();
        print!(
            "{}{}",
            to_money_string(*asset_total),
            " ".repeat(COL_WIDTH - money_len + 2)
        );
    }
    print!("\n\n");

    //////////// LIABILITIES //////////////////////////////
    println!("LIABILITIES");
    let liability_totals =
        helper_print_half_side_by_side(&snapshots, &selected_indices, &liability_items, COL_WIDTH);

    // Print Liability totals
    print!("{}", "_".repeat(MAX_CHARACTERS_ITEM_NAME + 2));
    for _ in 0..selected_indices.len() {
        print!("{}", "_".repeat(COL_WIDTH + 2));
    }
    println!();
    print!(
        "TOTAL LIABILITIES {} ",
        " ".repeat(MAX_CHARACTERS_ITEM_NAME - 17)
    );
    for liability_total in &liability_totals {
        let money_len = to_money_string(*liability_total).len();
        print!(
            "{}{}",
            to_money_string(*liability_total),
            " ".repeat(COL_WIDTH - money_len + 2)
        );
    }
    print!("\n\n");

    ///////  GRAND TOTALS  ////////////////////////////////
    // Print Grand totals
    print!("{}", "_".repeat(MAX_CHARACTERS_ITEM_NAME + 2));
    for _ in 0..selected_indices.len() {
        print!("{}", "_".repeat(COL_WIDTH + 2));
    }
    println!();
    print!(
        "TOTAL NET WORTH {} ",
        " ".repeat(MAX_CHARACTERS_ITEM_NAME - 15)
    );
    for i in 0..selected_indices.len() {
        let grand_total = asset_totals[i] - liability_totals[i];
        let money_len = to_money_string(grand_total).len();
        print!(
            "{}{}",
            to_money_string(grand_total),
            " ".repeat(COL_WIDTH - money_len + 2)
        );
    }
    print!("\n\n");
}

/// Helper function to print one half of the side-by-side visualizer
/// Returns the sum totals for each snapshot (half) as a Vector
fn helper_print_half_side_by_side(
    snapshots: &[Snapshot],
    selected_indices: &[usize],
    items: &[Item],
    col_width: usize,
) -> Vec<f64> {
    // Running total of printed values matching snapshot by index
    let mut totals: Vec<f64> = vec![0.0; selected_indices.len()];

    // Loop through the items and print a new one every time it's crossed
    let mut prev_item_origin: usize = usize::MAX;
    for (idx, item) in items.iter().enumerate() {
        if item.timeline_original == prev_item_origin {
            // This item name is already printed
            continue;
        }
        // This is a new item to be examined
        prev_item_origin = item.timeline_original;
        // Check if this item is involved in snapshots. Don't print if not
        // Must get timeline_origin and the last iteration of this item's timeline_deleted
        // Then see if any of the snapshot timelines fall into this range
        let mut idx_offset: usize = 0;
        // Traverse to the last iteration of this item
        loop {
            // Check if next index exists
            if idx + idx_offset + 1 < items.len() {
                // Check if it is the same item
                if items[idx + idx_offset + 1].timeline_original == prev_item_origin {
                    // Increment idx_offset
                    idx_offset += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        let first_created = &items[idx].timeline_original;
        let last_deleted = &items[idx + idx_offset].timeline_deleted;
        // Check if any of the timelines fall in this range
        let mut never_used: bool = true;
        for selected_index in selected_indices {
            let current_timeline = &snapshots[*selected_index].timeline;
            if current_timeline >= first_created && current_timeline < last_deleted {
                never_used = false;
            }
        }
        if never_used {
            continue;
        }

        // If the code gets here, then the item is used an should be printed
        // Print the most recent version of the name by using the index and offset
        print!(
            "{} {} ",
            items[idx + idx_offset].item,
            "-".repeat(MAX_CHARACTERS_ITEM_NAME - items[idx + idx_offset].item.len())
        );

        // After printing item name, must print value for each snapshot
        // Subsequent items in the vector will be other instances of this item if needed
        for (col, selected_index) in selected_indices.iter().enumerate() {
            let current_timeline = snapshots[*selected_index].timeline;
            if item.timeline_created <= current_timeline && item.timeline_deleted > current_timeline
            {
                // Correct value, print it here and add to running total
                print!("{}", to_money_string(item.value));
                totals[col] += item.value;
                // If this isn't the last column, print more dashes, otherwise new line
                let money_len = to_money_string(item.value).len();
                if col < selected_indices.len() - 1 {
                    print!(" {} ", "-".repeat(col_width - money_len));
                } else {
                    println!();
                }
            } else if item.timeline_created > current_timeline {
                // This item was created after this timeline point
                // Print a placeholder
                print!(" {} ", "-".repeat(col_width));
                // New line if this is the last column
                if col == selected_indices.len() - 1 {
                    println!();
                }
            } else {
                // This item was deleted before this timeline point
                let mut offset: usize = 0;
                loop {
                    // Step through the vector trying to find a later version of this same item
                    offset += 1;
                    // Make sure this is a valid index
                    if idx + offset < items.len() {
                        let item_to_check = &items[idx + offset];
                        if item_to_check.timeline_original != prev_item_origin {
                            // Not the same item so just print dashes and move on
                            print!(" {} ", "-".repeat(col_width));
                            // New line if this is the last column
                            if col == selected_indices.len() - 1 {
                                println!();
                            }
                            break;
                        } else {
                            // Same item, check again if the timeline matches. If not, just try on next one via loop
                            if item_to_check.timeline_created <= current_timeline
                                && item_to_check.timeline_deleted > current_timeline
                            {
                                // Can print its value here
                                print!("{}", to_money_string(item_to_check.value));
                                totals[col] += item_to_check.value;
                                // If this isn't the last column, print more dashes, otherwise new line
                                let money_len = to_money_string(item_to_check.value).len();
                                if col < selected_indices.len() - 1 {
                                    print!(" {} ", "-".repeat(col_width - money_len));
                                } else {
                                    println!();
                                }
                                // Break out of the loop since the correct item was found
                                break;
                            }
                        }
                    } else {
                        // End of vector, print dashes
                        print!(" {} ", "-".repeat(col_width));
                        // New line if this is the last column
                        if col == selected_indices.len() - 1 {
                            println!();
                        }
                        break;
                    }
                }
            }
        }
    }
    totals
}

/// Get the first five numbers delimited by spaces within the specified range
/// Used with the side-by-side snapshots viewer
/// NOTE: This converts the number from the number listed to an index by subtracting 1
fn parse_space_delim_response_for_int_to_index(
    response: String,
    minval: usize,
    maxval: usize,
) -> Option<Vec<usize>> {
    let vals_str: Vec<&str> = response.split(' ').collect();
    let mut vals: Vec<usize> = vec![];
    for val in vals_str {
        match val.parse::<usize>() {
            Ok(parsed) => {
                // There is a number here
                if parsed >= minval && parsed <= maxval && !vals.contains(&(parsed - 1)) {
                    // This is a correct number, subtract one to convert input number to index in snapshots vector
                    vals.push(parsed - 1);
                } else if parsed >= minval && parsed <= maxval && vals.contains(&(parsed - 1)) {
                    // This value is already in the vals vector
                    println!("\n{} was a duplicate. It only can be entered once.", val);
                    continue;
                } else {
                    // This is a number but outside the acceptable range
                    println!("\n{} wasn't included as it was an invalid response.", val);
                    continue;
                }
            }
            Err(_) => {
                if !val.is_empty() {
                    println!(
                        "\n\"{}\" wasn't included as it was not a valid number.",
                        val
                    );
                    println!("Enter only the numbers of the snapshots you'd like to include, separated by a single space.");
                }
                continue;
            }
        }
    }
    if vals.is_empty() {
        None
    } else if vals.len() > 5 {
        Some(vals[0..5].to_vec())
    } else {
        Some(vals)
    }
}

/// Get all of the items for a user - used for side-by-side viewer
/// This will return either the assets or liabilities (call once for each)
fn get_all_items(
    conn: &Connection,
    user: &User,
    which_half: &BalanceSheetHalf,
) -> Result<Vec<Item>> {
    // Next push all of the items that fit the snapshot into a vector
    let mut items: Vec<Item> = vec![];
    let mut stmt = conn.prepare(
        "SELECT * FROM balance_items 
        WHERE is_asset=?1 AND username_lower=?2",
    )?;
    let mut rows = stmt.query(rusqlite::params![
        which_half.to_bool_int(),
        user.username_lower,
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
    Ok(items)
}

/// Uses the textplots package to plot the net worth trend over the snapshots in the terminal
fn net_worth_graph(conn: &Connection, user: &User) -> Result<()> {
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

    if snapshots.is_empty() {
        println!("\n\nYou don't have any saved snapshots yet. Hit Enter to go back.");
        read_or_quit(); // Just to give the user a chance to acknowledge
        return Ok(());
    }

    if snapshots.len() == 1 {
        println!("\n\nYou need at least 2 snapshots for a trend line. Hit Enter to go back.");
        read_or_quit(); // Just to give the user a chance to acknowledge
        return Ok(());
    }

    // Sort the snapshots in chronological order
    snapshots.sort_by(|a, b| a.timeline.cmp(&b.timeline));

    let mut show_unscaled = true;

    // Switch back and forth between scaled and unscaled x-axis displays
    loop {
        if show_unscaled {
            // UNSCALED X-AXIS VERSION OF THE PLOT
            // Create a Vector of points with snapshot index as x
            let mut points: Vec<(f32, f32)> = vec![];
            let mut min_val: f64 = f64::MAX;
            let mut max_val: f64 = f64::MIN;
            for (idx, snapshot) in snapshots.iter().enumerate() {
                // Will later index back to snapshots vector to get date
                points.push((idx as f32, snapshot.net_worth as f32));
                if snapshot.net_worth < min_val {
                    min_val = snapshot.net_worth;
                }
                if snapshot.net_worth > max_val {
                    max_val = snapshot.net_worth;
                }
            }
            let lines = Shape::Lines(points.as_slice());
            let mut plot = Chart::new_with_y_range(
                250,
                80,
                0.0,
                (snapshots.len() - 1) as f32,
                (min_val - (0.2 * min_val.abs())) as f32,
                (max_val + (0.2 * min_val.abs())) as f32,
            );

            println!("\n\n\nYour Net Worth Trend");

            // Clone the dates for labeling the x-axis within the closure below which consumes the values
            // Avoids cloning the entire snapshots vector
            let closure_dates: Vec<String> = snapshots
                .iter()
                .map(|snapshot| snapshot.date_today.clone())
                .collect();

            plot.lineplot(&lines)
                .x_label_format(LabelFormat::Custom(Box::new(move |xval| {
                    closure_dates[xval as usize].to_owned()
                })))
                .y_label_format(LabelFormat::Custom(Box::new(move |yval| {
                    to_money_string(yval as f64)
                })))
                .y_tick_display(TickDisplay::Sparse)
                .nice();

            println!("\n\nNote: X-Axis not to scale\n");
            println!("What would you like to do next?");
            println!("1. View the trend with the x-axis to scale");
            println!("0. GO BACK");

            match print_instr_get_response(0, 1, || {}) {
                0 => break,
                1 => show_unscaled = false,
                x => panic!("Response {} is an error state. Exiting the program.", x),
            }
        } else {
            // SCALED X-AXIS VERSION OF THE PLOT
            // Guaranteed to have at least 2 snapshots
            let first_date = chrono::NaiveDate::parse_from_str(
                snapshots.first().unwrap().date_today.as_str(),
                "%Y-%m-%d",
            )
            .unwrap();
            let last_date = chrono::NaiveDate::parse_from_str(
                snapshots.last().unwrap().date_today.as_str(),
                "%Y-%m-%d",
            )
            .unwrap();

            // Make a new vector of points with duration in days from the first date for x
            // Min and max y values don't change from above
            let mut points: Vec<(f32, f32)> = vec![];
            let mut min_val: f64 = f64::MAX;
            let mut max_val: f64 = f64::MIN;
            for snapshot in &snapshots {
                let snap_date =
                    chrono::NaiveDate::parse_from_str(snapshot.date_today.as_str(), "%Y-%m-%d")
                        .unwrap();
                points.push((
                    (snap_date - first_date).num_days() as f32,
                    snapshot.net_worth as f32,
                ));
                if snapshot.net_worth < min_val {
                    min_val = snapshot.net_worth;
                }
                if snapshot.net_worth > max_val {
                    max_val = snapshot.net_worth;
                }
            }
            let lines = Shape::Lines(points.as_slice());
            let mut plot = Chart::new_with_y_range(
                250,
                80,
                0.0,
                (last_date - first_date).num_days() as f32,
                (min_val - (0.2 * min_val.abs())) as f32,
                (max_val + (0.2 * min_val.abs())) as f32,
            );

            println!("\n\n\nYour Net Worth Trend");

            plot.lineplot(&lines)
                .x_label_format(LabelFormat::Custom(Box::new(move |xval| {
                    format!("{}", first_date + Duration::days(xval as i64))
                })))
                .y_label_format(LabelFormat::Custom(Box::new(move |yval| {
                    to_money_string(yval as f64)
                })))
                .y_tick_display(TickDisplay::Sparse)
                .nice();

            println!("\n\nNote: X-Axis is now to scale\n");
            println!("What would you like to do next?");
            println!("1. View the trend with the x-axis unscaled");
            println!("0. GO BACK");

            match print_instr_get_response(0, 1, || {}) {
                0 => break,
                1 => show_unscaled = true,
                x => panic!("Response {} is an error state. Exiting the program.", x),
            }
        }
    }

    Ok(())
}
