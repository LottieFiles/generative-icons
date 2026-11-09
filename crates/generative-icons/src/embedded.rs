//! Files compiled into the binary: the agent skill and managed state files. The
//! authoring kit comes from [`gai_js::kit`] (see [`crate::kit`]).

/// The skill folder name under each skill root.
pub const SKILL_NAME: &str = "generative-icons";

/// Skill files, relative to the skill folder, in breadth-first bytewise order.
/// craft, design-principles, motion-principles, motion-system and polish-review are the family's
/// direction, generated from the canonical docs by `npm run docs:build`.
pub const SKILL_FILES: [(&str, &str); 10] = [
    ("SKILL.md", include_str!("../../../skills/generative-icons/SKILL.md")),
    ("references/authoring-v2.md", include_str!("../../../skills/generative-icons/references/authoring-v2.md")),
    ("references/authoring.md", include_str!("../../../skills/generative-icons/references/authoring.md")),
    ("references/craft.md", include_str!("../../../skills/generative-icons/references/craft.md")),
    ("references/design-principles.md", include_str!("../../../skills/generative-icons/references/design-principles.md")),
    ("references/motion-principles.md", include_str!("../../../skills/generative-icons/references/motion-principles.md")),
    ("references/motion-system.md", include_str!("../../../skills/generative-icons/references/motion-system.md")),
    ("references/polish-review.md", include_str!("../../../skills/generative-icons/references/polish-review.md")),
    ("references/review.md", include_str!("../../../skills/generative-icons/references/review.md")),
    ("references/runtime.md", include_str!("../../../skills/generative-icons/references/runtime.md")),
];

/// Contents of the managed `.generative-icons/.gitignore`: only the lock is committed.
pub const STATE_GITIGNORE: &str = "*\n!.gitignore\n!lock.json\n";

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    /// Breadth-first, bytewise per directory, files only (Node's recursive readdir order).
    fn walk(root: &Path) -> Vec<String> {
        let mut files = Vec::new();
        let mut queue = vec![String::new()];
        while !queue.is_empty() {
            let mut next = Vec::new();
            for dir in queue {
                let mut names: Vec<String> = std::fs::read_dir(root.join(&dir))
                    .unwrap()
                    .map(|e| e.unwrap().file_name().into_string().unwrap())
                    .collect();
                names.sort();
                for name in names {
                    let relative = if dir.is_empty() { name } else { format!("{dir}/{name}") };
                    if root.join(&relative).is_dir() { next.push(relative) } else { files.push(relative) }
                }
            }
            queue = next;
        }
        files
    }

    #[test]
    fn skill_list_matches_the_skill_folder() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills/generative-icons");
        let listed: Vec<&str> = SKILL_FILES.iter().map(|(path, _)| *path).collect();
        assert_eq!(walk(&root), listed);
    }

    #[test]
    fn skill_frontmatter_names_the_folder() {
        assert!(SKILL_FILES[0].1.starts_with(&format!("---\nname: {SKILL_NAME}\n")));
    }

    #[test]
    fn skill_shows_the_mcp_entry_init_prints() {
        let skill = SKILL_FILES[0].1;
        let start = skill.find("{\"mcpServers\"").expect("SKILL.md shows the MCP entry");
        let end = start + skill[start..].find('`').expect("the entry is inline code");
        let shown: serde_json::Value = serde_json::from_str(&skill[start..end]).unwrap();
        assert_eq!(shown, crate::mcp::client_config());
    }

    /// Command lines the skill shows: fenced lines and inline code spans.
    fn shown_commands(text: &str) -> Vec<&str> {
        let mut fragments = Vec::new();
        let mut fenced = false;
        for line in text.lines() {
            if line.starts_with("```") {
                fenced = !fenced;
            } else if fenced {
                fragments.push(line);
            } else {
                fragments.extend(line.split('`').skip(1).step_by(2));
            }
        }
        fragments.into_iter().filter(|fragment| fragment.starts_with("generative-icons ")).collect()
    }

    #[test]
    fn skill_commands_and_flags_exist() {
        use clap::CommandFactory;
        let cli = crate::cli::Cli::command();
        let mut checked = 0;
        for (file, text) in SKILL_FILES {
            for line in shown_commands(text) {
                let words: Vec<&str> = line
                    .split('#')
                    .next()
                    .unwrap_or_default()
                    .split_whitespace()
                    .map(|w| w.trim_matches(['[', ']']))
                    .collect();
                let Some(command) = cli.find_subcommand(words[1]) else {
                    assert!(words[1].starts_with('<'), "{file}: no command {} in `{line}`", words[1]);
                    continue;
                };
                for flag in words.iter().filter_map(|word| word.strip_prefix("--")) {
                    let known =
                        command.get_arguments().chain(cli.get_arguments()).any(|arg| arg.get_long() == Some(flag));
                    assert!(known, "{file}: {} has no --{flag} (`{line}`)", words[1]);
                }
                checked += 1;
            }
        }
        assert!(checked >= 15, "{checked}");
    }
}
