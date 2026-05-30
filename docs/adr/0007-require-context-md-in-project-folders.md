# 0007. Require context.md in project folders

Status: accepted
Date: 2026-05-30

## Context and Problem

Large projects decay when local assumptions are not written down. Agents are especially likely to make incorrect edits if each folder lacks local purpose, boundaries, and history.

## Decision Drivers

- Each folder needs local memory.
- Agents should not infer ownership from filenames alone.
- Public contracts and open questions should be visible near the files they affect.
- Context should be short enough to read before edits.

## Considered Options

- Root-level documentation only.
- README files in selected folders.
- Required `context.md` in every project folder.
- No explicit local context.

## Decision

Every project folder must contain a `context.md` file.

The file records purpose, current shape, public contracts, related decisions, history, and open questions.

## Consequences

Positive:

- Local decisions become discoverable.
- Agents can load relevant context before editing.
- Folder boundaries become easier to maintain.

Negative:

- New folders require a context file.
- Context files must be updated when folder purpose or contracts change.

## Links

- [../../context.md](../../context.md)
- [../memory/project-governance.md](../memory/project-governance.md)
