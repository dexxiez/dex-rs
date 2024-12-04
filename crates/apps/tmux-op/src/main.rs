mod languages;
mod project_finder;
mod ui;

use project_finder::find_project_files;

fn main() -> anyhow::Result<()> {
    let home = dirs::home_dir().expect("Failed to get home directory");
    let root_dirs = vec![home.join("Documents/code"), home.join("Documents/godot")];

    let projects = find_project_files(&root_dirs)?;

    for project in projects {
        match languages::Language::from_name(&project.language) {
            Some(lang) => {
                println!("{} {}", lang.icon, project.name);
            }

            None => {
                println!("  {}", project.name);
            }
        }
    }

    let _ = ui::main();
    Ok(())
}
