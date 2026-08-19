# stackboard

[![Crates.io](https://img.shields.io/crates/v/stackboard.svg)](https://crates.io/crates/stackboard)
[![CI](https://github.com/wabuntu/stackboard/actions/workflows/rust.yml/badge.svg)](https://github.com/wabuntu/stackboard/actions/workflows/rust.yml)

stackboard is a `k9s`-style TUI for browsing and operating an OpenStack
cloud: switch what you're looking at with a `:` command, drill into a
resource for details, delete/reboot/start/stop it — all without leaving
the terminal or memorizing `openstack` subcommands and piping their
output through `grep`.

```
$ stackboard
```

<img src="https://raw.githubusercontent.com/wabuntu/stackboard/main/docs/list.png" alt="stackboard's server list, colored by status, with live counts in the header" width="620">
<img src="https://raw.githubusercontent.com/wabuntu/stackboard/main/docs/detail.png" alt="stackboard's server detail popup, its border colored to match the server's status" width="620">

Both screenshots above are the real thing — captured against a live
OpenStack (DevStack) cloud, not a mockup.

## Zero-config if you already use the `openstack` CLI

stackboard reads credentials the exact same way the `openstack` CLI does —
`OS_*` environment variables first (so an already-`source`d `openrc.sh`
just works), then `clouds.yaml` (checked in the current directory,
`~/.config/openstack/`, and `/etc/openstack/`). If neither is found, it
walks you through a one-time setup wizard and saves what you enter to
`~/.config/openstack/clouds.yaml`-style file it manages itself, so you
only answer the questions once.

```
$ stackboard setup   # run the wizard on demand, e.g. to add another cloud
```

<img src="https://raw.githubusercontent.com/wabuntu/stackboard/main/docs/setup.png" alt="stackboard's setup wizard prompting for an auth URL, username, password, and project, then confirming it saved the result" width="620">

## Usage

```
$ stackboard              # connect using whatever credentials are found, open the TUI
```

Keys:

- `↑`/`↓`: move the selection
- `Enter`: show details for the selected resource
- `d`: delete the selected server
- `b`: reboot it (soft reboot)
- `p`: toggle power — stops an `ACTIVE` server, starts a `SHUTOFF` one
- `:`: open the command bar — type a resource name and press Enter to
  switch what's shown (only `servers` exists in this version; more are
  coming)
- `r`: refresh now
- `q` / `Esc`: quit

Every action asks first — `d`/`b`/`p` open a y/n confirmation popup
naming the exact server before anything is sent, and only `y` (not
Enter, not any other key) confirms it. Nothing fires on a stray
keypress.

<img src="https://raw.githubusercontent.com/wabuntu/stackboard/main/docs/confirm.png" alt="stackboard's delete confirmation popup, red-bordered, naming the exact server and warning it can't be undone" width="620">

The header keeps a running count of active/error/other so trouble is
visible before you even look at the list, and every row carries the same
status color as a ● marker — green (`ACTIVE`), red (`ERROR`), yellow
(`BUILD`/`REBOOT`/`MIGRATING`), muted blue-gray (`SHUTOFF`/`SUSPENDED`).
Open a server's details and its popup border picks up the same color, so
you never lose track of what you're looking at.

## What's here, and what isn't yet

This first version authenticates against Keystone, resolves the service
catalog, and browses and operates on **servers** (Nova instances) —
listing, detail view, delete, reboot, and start/stop are all verified
end-to-end against a real DevStack deployment, not just a mocked API.
Deliberately left out of this version:

- **Other resource types** (volumes, networks, images, projects) — the
  `:` command and internal resource-switching are built to make adding
  these straightforward, they're just not wired up yet.

## Install

- Cargo: `cargo install stackboard`
- Debian package: https://github.com/wabuntu/stackboard/tree/main/target/debian
- RPM package: https://github.com/wabuntu/stackboard/tree/main/target/release/rpmbuild/RPMS/x86_64
- Single binary: https://github.com/wabuntu/stackboard/tree/main/binaries
