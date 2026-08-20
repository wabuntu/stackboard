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
<img src="https://raw.githubusercontent.com/wabuntu/stackboard/main/docs/volumes.png" alt="stackboard's volume list after switching resource type with :volumes" width="620">

All screenshots above are the real thing — captured against a live
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
- `s`: SSH to the selected server — type a username (remembered for next
  time) and press Enter to shell out to a real, interactive `ssh` session
- `d`: delete the selected resource — servers, volumes, networks,
  images, and security groups; not projects
- `b`: reboot the selected server (soft reboot)
- `p`: toggle power — stops an `ACTIVE` server, starts a `SHUTOFF` one
- `:`: open the command bar — type a resource name (`servers`,
  `volumes`, `networks`, `images`, `secgroups`, or `projects`) and
  press Enter to switch what's shown; more resource types are coming
- `r`: refresh now
- `q` / `Esc`: quit

`s` picks an IPv4 address off the server automatically (there's no
floating-IP-vs-fixed distinction to go on yet, so it's a best guess),
suspends the TUI, and hands the terminal to a real `ssh` — the exact
same "step out, then back in" pattern editors use for `$EDITOR`. Every
other action asks first — `d`/`b`/`p` open a y/n confirmation popup
naming the exact resource before anything is sent, and only `y` (not
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

stackboard authenticates against Keystone, resolves the service catalog,
and browses **servers** (Nova), **volumes** (Cinder), **networks** and
**security groups** (Neutron), **images** (Glance), and **projects**
(Keystone) — all verified end-to-end against a real DevStack deployment,
not just a mocked API. Servers get the full set (delete/reboot/
start-stop/ssh); volumes, networks, images, and security groups can be
deleted; projects are browse-only — deleting a whole tenant is a
different category of destructive than deleting one resource in it, so
that's deliberately not wired up. Deliberately left out of this
version:

- **More resource types** (ports, routers, flavors, ...) — the `:`
  command and internal resource-switching are built to make adding
  these straightforward, they're just not wired up yet.
- **More actions** (reboot/power for anything but servers, attach/
  detach a volume, edit a security group rule, ...) — each needs its
  own safe, clear interaction, same as delete did.

## Install

- Cargo: `cargo install stackboard`
- Debian package: https://github.com/wabuntu/stackboard/tree/main/target/debian
- RPM package: https://github.com/wabuntu/stackboard/tree/main/target/release/rpmbuild/RPMS/x86_64
- Single binary: https://github.com/wabuntu/stackboard/tree/main/binaries
