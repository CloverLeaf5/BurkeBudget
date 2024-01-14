/// The User struct for the application
#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
    pub username_lower: String,
    pub firstname: String,
    pub lastname: String,
    pub is_deleted: bool,
}
impl User {
    pub fn fullname(&self) -> String {
        self.firstname.clone() + " " + self.lastname.clone().as_str()
    }
}

#[derive(Debug, PartialEq)]
pub struct Category {
    pub category: String,
    pub category_lower: String,
    pub username_lower: String,
    pub is_asset: bool,
}
/// Refers to an asset or liability item
/// category_lower and username_lower referance the balance_categories and users tables, respectively
/// The timeline is an incrementing value (stored in a separate table) that gives an order to the items
/// timeline_created refers to when this specific version of this item was created (changed with updates)
/// timeline_original refers to when the first iteration of this item was created (not changed with updates)
///     This may be used when making a view of snapshots in the Snapshot section
/// timeline_deleted is set to a very high number by default. When an item is deleted it is given a timeline value instead
#[derive(Debug, PartialEq)]
pub struct Item {
    pub item: String,
    pub item_lower: String,
    pub value: f64,
    pub category: String,
    pub category_lower: String,
    pub username_lower: String,
    pub is_asset: bool,
    pub timeline_created: usize,
    pub timeline_original: usize,
    pub is_deleted: bool,
    pub timeline_deleted: usize,
}

#[derive(Debug, PartialEq)]
pub struct Snapshot {
    pub timeline: usize,
    pub username_lower: String,
    pub date_today: String,
    pub net_worth: f64,
    pub comment: String,
    pub is_deleted: bool,
}

/// The maximum length is related to the formatting used
pub const MAX_CHARACTERS_ITEM_NAME: usize = 28;

/// Assets or Liabilities
#[derive(PartialEq)]
pub enum BalanceSheetHalf {
    Assets,
    Liabilities,
}
impl BalanceSheetHalf {
    pub fn to_str(&self) -> &str {
        match self {
            BalanceSheetHalf::Assets => "Assets",
            BalanceSheetHalf::Liabilities => "Liabilities",
        }
    }
    /// Works with the is_asset boolean in the SQLite DB
    pub fn to_bool_int(&self) -> usize {
        match self {
            BalanceSheetHalf::Assets => 1,
            BalanceSheetHalf::Liabilities => 0,
        }
    }
    /// Works with the is_asset boolean in the Category struct
    pub fn to_bool(&self) -> bool {
        match self {
            BalanceSheetHalf::Assets => true,
            BalanceSheetHalf::Liabilities => false,
        }
    }
}

#[derive(PartialEq)]
pub enum BudgetHalf {
    Income,
    Expenses,
}
impl BudgetHalf {
    pub fn to_str(&self) -> &str {
        match self {
            BudgetHalf::Income => "Income",
            BudgetHalf::Expenses => "Expenses",
        }
    }
    /// Works with the is_income boolean in the SQLite DB
    pub fn to_bool_int(&self) -> usize {
        match self {
            BudgetHalf::Income => 1,
            BudgetHalf::Expenses => 0,
        }
    }
    /// Works with the is_income boolean in the Category struct
    pub fn to_bool(&self) -> bool {
        match self {
            BudgetHalf::Income => true,
            BudgetHalf::Expenses => false,
        }
    }
}

/// Utilizes the read!() macro but exits the program if the user has input "quit"
// TODO: Sanitize input to only allow periods and characters (avoid SQL injection)
pub fn read_or_quit() -> String {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Error getting input");
    input = input.trim_end().to_string(); // Trim whitespace
    let words: Vec<&str> = input.split(' ').collect();
    for word in words {
        if word.eq_ignore_ascii_case("quit") {
            println!("\nYour budget is saved, and you have been logged out. See you next time!");
            std::process::exit(0);
        }
    }
    input
}

/// Get a unint response from the user within a specific range
/// Uses a function to print out the options and instructions
pub fn print_instr_get_response<F: Fn()>(minval: usize, maxval: usize, instructions: F) -> usize {
    let mut selection = usize::MAX;
    let mut user_input: String;
    while selection < minval || selection > maxval {
        // Print out instructions
        instructions();
        user_input = read_or_quit();
        // Make sure the input is valid
        match user_input.parse::<usize>() {
            Ok(input_num) => {
                selection = input_num;
                if selection < minval || selection > maxval {
                    println!("\nPlease enter a valid number.");
                }
            }
            Err(_err) => {
                println!("\nPlease enter a valid number.");
            }
        }
    }
    selection
}
