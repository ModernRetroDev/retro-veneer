use dirs;
use raylib::prelude::*;
use std::io::Write;
use std::fs;
use std::fs::File;
use std::process::Command;
use tokio;
use tokio::time::{sleep, Duration};

static mut EXIT_NOW:     bool = false;
static mut SPINNER:      u8 = 0;
static mut SPINNER_NEXT: u8 = 0;
static mut STEP_NEXT:    String = String::new();
static mut STEP:         String = String::new();


fn get_emu_x16() -> bool {
    let zip_addr: String;
    let zip_fname: String;
    let relpath: &str = 
        "https://github.com/X16Community/x16-emulator/releases/download/r48/";

    unsafe {
        STEP_NEXT = "Installing Emulator -- X16...".to_string();
    }

    //------------------------------------------------------------------------//
    // Download a copy of the appropriate archive.                            //
    //------------------------------------------------------------------------//
    match std::env::consts::ARCH {
        "x86_64" => {
            zip_fname = "x16emu_linux-x86_64-r48.zip".to_string();
        },
        "aarch64" => {
            zip_fname = "x16emu_linux-aarch64-r48.zip".to_string();
        },
        _ => {
            println!("Unsupported architecture! Aborting!!!");
            return true;
        }
    }
    zip_addr = format!("{}{}", relpath, zip_fname);

    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("wget {}", zip_addr))
        .output();

    //------------------------------------------------------------------------//
    // Make directory structure for this emulator.                            //
    //------------------------------------------------------------------------//
    let _ = Command::new("sh")
        .arg("-c")
        .arg("mkdir -p $HOME/retroveneer/emulators/x16emu")
        .output();

    let _ = Command::new("sh")
        .arg("-c")
        .arg("mkdir -p $HOME/retroveneer/data/x16emu")
        .output();

    //------------------------------------------------------------------------//
    // Unpack emulator into aformentioned directory.                          //
    //------------------------------------------------------------------------//
    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("unzip {} -d $HOME/retroveneer/emulators/x16emu", zip_fname))
        .output();

    //------------------------------------------------------------------------//
    // Make a directory for emualtor archives.                                //
    //------------------------------------------------------------------------//
    let _ = Command::new("sh")
        .arg("-c")
        .arg("mkdir -p $HOME/retroveneer/emulators/archives")
        .output();

    //------------------------------------------------------------------------//
    // Move the archive into the archives directory.                          //
    //------------------------------------------------------------------------//
    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("mv {} $HOME/retroveneer/emulators/archives", zip_fname))
        .output();

    return false;
}

fn get_emu_tic80() -> bool {
    let archive_addr: String;
    let archive_fname: String;
    let relpath: &str = 
        "https://github.com/nesbox/TIC-80/releases/download/v1.1.2837/";

    unsafe {
        STEP_NEXT = "Installing Emulator -- TIC-80...".to_string();
    }

    //------------------------------------------------------------------------//
    // Download a copy of the appropriate archive.                            //
    //------------------------------------------------------------------------//
    match std::env::consts::ARCH {
        "x86_64" => {
            archive_fname = "tic80-v1.1-linux.deb".to_string();
        },
        "aarch64" => {
            archive_fname = "tic80-v1.1-rpi.deb".to_string();
        },
        _ => {
            println!("Unsupported architecture! Aborting!!!");
            return true;
        }
    }
    archive_addr = format!("{}{}", relpath, archive_fname);

    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("wget {}", archive_addr))
        .output();

    //------------------------------------------------------------------------//
    // Make directory structure for this emulator.                            //
    //------------------------------------------------------------------------//
    let _ = Command::new("sh")
        .arg("-c")
        .arg("mkdir -p $HOME/retroveneer/emulators/tic80")
        .output();

    let _ = Command::new("sh")
        .arg("-c")
        .arg("mkdir -p $HOME/retroveneer/data/tic80")
        .output();

    //------------------------------------------------------------------------//
    // Unpack emulator into aformentioned directory.                          //
    //------------------------------------------------------------------------//
    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("dpkg-deb -xv {} $HOME/retroveneer/emulators/tic80", archive_fname))
        .output();

    //------------------------------------------------------------------------//
    // Make a directory for emualtor archives.                                //
    //------------------------------------------------------------------------//
    let _ = Command::new("sh")
        .arg("-c")
        .arg("mkdir -p $HOME/retroveneer/emulators/archives")
        .output();

    //------------------------------------------------------------------------//
    // Move the archive into the archives directory.                          //
    //------------------------------------------------------------------------//
    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("mv {} $HOME/retroveneer/emulators/archives", archive_fname))
        .output();

    return false;
}

