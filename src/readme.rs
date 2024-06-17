use crate::conf;

use anyhow::bail;
use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};

pub fn gen_table(
    env_conf: &conf::Env,
    file_conf: &Vec<conf::Technology>,
    repo_owner: &str,
) -> Result<String, anyhow::Error> {
    let mut lines = Vec::new();

    // Add header
    lines.push(format!(
        "| {} **Technology** | {} **Projects** |",
        &env_conf.technology_emoji, &env_conf.project_emoji
    ));
    lines.push(String::from("| - | - |"));

    const FRAGMENT: &AsciiSet = &NON_ALPHANUMERIC;

    for tech in file_conf.iter() {
        let mut projects = Vec::new();
        for project in tech.projects.iter() {
            // Getting repo name
            let url_chunks: Vec<&str> = project.url.split('/').collect();
            let project_repo_owner = url_chunks.get(3).unwrap_or(&"").to_owned();
            let project_repo_name = url_chunks.get(4).unwrap_or(&"").to_owned();
            if project_repo_name.is_empty() {
                bail!("Failed to extract repository name from {}", project.url)
            }

            // Adding (WIP) to message if wip
            let mut message = String::from(project_repo_name);
            if project.wip {
                message.push_str("%20(WIP)");
            }

            // Add badge/url
            let display_text = if repo_owner == project_repo_owner {
                String::from(project_repo_name)
            } else {
                format!("{}/{}", project_repo_owner, project_repo_name)
            };
            if env_conf.badges {
                projects.push(format!("[![{}](https://img.shields.io/static/v1?label=&message={}&color=000605&logo=github&logoColor=FFFFFF&labelColor=000605)]({})", display_text, utf8_percent_encode(&message, FRAGMENT), project.url));
            } else {
                projects.push(format!("[{}]({})", display_text, message));
            }
        }
        let joined_projects = projects.join(" ");

        if env_conf.badges {
            lines.push(format!("| [![{}](https://img.shields.io/static/v1?label=&message={}&color={}&logo={}&logoColor={})]({}) | {} |",
            tech.name,
            utf8_percent_encode(&tech.name, FRAGMENT),
            tech.color.replace("#", ""),
            utf8_percent_encode(&tech.logo, FRAGMENT),
            tech.logo_color.replace("#", ""),
            tech.url,
            joined_projects,
        ))
        } else {
            lines.push(format!(
                "| [{}]({}) | {} |",
                tech.name, tech.url, joined_projects,
            ));
        }
    }

    Ok(lines.join("\n"))
}

const TABLE_START_MSG: &'static str = "<!-- START OF PROFILE STACK, DO NOT REMOVE -->";
const TABLE_STOP_MSG: &'static str = "<!-- END OF PROFILE STACK, DO NOT REMOVE -->";

pub fn insert_table(readme: &str, table: &str) -> Result<String, anyhow::Error> {
    let mut new_lines: Vec<&str> = readme.lines().collect();

    // Finding start and stop headers
    let mut found = false;
    let mut start = 0;
    let mut stop = 0;
    for (i, line) in readme.lines().enumerate() {
        match line {
            TABLE_START_MSG => {
                found = true;
                start = i + 1;
            }
            TABLE_STOP_MSG => stop = i,
            _ => continue,
        }
    }

    // Replacing (if found) or inserting table at the end (if not found)
    if found {
        for _ in start..stop {
            new_lines.remove(start);
        }
        for (i, line) in table.lines().enumerate() {
            new_lines.insert(start + i, line);
        }
    } else {
        new_lines.push(TABLE_START_MSG);
        for line in table.lines() {
            new_lines.push(line);
        }
        new_lines.push(TABLE_STOP_MSG);
    }

    Ok(new_lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::conf::{Env, Project, Technology};

    use super::*;

    const TEST_TABLE: &str = "| 💻 **Technology** | 🚀 **Projects** |\n| - | - |\n| [![Go Language](https://img.shields.io/static/v1?label=&message=Go%20Language&color=7FD6EA&logo=go&logoColor=201020)](https://golang.org/) | [![fgh](https://img.shields.io/static/v1?label=&message=fgh&color=000605&logo=github&logoColor=FFFFFF&labelColor=000605)](https://github.com/gleich/fgh) |\n| [![Python](https://img.shields.io/static/v1?label=&message=Python&color=3C78A9&logo=python&logoColor=FFFFFF)](https://www.python.org/) | [![profile_stack](https://img.shields.io/static/v1?label=&message=profile_stack&color=000605&logo=github&logoColor=FFFFFF&labelColor=000605)](https://github.com/gleich/profile_stack) [![github/test](https://img.shields.io/static/v1?label=&message=test%20(WIP)&color=000605&logo=github&logoColor=FFFFFF&labelColor=000605)](https://github.com/github/test) |";

    #[test]
    fn test_gen_table() -> Result<(), anyhow::Error> {
        assert_eq!(
            gen_table(
                &Env {
                    config_filename: Path::new("stack.yml").to_owned(),
                    badges: true,
                    technology_emoji: '💻',
                    project_emoji: '🚀',
                    output_file: Path::new("README.md").to_owned()
                },
                &vec![
                    Technology {
                        name: String::from("Go Language"),
                        logo_color: String::from("#201020"),
                        logo: String::from("go"),
                        url: String::from("https://golang.org/"),
                        color: String::from("#7FD6EA"),
                        projects: vec![Project {
                            url: String::from("https://github.com/gleich/fgh"),
                            wip: false
                        },]
                    },
                    Technology {
                        name: String::from("Python"),
                        logo: String::from("python"),
                        logo_color: String::from("#FFFFFF"),
                        url: String::from("https://www.python.org/"),
                        color: String::from("#3C78A9"),
                        projects: vec![
                            Project {
                                url: String::from("https://github.com/gleich/profile_stack"),
                                wip: false
                            },
                            Project {
                                url: String::from("https://github.com/github/test"),
                                wip: true
                            }
                        ]
                    }
                ],
                "gleich"
            )?,
            TEST_TABLE
        );

        Ok(())
    }

    #[test]
    fn test_insert_table() -> Result<(), anyhow::Error> {
        // No table
        assert_eq!(
            insert_table("# Hello World!", TEST_TABLE)?,
            format!(
                "# Hello World!\n{}\n{}\n{}",
                TABLE_START_MSG, TEST_TABLE, TABLE_STOP_MSG
            )
        );

        // With same table
        assert_eq!(
            insert_table(
                &format!(
                    "# Hello World!\n{}\n{}\n{}",
                    TABLE_START_MSG, TEST_TABLE, TABLE_STOP_MSG
                ),
                TEST_TABLE
            )?,
            format!(
                "# Hello World!\n{}\n{}\n{}",
                TABLE_START_MSG, TEST_TABLE, TABLE_STOP_MSG
            )
        );

        // With different table
        assert_eq!(
            insert_table(
                &format!(
                    "{}\n{}\n| Testing | Testing |\n{}",
                    TABLE_START_MSG, TEST_TABLE, TABLE_STOP_MSG
                ),
                TEST_TABLE
            )?,
            format!("{}\n{}\n{}", TABLE_START_MSG, TEST_TABLE, TABLE_STOP_MSG)
        );

        Ok(())
    }
}
