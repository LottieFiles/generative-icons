# Optional Claude Code adapter

Skip this unless the developer asks for a separate text proposal from the Claude Code CLI. A normal coding-agent session, including Claude Code itself, follows [SKILL.md](https://github.com/LottieFiles/lottie-generative-icons/blob/main/.agents/skills/animated-icon-workflow/SKILL.md) directly.

The adapter sends one bounded, text-only request through an authenticated `claude` executable. It supplies the skill, the polish checklist, the harness rules and a verified reference packet as text, and saves the proposal. It cannot edit source, export animations or inspect images.

## Run a proposal

Write a brief (meaning or correction, states, protected assets, relevant neighbors) that applies [polish-review.md](https://github.com/LottieFiles/lottie-generative-icons/blob/main/.agents/skills/animated-icon-workflow/references/polish-review.md). Check `claude --version`, then run from the repository root:

```bash
python3 .agents/skills/animated-icon-workflow/scripts/claude_design.py \
  path/to/brief.md \
  --neighbor keyframes \
  --context motion/icon-sets/library/drawing.mjs \
  --context motion/icon-sets/06-tools/definitions.mjs \
  --output path/to/proposal-01
```

Choose `--neighbor` and `--context` for the task. Selected references must match current package hashes; a missing reference or an unavailable CLI stops the run before any model call.

The output directory must be new. It receives `design.md`, `prompt.md`, `response.json`, `invocation.json` and `generation-context.json`, with input hashes, CLI version and returned model identifiers. A timeout is recorded as a failure, not retried. The runner keeps the developer's model configuration. Keep prompts and transcripts out of contributions.

## Continue

Review the proposal and save findings in `design-review.md`: what meets the brief, specific faults, and affected parts or times. For a revision, fix it in the current session or write a new brief with the observed failure and run into a new directory.

Then author the design and follow [production.md](https://github.com/LottieFiles/lottie-generative-icons/blob/main/.agents/skills/animated-icon-workflow/references/production.md). A failed, empty or timed-out response is not a completed design pass; record the actual error.
