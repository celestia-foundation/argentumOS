---
title: FAQ
description: Common questions about argentumOS, answered with the appropriate amount of caffeine.
---

# Frequently asked, slightly answered

## Is this just NixOS with a theme?

No. NixOS is the base — like Ubuntu is Debian with a sensible release schedule — but argentumOS ships two first-party apps written from scratch (System Settings, App Store), a custom installer flow, custom Plymouth boot, a curated default desktop, and a deliberate policy against requiring the terminal for anything routine.

You can configure NixOS yourself to *look* like argentumOS. You'll have a great time. We'll see you in three weekends.

## Why NixOS and not Arch / Debian / Fedora?

Atomic updates and reliable rollbacks. If something breaks, the user reboots and picks the previous generation from the boot menu. That property is hard to fake on top of a traditional package manager. The cost is build complexity, and the build complexity is our problem, not yours.

## Why Flatpak for apps and not Nix packages?

Because asking a user to wait for a system rebuild every time they want to try a new app is not a real plan. Flatpak ships fast, sandboxes properly, updates per-app, and Flathub already has the apps people actually want. Nix is excellent for the *system*. Flatpak is excellent for the *applications*. We use both.

## Will my hardware work?

Probably. argentumOS uses the standard Linux kernel with the standard set of firmware blobs (`linux-firmware`), and Cinnamon as the desktop. If a piece of hardware works on Ubuntu, it almost certainly works here. Boot the live ISO and check Wi-Fi, sound, and brightness before installing. That's the time to find out.

## Can I install AppImage / .deb / .rpm files?

AppImages — yes, they run anywhere, just `chmod +x` and run them (or use an AppImage launcher from Flathub).

`.deb` and `.rpm` — no, those are package formats for other distros and we are not pretending to be one. If the app you want only ships as a `.deb`, see if there's a Flatpak; there usually is.

## Can I open a terminal?

Yes. We're not monsters. The terminal is in the menu. It runs `bash`. Knock yourself out.

The rule is *the OS should not require it* for normal use. Having one is fine. Needing one to fix Wi-Fi is not.

## How do I update the system?

System Settings → System → "Check for updates." It does the thing.

Behind the scenes this runs `nixos-rebuild`. You don't need to know that. You can if you want.

## Can I dual-boot?

You can, but the installer doesn't ship a dual-boot wizard yet. For now: install Windows first, leave free space, install argentumOS into the free space. GRUB will pick up both. If this sentence sounded like instructions, you're already power-user-adjacent and you'll be fine.

## What's the release cadence?

Slow. On purpose. We'd rather skip a month than ship a broken update. NixOS gets two stable channels per year and we track those; the apps and theme update more often, but you'll never wake up to a UX overhaul.

## Is there a community?

GitHub issues, for now. If something breaks, that's where it goes. Describe what you tried to do *without the terminal*. If the workaround you found involved the terminal, that's the bug.

## Why "argentum"?

It's Latin for silver. Cohesive dark theme with a metallic accent. We're not over-thinking the etymology and neither should you.

## Who makes this?

[sinisterMage](https://github.com), hosted under the Celestia Foundation. One person, one project, one strong opinion about save buttons.

xoxo
