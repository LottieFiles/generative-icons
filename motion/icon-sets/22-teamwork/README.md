# Teamwork

Source set `22-teamwork` in the Teamwork gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-teamwork`.

## Icons

- Invite teammate (`invite-teammate`)
- Join team (`join-team`)
- Team directory (`team-directory`)
- Team roles (`team-roles`)
- Team presence (`team-presence`)
- Assign task (`assign-task`)
- Delegate task (`delegate-task`)
- Task handoff (`task-handoff`)
- Task dependency (`task-dependency`)
- Shared goal (`shared-goal`)
- Kanban board (`kanban-board`)
- Sprint cycle (`sprint-cycle`)
- Team roadmap (`team-roadmap`)
- Milestone (`milestone`)
- Team workload (`team-workload`)
- Shared document (`shared-document`)
- Co-edit (`co-edit`)
- Live cursors (`live-cursors`)
- Follow collaborator (`follow-collaborator`)
- Suggested edit (`suggested-edit`)
- Add comment (`add-comment`)
- Reply thread (`reply-thread`)
- Mention teammate (`mention-teammate`)
- Resolve thread (`resolve-thread`)
- Reopen thread (`reopen-thread`)
- Request review (`request-review`)
- Approve change (`approve-change`)
- Request changes (`request-changes`)
- Compare revisions (`compare-revisions`)
- Version history (`version-history`)
- Share workspace (`share-workspace`)
- Invite link (`invite-link`)
- Guest access (`guest-access`)
- View access (`view-access`)
- Edit access (`edit-access`)
- Standup (`standup`)
- Raise hand (`raise-hand`)
- Team poll (`team-poll`)
- Decision log (`decision-log`)
- Team celebration (`team-celebration`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/22-teamwork
node motion/icon-sets/22-teamwork/verify.mjs
node motion/icon-sets/22-teamwork/verify-clearance.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/22/assets/set.json)
