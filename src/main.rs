use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use regex::Regex;
use zellij_tile::prelude::*;

#[derive(Default)]
struct ZellijForeman {
    config: BTreeMap<String, String>,
    error: Option<String>,
}

register_plugin!(ZellijForeman);

impl ZellijPlugin for ZellijForeman {
    fn load(&mut self, config: BTreeMap<String, String>) {
        self.config = config;

        request_permission(&[PermissionType::RunCommands]);
    }

    fn update(&mut self, _event: Event) -> bool {
        let procfile = "Procfile".to_string();
        let procfile = self.config.get("procfile").unwrap_or(&procfile);

        let data = match fs::read(format!("/host/{}", procfile)) {
            Ok(data) => data,

            Err(_) => {
                self.error = Some(format!("{} does not exist.", procfile));
                return false;
            }
        };

        let data = String::from_utf8_lossy(&data);
        let entries = parse_procfile(&data);

        if let Some(((_name, command), rest)) = entries.split_first() {
            open_command_pane_in_place(CommandToRun {
                path: PathBuf::from("sh"),
                args: vec!["-c".to_string(), command.to_string()],
                cwd: None,
            });

            for (_name, command) in rest {
                open_command_pane(CommandToRun {
                    path: PathBuf::from("sh"),
                    args: vec!["-c".to_string(), command.to_string()],
                    cwd: None,
                });
            }
        }

        false
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        if let Some(error) = &self.error {
            println!("{}", error);
        }
    }
}

fn parse_procfile(procfile: &str) -> Vec<(&str, &str)> {
    let re = Regex::new(r"(?m)^([\w-]+):\s*(.+)$").unwrap();

    re.captures_iter(procfile)
        .map(|c| {
            let (_, [name, command]) = c.extract();

            (name, command)
        })
        .collect()
}

#[test]
fn test_parse_procfile() {
    let procfile = r#"
web: npm start

api: npm run api
# comment one
comment two
    "#;

    let entries = parse_procfile(procfile);

    assert_eq!(entries, vec![("web", "npm start"), ("api", "npm run api"),]);
}
