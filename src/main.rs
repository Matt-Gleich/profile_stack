use std::io::Write;
use std::{env, fs};

use fs::File;
use tracing::{info, warn};

mod conf;
mod git;
mod readme;

fn main() {
    tracing_subscriber::fmt::init();

    env::set_current_dir("/github/workspace/")
        .expect("Failed to change directory to repo location");

    // Getting configuration
    let env_var_conf = conf::env_vars().expect("Failed to get env var config");
    let file_conf = conf::config_file(&env_var_conf)
        .expect("Failed to get configuration from file (CHECK FOR NEW UPDATE)");
    info!("Got configuration inputs");

    // Generating table
    let repo_owner = git::repo_owner().expect("Failed gto get repo owner");
    let table = readme::gen_table(&env_var_conf, &file_conf, &repo_owner)
        .expect("Failed to generate table");
    info!("Generated table");

    // Inserting table into README
    let readme_content = fs::read_to_string(readme::FILE_NAME)
        .expect(&format!("Failed to read from {}", readme::FILE_NAME));
    let patched_content = readme::insert_table(&readme_content, &table)
        .expect("Failed to insert table to README data");

    // Writing the changes to the README
    if readme_content != patched_content {
        // Writing changes
        let mut readme_file =
            File::create(&readme::FILE_NAME).expect("Failed to create README.md file struct");
        readme_file
            .write_all(patched_content.as_bytes())
            .expect(&format!("Failed to write changes to {}", readme::FILE_NAME));
        info!("Wrote changes to {}", readme::FILE_NAME);

        git::commit_and_push().expect("Failed to commit and push changes");

        info!("Committed changes! Have a good day :)")
    } else {
        warn!("No changes to README.md")
    }
}
