#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let mut arguments = std::env::args().skip(1);
    if arguments.next().as_deref() == Some("--migrate-canonical") {
        match skills_manager_lib::migrate_configured_canonical() {
            Ok(message) => {
                println!("{message}");
                return;
            }
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
    }
    let mut arguments = std::env::args().skip(1);
    if arguments.next().as_deref() == Some("--consolidate-agents") {
        let agent_ids = arguments.collect::<Vec<_>>();
        match skills_manager_lib::consolidate_configured_agents(&agent_ids) {
            Ok(message) => {
                println!("{message}");
                return;
            }
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
    }
    skills_manager_lib::run();
}
