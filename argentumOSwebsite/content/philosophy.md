---
title: Philosophy
description: Why argentumOS exists. Short answer — NixOS daily, and an unwillingness to inflict that on anyone else.
---

# Why this exists

I use NixOS daily.

argentumOS exists so nobody else has to.

That's the entire pitch, but you scrolled this far, so let's earn the page weight.

## The actual problem

Linux desktops are good. Most of the moving parts work. The kernel is fine, the userland is fine, the apps are mostly fine. What's not fine is the *seam between them*. The place where a perfectly reasonable thing you'd like to do — connect to Wi-Fi, change your display resolution, install Spotify — turns into a small research project.

A regular person should not have to know what `NetworkManager` is. Or `systemd-resolved`. Or that there are three different audio servers and one of them is being slowly phased out (which one? depends on the month).

The bar isn't *"the OS should be powerful"*. Linux is already powerful. The bar is **the OS should not require expertise to use**.

## What that actually means in practice

A few rules we keep coming back to:

1. **If the fix involves a terminal, the fix is broken.** Routine operations should be clickable. The terminal still exists — power users get to keep it — but no UI flow we ship should *require* it.

2. **Boring on top, interesting underneath.** NixOS is genuinely a great base. Reproducible builds, atomic updates, painless rollbacks. The user doesn't need to know any of this. They get the benefit without the syntax.

3. **Opinions, not options.** A settings menu with 400 toggles is a confession that you couldn't decide what the right defaults were. argentumOS picks. You can override if you really want to, but you mostly won't.

4. **Cohesion over comprehensiveness.** Two custom apps that match the rest of the system are worth more than ten apps that all look like they're from different decades.

5. **No save buttons.** This one keeps surprising people. If you can change a thing, the change applies when you change it. The model is "this is the current state," not "this is the state I'm thinking about."

## Who this is for

- People who tried Linux once, got tired, and went back to macOS or Windows.
- People who currently use Linux but spend two hours a month yak-shaving instead of doing the thing they meant to do.
- People who installed Linux on a parent's laptop and now do tech support every Sunday.
- Anyone who'd like *this* particular computer to be the one that stops being a project.

## Who this is not for

- People who enjoy `nix-env` as a hobby. (Hi! We see you. You're already happy. Stay there.)
- People whose preferred desktop is *"whatever I configure across 47 dotfiles in a public GitHub repo."* Cinnamon is going to feel quite plain. By design.
- People who consider it a personality trait that their laptop runs five-year-old GTK 2 apps in a hand-rolled WM. We respect the craft. We are not the OS for it.

## The honest part

This is one person's frustration, turned into a project. The README's last line says it best:

> I use NixOS daily. argentumOS exists so nobody else has to. xoxo Celestia

Everything past that is just engineering.

[Install it →](/install)
