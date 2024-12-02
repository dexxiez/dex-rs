mod project_finder;

use project_finder::find_project_files;

fn main() -> anyhow::Result<()> {
    let home = dirs::home_dir().expect("Failed to get home directory");
    let root_dirs = vec![home.join("Documents/code"), home.join("Documents/godot")];

    let projects = find_project_files(&root_dirs)?;

    for project in projects {
        println!("{:?}", project);
    }
    Ok(())
}
