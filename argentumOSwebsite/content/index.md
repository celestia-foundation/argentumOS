---
title: Home
description: argentumOS — a Linux desktop that behaves.
---

## The bar is on the floor and we're stepping over it

Most Linux distros assume you woke up wanting to learn. argentumOS assumes you woke up wanting to check email, write something, and maybe edit a photo without `journalctl` becoming part of your day.

It's NixOS underneath — reproducible, atomic, the whole party — but you will never see a `.nix` file unless you go looking for one. We hid them. They're fine. Don't worry about it.

> If fixing Wi-Fi requires opening `nano`, something has gone terribly wrong. Ordinary actions should be… ordinary. Click button → thing happens. That's the bar.

There are two first-party apps: a **System Settings** panel and an **App Store**. Both written in Rust with [GPUI](https://github.com/zed-industries/zed) (yes, the framework Zed uses, because we like control and mild suffering). Everything else comes from Flathub, because reinventing the file manager is a hobby and you have things to do.

If something is broken, the fix should not require typing. If it does, congratulations — you found a bug.
