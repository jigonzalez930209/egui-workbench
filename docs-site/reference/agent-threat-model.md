---
title: Agent threat model
description: What the MCP server trusts, what it refuses by default, and where the embedding application still owns the risk.
---

# Agent threat model

The agent interface is an application surface, not a sandbox for a hostile model. This
page states what 0.14 protects and what it deliberately leaves to the embedding
application.

## Assets

- The host spec and the application state it describes.
- The commands the application registered, especially destructive ones.
- Data loaded into datasets (files, databases, live channels).
- The user's attention: a proposed change must be reviewable before it runs.

## Trust boundaries

| Boundary | Trust |
|---|---|
| MCP client ↔ server | Untrusted request stream. Every call is checked. |
| Server ↔ host | The embedding application owns this boundary. |
| Host ↔ plugins/native widgets | Native code is trusted; it is compiled into the application. |
| Model output ↔ UI | Untrusted text. It is rendered as text, never as executable policy. |

The MCP transport is stdio. The server opens no socket and has no authentication of its
own; the process that starts it is the trust anchor. The remote WebSocket server from
0.13 is a separate surface and is not exposed by the MCP binary.

## Controls in 0.14

1. **Read-only by default.** `apply_patch` and `run_command` fail with `read-only` unless
   the embedding application calls `Permissions::read_write()` or passes `--read-write`.
2. **Command allow-list.** When set, a command id outside the list is denied before the
   host sees it. An empty allow-list denies every command. Without an allow-list, only
   commands the spec declares (`commands`, menu rows, button ids) can run; an invented id
   is denied.
3. **Confirmation for sensitive commands.** Commands marked `destructive` can require a
   value that the client must supply in `arguments.confirmation`. The value is compared
   to the configured token in constant time and is not logged. An empty token never
   matches.
4. **Audit log.** Every tool call, allowed or denied, is appended with a sequence number,
   the tool name, and a detail of at most 240 characters. The log keeps the last 10,000
   entries. `list_activity` returns it.
5. **Local-only transport.** stdio; no listening socket, no network discovery.
6. **Accessible names, not pixels.** The automation and MCP surfaces use the spec's roles
   and names. A model cannot click arbitrary screen coordinates through this interface.
7. **Text rendering.** Agent messages render Markdown as text and code; the panel does not
   execute links, HTML, or commands.

## Known limits

- **The model can propose harmful patches.** The panel shows a diff and approvals are
  explicit, but an application that auto-approves defeats this control. Approval must be
  a person or policy the application trusts.
- **The allow-list is per session.** A long-lived server should restart or re-read policy
  when the user's intent changes.
- **No data-loss prevention.** A command that writes a file can still write it if it is on
  the allow-list. The host does not inspect command side effects.
- **Reads are side-effect free.** `get_state` returns pending events without draining
  them.
- **The spec is visible.** `get_spec` and `get_state` return everything in the spec,
  including metadata. Do not put secrets in `metadata` or widget text.
- **Prompt injection.** Text from a dataset, a file name, or a tool result can contain
  instructions. The model may follow them; the permission layer limits what that can do,
  not what the model says.
- **Screenshots are opt-in.** `get_screenshot` only works when the embedding installs a
  provider. That provider decides what is captured; a screenshot can contain secrets, so
  treat it with the same care as the underlying window.
- **Native custom widgets are trusted code.** They run in the host process and are not
  covered by the MCP permission model.

## Deployment checklist

- Start read-only unless the use case requires writes.
- Allow-list the smallest command set; keep destructive commands confirmable.
- Run the MCP server as the same user that owns the data, with the smallest file access
  that still works.
- Keep the activity log and review denials.
- Do not put credentials in the spec, widget text, dataset names, or metadata.
- Treat model output as untrusted when rendering it in your own UI.
