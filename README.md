# retro-veneer

A set of codes/scripts to simplify the process of setting up and keeping up-to-date a set of emulators for modern retrocomputers. More specifically, RetroVeneer will cause the system to boot directly into the emulator of your choice, resulting in an faux modern-retro system.

## Target Platform

This script and set of programs was designed and tested for use only on Raspberry Pi systems (Raspberry Pi 4 or later) running stock Raspberry Pi OS.

### Installing Retroveneer

I recommend that you start off with a fresh install of the current Raspberry Pi OS, flashed to a micro-sd card. Once you have booted into this OS, make sure that it is fully updated. Once you have done this, you can simply install RetroVeneer by running the following command within a terminal on that system:

```bash
scr=`mktemp` && wget -q https://raw.githubusercontent.com/ModernRetroDev/retro-veneer/refs/heads/master/hosted/rvbs.sh -O $scr && bash $scr
```

This command will download and run the graphical installer.

After installing RetroVeneer, you will be asked to select the platform you would like to boot right into.

Once that is done, you can either launch the emulator, or restart the system and have it boot right into the emulator.

### Expected Usage

Upon powering up a system with RetroVeneer installed, it will autostart the RetroVeneer UI with a 10 second countdown. If you take no action during that ten seconds, RetroVeneer will boot right into the emulator for the currently selected platform.

Should you wish to restart an emulated system (as is customary with other systems) at any point, simply kill the emulator by pressing the key combo `ALT+F4`. RetroVeneer is designed to not be killed by this key combination and will in fact, will instantly restart the emulator for the currently selected platform.

If you wish to not launch into this endless emulation loop, simply power cycle the Raspberry Pi system and press `ESC` on the keyboard during the intial 10 second countdown. This will put you into the configuration menu for RetroVeneer.

### Emulated Platform Data Directories

Upon installation, RetroVeneer will create numerous platform directories under `$HOME/retroveneer/data`. These are the directories which are mounted by the respective emulators when they are launched. So if you wish to download and make software or games available to these systems, simply put them in these locations.

## Unsupported Platforms

While I do technically have a version the installer and UI compiled for x86_64 systems, I don't exhaustively test RetroVeneer on these systems. In development I was able to install RetroVeneer to an emulated x86_64 system running PopOS. However, this distribution does not include by default the same shared libraries as Raspberry Pi OS running on aarch64. As such, I noticed that some of the emulators would not run out of the box. So if you really feel inclined to run this on a system of this nature. you may need to test the functionality of the various emulators individually to see if they are missing any system libraries (i.e., use `ldd emuname` and see if any libraries are marked as "Not Found"). If there are missing libraries, install them using the package manager for the system in question.