fn get_retrovaneer_ui() -> bool {
    let appimg_addr: String;
    let appimg_fname: String;
    let relpath: &str =
        "https://raw.githubusercontent.com/ModernRetroDev/retro-veneer/refs/heads/master/hosted/";

    unsafe {
        STEP_NEXT = "Installing RetroVeneer UI...".to_string();
    }

    //------------------------------------------------------------------------//
    // Download a copy of the appropriate appimage.                           //
    //------------------------------------------------------------------------//
    match std::env::consts::ARCH {
        "x86_64" => {
            appimg_fname = "retroveneer-ui-x86_64.AppImage".to_string();
        },
        "aarch64" => {
            appimg_fname = "retroveneer-ui-aarch64.AppImage".to_string();
        },
        _ => {
            println!("Unsupported architecture! Aborting!!!");
            return true;
        }
    }
    appimg_addr = format!("{}{}", relpath, appimg_fname);

    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("wget {}", appimg_addr))
        .output();

    //------------------------------------------------------------------------//
    // Set the executable flag for the appimage.                              //
    //------------------------------------------------------------------------//
    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("chmod +x {}", appimg_fname))
        .output();

    //------------------------------------------------------------------------//
    // Make directory structure for the retroveneer-ui.                       //
    //------------------------------------------------------------------------//
    let _ = Command::new("sh")
        .arg("-c")
        .arg("mkdir -p $HOME/retroveneer")
        .output();

    //------------------------------------------------------------------------//
    // Move the appimage into the retroveneer directory.                      //
    //------------------------------------------------------------------------//
    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("mv {} $HOME/retroveneer/retroveneer.appimage", appimg_fname))
        .output();

    return false;
}

fn setup_autostart() {
    let homedir = format!("{}", dirs::home_dir().unwrap().display());
    println!("homedir = `{}`", homedir);
    let autostart_path = format!("{homedir}/.config/autostart");

    println!("autostart_path = `{}`", autostart_path);
    fs::create_dir_all(&autostart_path).unwrap();

    let autostart_script = format!("{autostart_path}/retroveneer.desktop");
    let binpath_rv_ui = format!("{homedir}/retroveneer");

    let mut autofile = File::create(autostart_script).unwrap();

    writeln!(&mut autofile, "[Desktop Entry]").unwrap();
    writeln!(&mut autofile, "Type=Application").unwrap();
    writeln!(&mut autofile, "Version=0.1").unwrap();
    writeln!(&mut autofile, "Name=RetroVeneer").unwrap();
    writeln!(&mut autofile, "Comment=Manages an instance of RetroVeneer").unwrap();
    writeln!(&mut autofile, "Exec=/home/pi/retroveneer/retroveneer.appimage").unwrap();
    writeln!(&mut autofile, "Icon=/home/pi/retroveneer/retroveneer.png").unwrap();
    writeln!(&mut autofile, "Terminal=false").unwrap();
    writeln!(&mut autofile, "Categories=Utility;Emulation;").unwrap();
    writeln!(&mut autofile, "StartupNotify=true").unwrap();

    autofile.flush().unwrap();

    //------------------------------------------------------------------------//
    // Copy the .desktop file so that it shows up within launchers.           //
    //------------------------------------------------------------------------//
    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("cp {autostart_script} $HOME/.local/share/applications"))
        .output();

}

async fn update_spinner() {
    loop {
        sleep(Duration::from_millis(150)).await;

        unsafe {
            SPINNER_NEXT += 1;
            if SPINNER_NEXT > 3 {
                SPINNER_NEXT = 0;
            }
        }
    }
}

fn mode_splash(rl: &mut RaylibHandle, thread: &RaylibThread) {
    let stepmsg: String;
    let spinnertxt: String;

    let mut d = rl.begin_drawing(&thread);

    d.clear_background(Color::BLACK);
    d.draw_text("Installing RETRO VENEER", 125, 24, 30, Color::GRAY);

    unsafe {
        if STEP_NEXT.len() > 0 {
            STEP = STEP_NEXT.clone();
            STEP_NEXT = "".to_string();
        }
        if SPINNER_NEXT != SPINNER {
            SPINNER = SPINNER_NEXT;
        }
    }
    unsafe {
        stepmsg = STEP.clone();
        spinnertxt = match SPINNER {
            0 => "/".to_string(),
            1 => "--".to_string(),
            2 => "\\".to_string(),
            3 => " |".to_string(),
            _ => "/".to_string(),
        }
    }
    d.draw_text(&stepmsg, 30, 225, 30, Color::WHITE);
    d.draw_text(&spinnertxt, 585, 425, 30, Color::GRAY);
}

async fn install_in_bg() {
    if get_emu_x16() {
        return
    }
    if get_emu_tic80() {
        return
    }
    if get_retrovaneer_ui() {
        return
    }
    setup_autostart();

    unsafe {
        STEP_NEXT = "All Done!!!".to_string();
    }
    sleep(Duration::from_secs(2)).await;
    unsafe {
        EXIT_NOW = true;
    }
}

#[tokio::main]
async fn main() {
    let (mut rl, thread) = raylib::init()
        .size(640, 480)
        .title("Retro Veneer Installer")
        .build();

    tokio::spawn(update_spinner());
    tokio::spawn(install_in_bg());

    rl.set_target_fps(60);
    // rl.toggle_fullscreen();
    while !rl.window_should_close() {
        mode_splash(&mut rl, &thread);

        unsafe {
            if EXIT_NOW {
                break;
            }
        }
    }
}