# stackboard

[![Crates.io](https://img.shields.io/crates/v/stackboard.svg)](https://crates.io/crates/stackboard)
[![docs.rs](https://img.shields.io/docsrs/stackboard)](https://docs.rs/stackboard)
[![CI](https://github.com/wabuntu/stackboard/actions/workflows/rust.yml/badge.svg)](https://github.com/wabuntu/stackboard/actions/workflows/rust.yml)

A TUI for browsing an OpenStack cloud, in the same spirit as `k9s` for
Kubernetes: switch what you're looking at with a `:` command, drill into
a resource for details, all without leaving the terminal or memorizing
`openstack` subcommands.

```
$ stackboard
```

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

## Usage

```
$ stackboard              # connect using whatever credentials are found, open the TUI
```

Keys:

- `↑`/`↓`: move the selection
- `Enter`: show details for the selected resource
- `:`: open the command bar — type a resource name and press Enter to
  switch what's shown (only `servers` exists in this version; more are
  coming)
- `r`: refresh now
- `q` / `Esc`: quit

Rows are colored by status — green (`ACTIVE`), red (`ERROR`), yellow
(`BUILD`/`REBOOT`/`MIGRATING`), muted blue-gray (`SHUTOFF`/`SUSPENDED`).

## What's in v0.1.0, and what isn't yet

This first version is read-only: it authenticates against Keystone,
resolves the service catalog, and browses **servers** (Nova instances).
Deliberately left out of this version:

- **Other resource types** (volumes, networks, images, projects) — the
  `:` command and internal resource-switching are built to make adding
  these straightforward, they're just not wired up yet.
- **Actions** (delete, reboot, start/stop) — k9s-style tools live and die
  by being safe to use against a real cluster, and this version hasn't
  been run against a real OpenStack deployment yet (verified so far only
  against a mocked Keystone/Nova API). Write operations are coming once
  that verification is possible.

## Testing against a real OpenStack

No cloud handy? `scripts/setup-devstack-vm.sh` launches an isolated
[multipass](https://multipass.run) VM and installs a minimal
[DevStack](https://docs.openstack.org/devstack/latest/) (Keystone, Nova,
Neutron, Glance, Cinder) inside it — DevStack makes invasive changes to
whatever it's installed on, so it runs in a disposable VM rather than on
your real machine. Takes 20-40 minutes; needs `multipass` (`sudo snap
install multipass`), 8+ CPUs, 12GB+ RAM, and 60GB+ free disk.

```
$ ./scripts/setup-devstack-vm.sh
...
$ source ~/.devstack-devstack-credentials
$ stackboard
```

Tear it down with `multipass delete --purge devstack` when you're done.

## Install

- Cargo: `cargo install stackboard`
- Debian package: https://github.com/wabuntu/stackboard/tree/main/target/debian
- RPM package: https://github.com/wabuntu/stackboard/tree/main/target/release/rpmbuild/RPMS/x86_64
- Single binary: https://github.com/wabuntu/stackboard/tree/main/binaries
