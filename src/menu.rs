use crate::structs_utils::*;
use rusqlite::Connection;
#[path = "balance_sheet.rs"]
mod balance_sheet;

/// Display the main menu and handle response
pub fn main_menu(conn: &Connection, user: &User) {
    match print_instr_get_response(1, 3, || {
        println!("Which section would you like to use? (Enter the number)");
        println!("1. Budget (coming soon!)");
        println!("2. Balance Sheet");
        println!("3. Quit");
    }) {
        1 => budget_menu(conn, user),
        2 => balance_sheet_menu(conn, user),
        3 => return,
        x => panic!("Response {} is an error state. Exiting the program.", x),
    }
}

/// Display the budget menu
pub fn budget_menu(_conn: &Connection, _user: &User) {
    match print_instr_get_response(1, 4, || {
        println!("\n\nBUDGET: For tracking money entering and leaving your possession each month");
        println!("Longer and shorter term expenses / income should be averaged on a monthly basis");
        println!("Example: A holiday bonus can be divided by 12 to reflect its effect on your monthly budget");
        println!("\nWhat would you like to do with your budget? (Enter the number)");
        println!("1. View Budget");
        println!("2. Update Monthly Expenses");
        println!("3. Update Monthly Income");
        println!("4. Quit")
    }) {
        1 => println!("Coming Soon"),
        2 => println!("Coming Soon"),
        3 => println!("Coming Soon"),
        x => panic!("Response {} is an error state. Exiting the program.", x),
    }
}

/// Display the balance sheet menu
pub fn balance_sheet_menu(conn: &Connection, user: &User) {
    match print_instr_get_response(1, 4, || {
        println!("\n\nBALANCE SHEET: For tracking long term assets and liabilities");
        println!("This does not track money that it moving, rather it tracks your net worth");
        println!("The only points you can later return to are the ones saved as a Snapshot");
        println!("\nWhat would you like to do with your balance sheet? (Enter the number)");
        println!("1. View Balance Sheet / Create Snapshot");
        println!("2. Update Assets");
        println!("3. Update Liabilities");
    }) {
        1 => balance_sheet::balance_sheet_whole_entry_point(conn, user),
        2 => balance_sheet::balance_sheet_half_entry_point(conn, user, BalanceSheetHalf::Assets),
        3 => {
            balance_sheet::balance_sheet_half_entry_point(conn, user, BalanceSheetHalf::Liabilities)
        }
        x => panic!("Response {} is an error state. Exiting the program.", x),
    }
}
