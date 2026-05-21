---
title: Install argentumOS
description: How to get argentumOS on a USB stick and onto your actual computer, with minimal weeping.
---

# Install argentumOS

It's a normal install. We promise. No `dd` rituals, no checksum séances, no Arch wiki tab open in the corner of your eye judging you.

## 1. Get the ISO

Grab `argentumOS.iso` from the [Releases page](https://github.com).

*(it exists. or will. probably. depends when you're reading this.)*

A roughly 4 GB download. You'll know it's done because it'll stop. Computers are like that.

## 2. Put it on a USB stick

**8 GB or larger.** Smaller works in theory and disappoints in practice.

- **Windows** — use [Rufus](https://rufus.ie). Select ISO. Select USB. Press *Start*. Resist overthinking.
- **macOS** — use [balenaEtcher](https://etcher.balena.io). Click *Flash from file*. Accept your fate.
- **Linux** — open *Disks*. Pick your USB. *Restore Disk Image…*. Done.

No `dd`. No checksums. Just click things until they stop being interactive.

## 3. Boot from it

Insert USB. Turn on the computer. Mash the boot key like it owes you money — usually **F12**, **F10**, **Esc**, or **F2**, depending on what your manufacturer decided was funny that year.

Select the USB device from the boot menu. argentumOS boots directly to the desktop, because grub menus are a cry for help.

## 4. Run the installer

It opens on its own. If it doesn't, the icon's on the desktop. Double-click it like a reasonable human.

Then:

1. **Language** — pick one you can read.
2. **Keyboard** — ideally matches your physical one.
3. **Disk** — choose carefully. This step deletes things *permanently*. Not metaphorically.
4. **You** — name and password. Try not to forget them immediately.
5. **Install** — wait. Maybe hydrate.

When it's done, click *Restart*. Remove the USB once the screen goes dark.

If you forget the USB, the computer will remind you by being annoying.

## 5. First boot

Silence. No scrolling logs. A splash screen, then a login prompt. It is possible to interpret this as ominous; it's actually just calm.

Log in. Two apps are pinned to the dock you should know about:

- **App Store** — search, click *Install*, app appears.
- **System Settings** — Wi-Fi, display, users, appearance, etc. It opens like a normal settings panel because it is one.

That's the install. If you had to Google a command, we messed up.

xoxo Celestia
