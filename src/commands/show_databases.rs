use crate::database;

pub fn handle_show_databases() {
    let databases = database::list_databases();

    if databases.is_empty() {
        println!("No databases found.");
    } else {
        println!("Databases:");
        for db in databases {
            println!("- {}", db);
        }
    }
}
