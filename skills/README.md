# Agent skills

The agent skill for apps is **`generative-icons`**. Its one source is [crates/generative-icons/assets/skill/](../crates/generative-icons/assets/skill/SKILL.md): the CLI embeds it, and `generative-icons init` installs it into an app's `.claude/skills/generative-icons/` and `.agents/skills/generative-icons/`. Edit it there.

`lottie-icons/` is the skill of the retired `lottie-icons` command, frozen byte for byte. `generative-icons init --migrate` recognizes these exact files by their hashes when it moves a project over, and the legacy package build still copies them into `packages/cli`. Do not edit them.

The maintainer workflow for this repository's own sets is [.agents/skills/animated-icon-workflow/](../.agents/skills/animated-icon-workflow/SKILL.md).
