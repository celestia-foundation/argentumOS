# argentumOS

A Linux desktop for people who would like their computer to behave normally for once.

You install it. You log in. Things mostly function. No ritual incantations, no terminal spelunking, no “quick fix” that turns into a 3-hour side quest.

> **Philosophy:** if fixing Wi-Fi requires opening `nano`, something has gone terribly wrong.
>
> Ordinary actions should be… ordinary. Click button → thing happens. That’s the bar.

---

## Get argentumOS

1. **Download** the latest `argentumOS.iso` from the Releases page
   *(it exists. or will. probably.)*

2. **Put it on a USB stick** (8 GB or more, unless you enjoy disappointment):

   * **Windows:** Use Rufus. Select ISO. Select USB. Press *Start*. Resist overthinking.
   * **macOS:** Use balenaEtcher. Click *Flash from file*. Accept your fate.
   * **Linux:** Open *Disks*. Find your USB. *Restore Disk Image…*. Done.

No `dd`. No checksum rituals. Just click things until it works.

---

## Install it

1. Insert USB. Turn on computer. Mash the boot key like it owes you money
   (**F12**, **F10**, **Esc**, **F2**, or whatever your manufacturer decided).

2. Select the USB device. argentumOS boots directly into a desktop because menus are a cry for help.

3. The installer appears on its own. If it doesn’t, double-click the icon like a reasonable human.

4. Follow the sequence:

   * **Language** — pick one you can read.
   * **Keyboard** — ideally matches your physical one.
   * **Disk** — choose carefully. This step deletes things permanently. Not metaphorically.
   * **You** — name + password. Try not to forget them immediately.
   * **Install** — wait. Maybe hydrate.

5. When it finishes, click *Restart*. Remove the USB when the screen goes dark.
   If you forget, the computer will remind you by being annoying.

That’s it. No terminal. No config files. If you had to Google a command, we messed up.

---

## After install

* Boot is silent. No scrolling nonsense. Just a splash screen and then login. Suspiciously calm.

* Log in using the credentials you hopefully remember.

* Apps live in **Software**:
  Search → Click *Install* → App appears.
  This is powered by Flathub, which means the usual suspects are available.

* Everything else lives in **System Settings**:
  Wi-Fi, display, users, appearance, etc.
  It opens like a normal settings panel, because it is one.

---

## Something’s wrong

Open an issue on GitHub.

Describe what you tried to do without the terminal.
If the solution involved the terminal, congratulations, you found a bug.

---

## Coming soon

* **argentumOS App Store**
  Same apps, less chaos. Curated. Slightly more opinionated.

* **Wine integration**
  Double-click a `.exe`. It runs. Ideally without summoning ancient forces.

* **Custom themes**
  A proper look, instead of “generic Linux but polite.”

---

## Done recently

* **Native System Settings panel**
  Built in Rust/GPUI because we like control and mild suffering.
  Lives in `settings-panel/` if you feel curious or reckless.

---

*Contributing? There’s a `flake.nix`, some scripts, and documentation scattered around like breadcrumbs. You’ll find it.*
