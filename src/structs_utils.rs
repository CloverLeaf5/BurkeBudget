use text_io::read;

/// The User struct for the application
#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
    pub username_lower: String,
    pub firstname: String,
    pub lastname: String,
}
impl User {
    pub fn fullname(&self) -> String {
        self.firstname.clone() + " " + self.lastname.clone().as_str()
    }
}

/// Utilizes the read!() macro but exits the program if the user has input "quit"
pub fn read_or_quit() -> String {
    let input: String = read!();
    if input.eq_ignore_ascii_case("quit") {
        std::process::exit(0);
    } else {
        input
    }
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
