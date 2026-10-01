# Agent skills

The agent skill for apps is **`gen-animated-icons`**. Its one source is [crates/gen-animated-icons/assets/skill/](../crates/gen-animated-icons/assets/skill/SKILL.md): the CLI embeds it, and `gen-animated-icons init` installs it into an app's `.claude/skills/gen-animated-icons/` and `.agents/skills/gen-animated-icons/`. Edit it there.

`lottie-icons/` is the skill of the retired `lottie-icons` command, frozen byte for byte. `gen-animated-icons init --migrate` recognizes these exact files by their hashes when it moves a project over, and the legacy package build still copies them into `packages/cli`. Do not edit them.

The maintainer workflow for this repository's own sets is [.agents/skills/animated-icon-workflow/](../.agents/skills/animated-icon-workflow/SKILL.md).
