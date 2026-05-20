# argentumOS

A consumer-grade Linux desktop built around a polished, silent boot and a
familiar Cinnamon environment. You install it, you sign in, and your computer
just works — no dotfiles, no terminal, no weekend lost to configuration.

> **Our philosophy:** if you need to open `nano` and the terminal to fix
> something mundane, *we failed.*
>
> Everything an average person should be able to do on their own computer —
> installing an app, changing the wallpaper, connecting to Wi-Fi, fixing a
> broken setting — should be a click, not a command.

## Get argentumOS

1. **Download** the latest `argentumOS.iso` from the
   [Releases page](https://github.com/) *(link will go live with the first
   release)*.
2. **Write it to a USB stick** (8 GB or larger) using whichever tool you
   already trust:
   - **On Windows:** [Rufus](https://rufus.ie) — pick the ISO, pick your USB,
     click *Start*.
   - **On macOS:** [balenaEtcher](https://etcher.balena.io) — *Flash from
     file*, choose your USB, *Flash*.
   - **On another Linux:** open *Disks* (or *GNOME Disks*), select your USB,
     and use *Restore Disk Image…* to point at the ISO.

No `dd`, no checksums to copy-paste — pick a tool, click the buttons, eject.

## Install it

1. Plug the USB stick into the computer you want argentumOS on, and turn it on
   while holding the boot-menu key (usually **F12**, **F10**, **Esc**, or
   **F2** — your laptop manufacturer tells you which).
2. Pick your USB stick from the boot menu. argentumOS boots straight to the
   desktop — no menu, no prompts.
3. The **Install argentumOS** window opens automatically a moment after the
   desktop appears. (If it doesn't, double-click the *Install argentumOS*
   icon.)
4. Click through:
   - **Language** — pick yours.
   - **Keyboard** — pick yours.
   - **Disk** — pick the disk you want to install on. *This erases the disk
     you select.* Pick carefully.
   - **You** — type a name and a password.
   - **Install** — go grab a coffee. When it finishes, *Restart*.
5. Pull the USB stick out when the screen goes dark. The computer reboots
   into argentumOS.

That's the whole installer. There is no terminal step. There is no
configuration file to edit. If any of the screens above ever asked you to
type a command, that is a bug — please open an issue.

## After install

- The first boot is silent — no scrolling text, no bootloader menu. You see a
  splash and then the login screen.
- Sign in with the name and password you just chose.
- Apps live in **Software**. Search, click *Install*, done. The catalog comes
  from [Flathub](https://flathub.org), so the apps you'd expect — Firefox,
  Spotify, Discord, OBS, LibreOffice, Steam — are all there.
- Everything else is in **System Settings** — the native argentumOS panel
  (Wi-Fi, display, users, appearance, software, system). It opens from the
  start menu like on any other desktop.

## Something's wrong

Open an issue on [GitHub](https://github.com/). Please describe what you
expected to be possible without the terminal — if the answer was "use the
terminal," we want to know about it.

## Coming soon

- A native **argentumOS App Store** — a curated, friendlier face for Flathub
  with featured picks and one-click app collections.
- **First-class Wine integration** — install a Windows program by
  double-clicking the `.exe`, with sensible defaults handled for you.
- **Custom argentum themes** — proper `argentum-gtk` and `argentum-icons`
  packages, replacing the placeholder look you see today.

## Done recently

- **Native System Settings panel** — argentumOS now ships its own
  Rust/GPUI settings app instead of the default Cinnamon panel. Source
  lives in `settings-panel/`; see `settings-panel/README.md`.

---

*Contributing? See `flake.nix` and `scripts/build.sh` for the build entry
points, and `installer/README.md` for the installer architecture.*
