use dirs;
use raylib::prelude::*;
use raylib::ffi::KeyboardKey;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use tokio;
use tokio::time::{sleep, Duration};


#[derive(Copy, Clone)]
enum RunMode {
    About,
    AboutRetroVeneer,
    AboutLicense,
    AboutTheEmulators,
    AboutDirectories,
    Configure,
    Countdown,
    EmulatorRunning,
    UpdateEverything,
    SelectPlatform,
}

#[derive(Copy, Clone, PartialEq)]
enum ConfigSelection {
    SelectPlatform,
    ChangeAutostart,
    UpdateEverything,
    AboutRetroVeneer,
    //
    LaunchEmulator,
    ExitRetroVeneer,
}

#[derive(Copy, Clone, PartialEq)]
enum PlatformSelection {
    Invalid,
    CommanderX16,
    Tic80,
}

#[derive(Copy, Clone, PartialEq)]
enum AboutSelection {
    RetroVeneer,
    License,
    TheEmulators,
    Directories,
    Back,
}

#[derive(Copy, Clone, PartialEq)]
enum UpdateSelection {
    InstallUpdates,
    Back,
}


static VERSION: &str = "0.1";
static URL_RETROVENEER: &str = 
    "https://raw.githubusercontent.com/ModernRetroDev/retro-veneer/refs/heads/master/hosted";

static mut COUNTDOWN_TICS: u8 = 0;
static mut COUNTDOWN_SECONDS: u8 = 9;

static mut EXIT_NOW: bool = false;
static mut RUNMODE: RunMode = RunMode::Countdown;
static mut ABOUT_SELECTION: AboutSelection = 
    AboutSelection::RetroVeneer;
static mut CONFIG_SELECTION: ConfigSelection = 
    ConfigSelection::SelectPlatform;
static mut PLATFORM_SELECTION: PlatformSelection = 
    PlatformSelection::CommanderX16;
static mut UPDATE_SELECTION: UpdateSelection = 
    UpdateSelection::Back;

static mut SPINNER:      u8 = 0;
static mut SPINNER_NEXT: u8 = 0;
static mut AUTOSTART_ENABLED: bool = false;
static mut UPDATES_AVAILABLE: bool = false;
static mut NETWORKING_ERROR:  bool = false;
static mut INSTALL_FREEZE:    bool = false;


fn glob_get_platform_selection() -> PlatformSelection {
    unsafe {
        return PLATFORM_SELECTION;
    }
}

fn glob_get_about_selection() -> AboutSelection {
    unsafe {
        return ABOUT_SELECTION;
    }
}

fn glob_get_update_selection() -> UpdateSelection {
    unsafe {
        return UPDATE_SELECTION;
    }
}

fn autostart_is_enabled() -> bool {
    let homedir = format!("{}", dirs::home_dir().unwrap().display());
    let autostart_path = format!("{homedir}/.config/autostart");
    let autostart_script = format!("{autostart_path}/retroveneer.desktop");

    return Path::new(&autostart_script).exists();
}

fn config_get_platform() -> PlatformSelection {
    let platform: PlatformSelection;
    let homedir = format!("{}", dirs::home_dir().unwrap().display());
    let rv_path = format!("{homedir}/.config/retroveneer");
    let cfg_platform = format!("{rv_path}/platform");

    if !Path::new(&cfg_platform).exists() {
        return PlatformSelection::Invalid;
    }

    let contents = fs::read_to_string(&cfg_platform).unwrap();

    match contents.as_ref() {
        "CommanderX16\n" => {
            platform = PlatformSelection::CommanderX16;
        },
        "TIC-80\n" => {
            platform = PlatformSelection::Tic80;
        }
        _ => {
            platform = PlatformSelection::Invalid;
        }
    }

    return platform;
}

fn enable_autostart() {
    let homedir = format!("{}", dirs::home_dir().unwrap().display());
    let autostart_path = format!("{homedir}/.config/autostart");

    fs::create_dir_all(&autostart_path).unwrap();

    let autostart_script = format!("{autostart_path}/retroveneer.desktop");
    let retroveneer_path = format!("{homedir}/retroveneer");

    let mut autofile = File::create(&autostart_script).unwrap();

    writeln!(&mut autofile, "[Desktop Entry]").unwrap();
    writeln!(&mut autofile, "Type=Application").unwrap();
    writeln!(&mut autofile, "Version=0.1").unwrap();
    writeln!(&mut autofile, "Name=RetroVeneer").unwrap();
    writeln!(&mut autofile, "Comment=Manages an instance of RetroVeneer").unwrap();
    writeln!(&mut autofile, "Exec={retroveneer_path}/retroveneer.appimage").unwrap();
    writeln!(&mut autofile, "Icon={retroveneer_path}/retroveneer.png").unwrap();
    writeln!(&mut autofile, "Terminal=false").unwrap();
    writeln!(&mut autofile, "Categories=Utility;Emulation;").unwrap();
    writeln!(&mut autofile, "StartupNotify=true").unwrap();

    autofile.flush().unwrap();
}

fn disable_autostart() {
    let homedir = format!("{}", dirs::home_dir().unwrap().display());
    let autostart_path = format!("{homedir}/.config/autostart");
    let autostart_script = format!("{autostart_path}/retroveneer.desktop");

    fs::remove_file(autostart_script).unwrap();
}

fn config_save_platform() {
    let homedir = format!("{}", dirs::home_dir().unwrap().display());
    let rv_path = format!("{homedir}/.config/retroveneer");
    let platform_str: String;

    fs::create_dir_all(&rv_path).unwrap();

    let cfg_platform = format!("{rv_path}/platform");

    let mut file_platform = File::create(cfg_platform).unwrap();

    match glob_get_platform_selection() {
        PlatformSelection::CommanderX16 => {
            platform_str = "CommanderX16".to_string();
        },
        PlatformSelection::Tic80 => {
            platform_str = "TIC-80".to_string();
        },
        PlatformSelection::Invalid => {
            platform_str = "".to_string();
        },
    }
    writeln!(&mut file_platform, "{}", &platform_str).unwrap();

    file_platform.flush().unwrap();
}

fn touch_rv_running_file() {
    let homedir = format!("{}", dirs::home_dir().unwrap().display());
    let temp_path = format!("{homedir}/retroveneer/.temp");
    let status_path = format!("{temp_path}/rv_is_running");

    fs::create_dir_all(&temp_path).unwrap();

    let _file_status = File::create(status_path).unwrap();
}

fn remove_rv_running_file() {
    let homedir = format!("{}", dirs::home_dir().unwrap().display());
    let temp_path = format!("{homedir}/retroveneer/.temp");
    let status_path = format!("{temp_path}/rv_is_running");

    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("rm -f {}", status_path))
        .output();
}

fn updates_are_available() -> bool {
    let homedir     = format!("{}", dirs::home_dir().unwrap().display());
    let temp_path   = format!("{homedir}/retroveneer/.temp");
    let vers_path   = format!("{temp_path}/current_version");
    let freeze_path = format!("{temp_path}/install_freeze");
    let url_freeze  = format!("{URL_RETROVENEER}/install_freeze");
    let url_version = format!("{URL_RETROVENEER}/current_version");

    fs::create_dir_all(&temp_path).unwrap();

    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("rm -f {vers_path} {freeze_path}"))
        .output();

    //------------------------------------------------------------------------//
    // Grab the latest version number of RV from the server.                  //
    //------------------------------------------------------------------------//
    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("wget {url_version} -P {temp_path}"))
        .output();

    if !Path::new(&vers_path).exists() {
        // network likely unavailable...
        unsafe {
            NETWORKING_ERROR = true;
        }
        return false;
    }

    //------------------------------------------------------------------------//
    // Grab the install_freeze file from the server.                          //
    //------------------------------------------------------------------------//
    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("wget {url_freeze} -P {temp_path}"))
        .output();

    if Path::new(&freeze_path).exists() {
        let contents = fs::read_to_string(&freeze_path).unwrap();

        if &contents == "TRUE\n" {
            unsafe {
                INSTALL_FREEZE = true;
            }
            return false;
        }
    }

    let contents = fs::read_to_string(&vers_path).unwrap();

    if &contents != VERSION {
        return true;
    }
    return false;
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


fn switch_to_mode_about() {
    unsafe {
        RUNMODE = RunMode::About;
        ABOUT_SELECTION = AboutSelection::RetroVeneer;
    }
}

fn switch_to_mode_about_retroveneer() {
    unsafe {
        RUNMODE = RunMode::AboutRetroVeneer;
    }
}

fn switch_to_mode_about_license() {
    unsafe {
        RUNMODE = RunMode::AboutLicense;
    }
}

fn switch_to_mode_about_the_emulators() {
    unsafe {
        RUNMODE = RunMode::AboutTheEmulators;
    }
}

fn switch_to_mode_about_directories() {
    unsafe {
        RUNMODE = RunMode::AboutDirectories;
    }
}

fn switch_to_mode_configure() {
    unsafe {
        RUNMODE = RunMode::Configure;
        CONFIG_SELECTION = ConfigSelection::SelectPlatform;
    }
}

fn switch_to_mode_select_platform() {
    unsafe {
        RUNMODE = RunMode::SelectPlatform;
        PLATFORM_SELECTION = PlatformSelection::CommanderX16;
    }
}

fn switch_to_mode_emulator_running() {
    unsafe {
        RUNMODE = RunMode::EmulatorRunning;
    }
}

fn switch_to_mode_update_everything() {
    unsafe {
        RUNMODE           = RunMode::UpdateEverything;
        UPDATES_AVAILABLE = updates_are_available();
    }
}


fn mode_about(rl: &mut RaylibHandle, thread: &RaylibThread) {
    let spinnertxt: String;
    let mut selection: AboutSelection;
    let mut selection_new: Option<AboutSelection> = None;
    let spinner_x: i32 =  16;
    let mut spinner_y: i32 = 116;
    let s_x: i32 = 35; // shift x
    let s_y: i32 = 70; // shift y
    let mut text_option: String;
    let mut y_base: i32;

    unsafe {
        if SPINNER_NEXT != SPINNER {
            SPINNER = SPINNER_NEXT;
        }
    }
    unsafe {
        spinnertxt = match SPINNER {
            0 => "/".to_string(),
            1 => "--".to_string(),
            2 => "\\".to_string(),
            3 => " |".to_string(),
            _ => "/".to_string(),
        }
    }
    selection = glob_get_about_selection();

    let pressed_key = rl.get_key_pressed();

    if let Some(pressed_key) = pressed_key {
        if pressed_key == KeyboardKey::KEY_DOWN {
            match selection {
                AboutSelection::RetroVeneer => {
                    selection_new = Some(AboutSelection::License);
                },
                AboutSelection::License => {
                    selection_new = Some(AboutSelection::TheEmulators);
                },
                AboutSelection::TheEmulators => {
                    selection_new = Some(AboutSelection::Directories);
                },
                AboutSelection::Directories => {
                    selection_new = Some(AboutSelection::Back);
                },
                AboutSelection::Back => {},
            }
        }
        if pressed_key == KeyboardKey::KEY_UP {
            match selection {
                AboutSelection::RetroVeneer => {},
                AboutSelection::License => {
                    selection_new = Some(AboutSelection::RetroVeneer);
                },
                AboutSelection::TheEmulators => {
                    selection_new = Some(AboutSelection::License);
                },
                AboutSelection::Directories => {
                    selection_new = Some(AboutSelection::TheEmulators);
                },
                AboutSelection::Back => {
                    selection_new = Some(AboutSelection::Directories);
                },
            }
        }
        if pressed_key == KeyboardKey::KEY_ESCAPE {
            switch_to_mode_configure();
            return;
        }
        if pressed_key == KeyboardKey::KEY_ENTER {
            match selection {
                AboutSelection::RetroVeneer => {
                    switch_to_mode_about_retroveneer();
                    return;
                },
                AboutSelection::License => {
                    switch_to_mode_about_license();
                    return;
                },
                AboutSelection::TheEmulators => {
                    switch_to_mode_about_the_emulators();
                    return;
                },
                AboutSelection::Directories => {
                    switch_to_mode_about_directories();
                    return;
                },
                AboutSelection::Back => {
                    switch_to_mode_configure();
                    return;
                },
            }
        }
    }

    match selection_new {
        Some(selection_new) => {
            unsafe {
                selection = selection_new;
                ABOUT_SELECTION = selection;
            }
        },
        None => {},
    }

    let mut d = rl.begin_drawing(&thread);

    d.clear_background(Color::BLACK);
    d.draw_text("RETRO VENEER", 12, 12, 40, Color::GRAY);
    d.draw_text(&format!("v{}", VERSION), 580, 12, 20, Color::GRAY);
    d.draw_line(6,58,632,58, Color::GRAY);

    d.draw_text("ABOUT", 12, 70, 40, Color::BLUE);


    y_base = 96;
    text_option = "RETROVENEER".to_string();
    if selection == AboutSelection::RetroVeneer {
        spinner_y = y_base+s_y;
        d.draw_text(&text_option,  62+s_x, y_base+s_y+2, 30, Color::RED);
        d.draw_text(&text_option,  60+s_x, y_base+s_y, 30, Color::YELLOW);
    } else {
        d.draw_text(&text_option,  60+s_x, y_base+s_y, 30, Color::WHITE);
    }

    y_base += 40;
    text_option = "CODE LICENSE".to_string();
    if selection == AboutSelection::License {
        spinner_y = y_base+s_y;
        d.draw_text(&text_option,  62+s_x, y_base+s_y+2, 30, Color::RED);
        d.draw_text(&text_option,  60+s_x, y_base+s_y, 30, Color::YELLOW);
    } else {
        d.draw_text(&text_option,  60+s_x, y_base+s_y, 30, Color::WHITE);
    }

    y_base += 40;
    text_option = "THE EMULATORS".to_string();
    if selection == AboutSelection::TheEmulators {
        spinner_y = y_base+s_y;
        d.draw_text(&text_option,  62+s_x, y_base+s_y+2, 30, Color::RED);
        d.draw_text(&text_option,  60+s_x, y_base+s_y, 30, Color::YELLOW);
    } else {
        d.draw_text(&text_option,  60+s_x, y_base+s_y, 30, Color::WHITE);
    }

    y_base += 40;
    text_option = "DIRECTORIES".to_string();
    if selection == AboutSelection::Directories {
        spinner_y = y_base+s_y;
        d.draw_text(&text_option,  62+s_x, y_base+s_y+2, 30, Color::RED);
        d.draw_text(&text_option,  60+s_x, y_base+s_y, 30, Color::YELLOW);
    } else {
        d.draw_text(&text_option,  60+s_x, y_base+s_y, 30, Color::WHITE);
    }

    y_base += 80;
    text_option = "BACK".to_string();
    if selection == AboutSelection::Back {
        spinner_y = y_base+s_y;
        d.draw_text(&text_option,  62+s_x, y_base+s_y+2, 30, Color::RED);
        d.draw_text(&text_option,  60+s_x, y_base+s_y, 30, Color::YELLOW);
    } else {
        d.draw_text(&text_option,  60+s_x, y_base+s_y, 30, Color::GRAY);
    }


    d.draw_line(6,442,632,442, Color::GRAY);
    d.draw_text("by: ModernRetroDev", 420, 450, 20, Color::GRAY);

    d.draw_text(&spinnertxt, spinner_x+2+s_x, spinner_y+2, 30, Color::RED);
    d.draw_text(&spinnertxt, spinner_x+s_x, spinner_y, 30, Color::YELLOW);
}

fn mode_about_retroveneer(rl: &mut RaylibHandle, thread: &RaylibThread) {
    let spinnertxt: String;
    let spinner_x: i32 =  16;
    let spinner_y: i32;
    let s_x: i32 = 35; // shift x
    let s_y: i32 = 70; // shift y

    let mut text_ln: String;
    let text_option: String;
    let mut y_base: i32;

    unsafe {
        if SPINNER_NEXT != SPINNER {
            SPINNER = SPINNER_NEXT;
        }
    }
    unsafe {
        spinnertxt = match SPINNER {
            0 => "/".to_string(),
            1 => "--".to_string(),
            2 => "\\".to_string(),
            3 => " |".to_string(),
            _ => "/".to_string(),
        }
    }

    let pressed_key = rl.get_key_pressed();

    if let Some(pressed_key) = pressed_key {
        if pressed_key == KeyboardKey::KEY_ENTER {
            switch_to_mode_about();
        }
        if pressed_key == KeyboardKey::KEY_ESCAPE {
            switch_to_mode_about();
        }
    }

    let mut d = rl.begin_drawing(&thread);

    d.clear_background(Color::BLACK);
    d.draw_text("RETRO VENEER", 12, 12, 40, Color::GRAY);
    d.draw_text(&format!("v{}", VERSION), 580, 12, 20, Color::GRAY);
    d.draw_line(6,58,632,58, Color::GRAY);

    d.draw_text("ABOUT RETRO VENEER", 12, 70, 40, Color::BLUE);


    y_base = 75;
    text_ln = "RetroVeneer is a set of codes/scripts to simplify the".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, 20, Color::WHITE);

    y_base += 25;
    text_ln = "process of setting up and keeping up-to-date a set of".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, 20, Color::WHITE);

    y_base += 25;
    text_ln = "emulators for modern retrocomputers.".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, 20, Color::WHITE);


    y_base += 45;
    text_ln = "If you enjoyed this project, please follow me at:".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, 20, Color::WHITE);

    y_base += 35;
    text_ln = "* YouTube: @ModernRetroDev".to_string();
    d.draw_text(&text_ln, s_x+10, y_base+s_y, 20, Color::GRAY);

    y_base += 35;
    text_ln = "* GitHub: https://github.com/ModernRetroDev".to_string();
    d.draw_text(&text_ln, s_x+10, y_base+s_y, 20, Color::GRAY);

    y_base += 35;
    text_ln = "Thanks! ~~ Mike".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, 20, Color::WHITE);


    text_option = "BACK".to_string();
    y_base = 331;

    spinner_y = y_base+s_y;
    d.draw_text(&text_option,  62+s_x, y_base+s_y+2, 30, Color::RED);
    d.draw_text(&text_option,  60+s_x, y_base+s_y, 30, Color::YELLOW);


    d.draw_line(6,442,632,442, Color::GRAY);
    d.draw_text("by: ModernRetroDev", 420, 450, 20, Color::GRAY);

    d.draw_text(&spinnertxt, spinner_x+2+s_x, spinner_y+2, 30, Color::RED);
    d.draw_text(&spinnertxt, spinner_x+s_x, spinner_y, 30, Color::YELLOW);
}

fn mode_about_license(rl: &mut RaylibHandle, thread: &RaylibThread) {
    let spinnertxt: String;
    let spinner_x: i32 =  16;
    let spinner_y: i32;
    let s_x: i32 = 35; // shift x
    let s_y: i32 = 70; // shift y

    let mut text_ln: String;
    let text_option: String;
    let mut y_base: i32;

    unsafe {
        if SPINNER_NEXT != SPINNER {
            SPINNER = SPINNER_NEXT;
        }
    }
    unsafe {
        spinnertxt = match SPINNER {
            0 => "/".to_string(),
            1 => "--".to_string(),
            2 => "\\".to_string(),
            3 => " |".to_string(),
            _ => "/".to_string(),
        }
    }

    let pressed_key = rl.get_key_pressed();

    if let Some(pressed_key) = pressed_key {
        if pressed_key == KeyboardKey::KEY_ENTER {
            switch_to_mode_about();
        }
        if pressed_key == KeyboardKey::KEY_ESCAPE {
            switch_to_mode_about();
        }
    }

    let mut d = rl.begin_drawing(&thread);

    d.clear_background(Color::BLACK);
    d.draw_text("RETRO VENEER", 12, 12, 40, Color::GRAY);
    d.draw_text(&format!("v{}", VERSION), 580, 12, 20, Color::GRAY);
    d.draw_line(6,58,632,58, Color::GRAY);

    d.draw_text("SOFTWARE LICENSE", 12, 70, 40, Color::BLUE);

    y_base = 56;
    text_ln = "Copyright (c) 2025 <Mike AKA: ModernRetroDev>".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, 10, Color::WHITE);

    y_base += 30;
    text_ln = "Permission is hereby granted, free of charge, to any person obtaining a copy of".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, 10, Color::WHITE);
    y_base += 15;
    text_ln = "this software and associated documentation files (the \"Software\"), to deal in.".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, 10, Color::WHITE);
    y_base += 15;
    text_ln = "the Software without restriction, including without limitation the rights to".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, 10, Color::WHITE);
    y_base += 15;
    text_ln = "use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, 10, Color::WHITE);
    y_base += 15;
    text_ln = "the Software, and to permit persons to whom the Software is furnished to do so,".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, 10, Color::WHITE);
    y_base += 15;
    text_ln = "subject to the following conditions:".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, 10, Color::WHITE);

    y_base += 30;
    text_ln = "The above copyright notice and this permission notice shall be included in all".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, 10, Color::WHITE);
    y_base += 15;
    text_ln = "copies or substantial portions of the Software.".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, 10, Color::WHITE);

    y_base += 30;
    text_ln = "THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, 10, Color::WHITE);
    y_base += 15;
    text_ln = "IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, 10, Color::WHITE);
    y_base += 15;
    text_ln = "FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, 10, Color::WHITE);
    y_base += 15;
    text_ln = "COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, 10, Color::WHITE);
    y_base += 15;
    text_ln = "IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, 10, Color::WHITE);
    y_base += 15;
    text_ln = "CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, 10, Color::WHITE);

    text_option = "BACK".to_string();
    y_base = 331;

    spinner_y = y_base+s_y;
    d.draw_text(&text_option,  62+s_x, y_base+s_y+2, 30, Color::RED);
    d.draw_text(&text_option,  60+s_x, y_base+s_y, 30, Color::YELLOW);

    d.draw_line(6,442,632,442, Color::GRAY);
    d.draw_text("by: ModernRetroDev", 420, 450, 20, Color::GRAY);

    d.draw_text(&spinnertxt, spinner_x+2+s_x, spinner_y+2, 30, Color::RED);
    d.draw_text(&spinnertxt, spinner_x+s_x, spinner_y, 30, Color::YELLOW);
}

fn mode_about_the_emulators(rl: &mut RaylibHandle, thread: &RaylibThread) {
    let spinnertxt: String;
    let spinner_x: i32 =  16;
    let spinner_y: i32;
    let txt_sz: i32 = 20;
    let s_x: i32 = 15; // shift x
    let s_y: i32 = 70; // shift y

    let mut text_ln: String;
    let text_option: String;
    let mut y_base: i32;

    unsafe {
        if SPINNER_NEXT != SPINNER {
            SPINNER = SPINNER_NEXT;
        }
    }
    unsafe {
        spinnertxt = match SPINNER {
            0 => "/".to_string(),
            1 => "--".to_string(),
            2 => "\\".to_string(),
            3 => " |".to_string(),
            _ => "/".to_string(),
        }
    }

    let pressed_key = rl.get_key_pressed();

    if let Some(pressed_key) = pressed_key {
        if pressed_key == KeyboardKey::KEY_ENTER {
            switch_to_mode_about();
        }
        if pressed_key == KeyboardKey::KEY_ESCAPE {
            switch_to_mode_about();
        }
    }

    let mut d = rl.begin_drawing(&thread);

    d.clear_background(Color::BLACK);
    d.draw_text("RETRO VENEER", 12, 12, 40, Color::GRAY);
    d.draw_text(&format!("v{}", VERSION), 580, 12, 20, Color::GRAY);
    d.draw_line(6,58,632,58, Color::GRAY);

    d.draw_text("ABOUT THE EMULATORS", 12, 70, 40, Color::BLUE);


    y_base = 70;
    text_ln = "The various platform emulators installed by RetroVeneer".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, txt_sz, Color::WHITE);

    y_base += 25;
    text_ln = "all utilize their own licenses. The archives from which".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, txt_sz, Color::WHITE);

    y_base += 25;
    text_ln = "they are installed should all include these licenses.".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, txt_sz, Color::WHITE);


    y_base += 45;
    text_ln = "Archives used for emulator installation are stored here:".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, txt_sz, Color::WHITE);

    y_base += 35;
    text_ln = "$HOME/retroveneer/emulator/archives".to_string();
    d.draw_text(&text_ln, s_x+40, y_base+s_y, txt_sz, Color::GRAY);


    y_base += 45;
    text_ln = "If you enjoy the platforms being emulated, please support".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, txt_sz, Color::GREEN);

    y_base += 35;
    text_ln = "these projects by purchasing actual hardware (if possible).".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, txt_sz, Color::GREEN);


    text_option = "BACK".to_string();
    y_base = 331;

    spinner_y = y_base+s_y;
    d.draw_text(&text_option,  62+s_x, y_base+s_y+2, 30, Color::RED);
    d.draw_text(&text_option,  60+s_x, y_base+s_y, 30, Color::YELLOW);


    d.draw_line(6,442,632,442, Color::GRAY);
    d.draw_text("by: ModernRetroDev", 420, 450, 20, Color::GRAY);

    d.draw_text(&spinnertxt, spinner_x+2+s_x, spinner_y+2, 30, Color::RED);
    d.draw_text(&spinnertxt, spinner_x+s_x, spinner_y, 30, Color::YELLOW);
}

fn mode_about_directories(rl: &mut RaylibHandle, thread: &RaylibThread) {
    let spinnertxt: String;
    let spinner_x: i32 =  16;
    let spinner_y: i32;
    let txt_sz: i32 = 20;
    let s_x: i32 = 20; // shift x
    let s_y: i32 = 70; // shift y

    let mut text_ln: String;
    let text_option: String;
    let mut y_base: i32;

    unsafe {
        if SPINNER_NEXT != SPINNER {
            SPINNER = SPINNER_NEXT;
        }
    }
    unsafe {
        spinnertxt = match SPINNER {
            0 => "/".to_string(),
            1 => "--".to_string(),
            2 => "\\".to_string(),
            3 => " |".to_string(),
            _ => "/".to_string(),
        }
    }

    let pressed_key = rl.get_key_pressed();

    if let Some(pressed_key) = pressed_key {
        if pressed_key == KeyboardKey::KEY_ENTER {
            switch_to_mode_about();
        }
        if pressed_key == KeyboardKey::KEY_ESCAPE {
            switch_to_mode_about();
        }
    }

    let mut d = rl.begin_drawing(&thread);

    d.clear_background(Color::BLACK);
    d.draw_text("RETRO VENEER", 12, 12, 40, Color::GRAY);
    d.draw_text(&format!("v{}", VERSION), 580, 12, 20, Color::GRAY);
    d.draw_line(6,58,632,58, Color::GRAY);

    d.draw_text("ABOUT DIRECTORIES", 12, 70, 40, Color::BLUE);


    y_base = 70;
    text_ln = "Files installed by RetroVeneer end up within the path:".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, txt_sz, Color::WHITE);

    y_base += 35;
    text_ln = "$HOME/retroveneer".to_string();
    d.draw_text(&text_ln, s_x+40, y_base+s_y, txt_sz, Color::GRAY);


    y_base += 35;
    text_ln = "Most importantly, platform directories located within:".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, txt_sz, Color::WHITE);

    y_base += 35;
    text_ln = "$HOME/retroveneer/data".to_string();
    d.draw_text(&text_ln, s_x+40, y_base+s_y, txt_sz, Color::GRAY);


    y_base += 35;
    text_ln = "are mounted by the emulators when they are started. So".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, txt_sz, Color::WHITE);

    y_base += 35;
    text_ln = "if you wish to make code and or programs accessible to".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, txt_sz, Color::WHITE);

    y_base += 35;
    text_ln = "the emulators. Save them within these directories.".to_string();
    d.draw_text(&text_ln, s_x, y_base+s_y, txt_sz, Color::WHITE);


    text_option = "BACK".to_string();
    y_base = 331;

    spinner_y = y_base+s_y;
    d.draw_text(&text_option,  62+s_x, y_base+s_y+2, 30, Color::RED);
    d.draw_text(&text_option,  60+s_x, y_base+s_y, 30, Color::YELLOW);


    d.draw_line(6,442,632,442, Color::GRAY);
    d.draw_text("by: ModernRetroDev", 420, 450, 20, Color::GRAY);

    d.draw_text(&spinnertxt, spinner_x+2+s_x, spinner_y+2, 30, Color::RED);
    d.draw_text(&spinnertxt, spinner_x+s_x, spinner_y, 30, Color::YELLOW);
}

fn mode_countdown(rl: &mut RaylibHandle, thread: &RaylibThread) {
    let startmsg: String;
    let platform_str: String;
    let s_x: i32 = 35; // shift x
    let s_y: i32 = 5; // shift y

    let pressed_key = rl.get_key_pressed();
    let mut d = rl.begin_drawing(&thread);

    match glob_get_platform_selection() {
        PlatformSelection::CommanderX16 => {
            platform_str = "[COMMANDER X16]".to_string();
        },
        PlatformSelection::Tic80 => {
            platform_str = "[TIC-80]".to_string();
        },
        PlatformSelection::Invalid => {
            platform_str = "".to_string();
        },
    }

    d.clear_background(Color::BLACK);
    d.draw_text("RETRO VENEER", 12, 12, 40, Color::GRAY);
    d.draw_text(&format!("v{}", VERSION), 580, 12, 20, Color::GRAY);
    d.draw_line(6,58,632,58, Color::GRAY);

    d.draw_text(&platform_str, 142+s_x, 158+s_y, 30, Color::BLUE);
    d.draw_text(&platform_str, 140+s_x, 156+s_y, 30, Color::GRAY);


    d.draw_text("PUSH `ESC` TO CONFIGURE SYSTEM", 30, 225, 30, Color::WHITE);
    d.draw_text("ESC", 135, 225, 30, Color::RED);

    d.draw_line(6,442,632,442, Color::GRAY);
    d.draw_text("by: ModernRetroDev", 420, 450, 20, Color::GRAY);

    unsafe {
        startmsg = format!("Starting Emulation in {} seconds...", COUNTDOWN_SECONDS);
    }
    d.draw_text(&startmsg, 170, 320, 20, Color::RED);


    if let Some(pressed_key) = pressed_key {
        if pressed_key == KeyboardKey::KEY_ESCAPE {
            switch_to_mode_configure();
        }
    }
    unsafe {
        COUNTDOWN_TICS += 1;
        if COUNTDOWN_TICS >= 60 {
            COUNTDOWN_TICS = 0;
            COUNTDOWN_SECONDS -= 1;

            if COUNTDOWN_SECONDS == 0 {
                launch_platform_emulator();
                switch_to_mode_emulator_running();
                return;
            }
        }
    }
}

fn mode_emulator_running(rl: &mut RaylibHandle, thread: &RaylibThread) {
    let mut d = rl.begin_drawing(&thread);

    d.clear_background(Color::BLACK);
    d.draw_text("RETRO VENEER", 12, 12, 40, Color::GRAY);
    d.draw_text(&format!("v{}", VERSION), 580, 12, 20, Color::GRAY);
    d.draw_line(6,58,632,58, Color::GRAY);

    d.draw_text("EMU RUNNING; RESET TO STOP.", 75, 225, 30, Color::WHITE);

    d.draw_line(6,442,632,442, Color::GRAY);
    d.draw_text("by: ModernRetroDev", 420, 450, 20, Color::GRAY);
}

fn mode_configure(rl: &mut RaylibHandle, thread: &RaylibThread) {
    let autostart_enabled: bool;
    let platform_str: String;
    let spinnertxt: String;
    let mut selection: ConfigSelection;
    let mut selection_new: Option<ConfigSelection> = None;
    let spinner_x: i32 =  16;
    let mut spinner_y: i32 = 116;
    let mut y_base: i32;
    let mut y2_base: i32;
    let mut text_ln: String;
    let s_x: i32 = 35; // shift x
    let s_y: i32 = 5; // shift y

    unsafe {
        if SPINNER_NEXT != SPINNER {
            SPINNER = SPINNER_NEXT;
        }
    }
    unsafe {
        spinnertxt = match SPINNER {
            0 => "/".to_string(),
            1 => "--".to_string(),
            2 => "\\".to_string(),
            3 => " |".to_string(),
            _ => "/".to_string(),
        }
    }
    unsafe {
        selection = CONFIG_SELECTION;
    }
    unsafe {
        autostart_enabled = AUTOSTART_ENABLED;
    }

    let pressed_key = rl.get_key_pressed();


    if let Some(pressed_key) = pressed_key {
        if pressed_key == KeyboardKey::KEY_DOWN {
            match selection {
                ConfigSelection::SelectPlatform => {
                    selection_new = Some(ConfigSelection::ChangeAutostart);
                },
                ConfigSelection::ChangeAutostart => {
                    selection_new = Some(ConfigSelection::UpdateEverything);
                },
                ConfigSelection::UpdateEverything => {
                    selection_new = Some(ConfigSelection::AboutRetroVeneer);
                },
                ConfigSelection::AboutRetroVeneer => {
                    selection_new = Some(ConfigSelection::LaunchEmulator);
                },
                ConfigSelection::LaunchEmulator => {
                    selection_new = Some(ConfigSelection::ExitRetroVeneer);
                },
                ConfigSelection::ExitRetroVeneer => {},
            }
        }
        if pressed_key == KeyboardKey::KEY_UP {
            match selection {
                ConfigSelection::SelectPlatform => {},
                ConfigSelection::ChangeAutostart => {
                    selection_new = Some(ConfigSelection::SelectPlatform);
                },
                ConfigSelection::UpdateEverything => {
                    selection_new = Some(ConfigSelection::ChangeAutostart);
                },
                ConfigSelection::AboutRetroVeneer => {
                    selection_new = Some(ConfigSelection::UpdateEverything);
                },
                ConfigSelection::LaunchEmulator => {
                    selection_new = Some(ConfigSelection::AboutRetroVeneer);
                },
                ConfigSelection::ExitRetroVeneer => {
                    selection_new = Some(ConfigSelection::LaunchEmulator);
                },
            }
        }
        if pressed_key == KeyboardKey::KEY_ENTER {
            match selection {
                ConfigSelection::SelectPlatform => {
                    switch_to_mode_select_platform();
                    return;
                },
                ConfigSelection::ChangeAutostart => {
                    if autostart_enabled {
                        disable_autostart();
                        unsafe {
                            AUTOSTART_ENABLED = false;
                        }
                    } else {
                        enable_autostart();
                        unsafe {
                            AUTOSTART_ENABLED = true;
                        }
                    }
                },
                ConfigSelection::UpdateEverything => {
                    switch_to_mode_update_everything();
                    return;
                },
                ConfigSelection::AboutRetroVeneer => {
                    switch_to_mode_about();
                    return;
                },
                //------------------------------------------------------------//
                ConfigSelection::LaunchEmulator => {
                    switch_to_mode_emulator_running();
                    launch_platform_emulator();
                    return;
                },
                ConfigSelection::ExitRetroVeneer => {
                    unsafe {
                        EXIT_NOW = true;
                    }
                },
            }
        }
    }

    match selection_new {
        Some(selection_new) => {
            unsafe {
                selection = selection_new;
                CONFIG_SELECTION = selection;
            }
        },
        None => {},
    }

    match glob_get_platform_selection() {
        PlatformSelection::CommanderX16 => {
            platform_str = "* COMMANDER X16".to_string();
        },
        PlatformSelection::Tic80 => {
            platform_str = "* TIC-80".to_string();
        },
        PlatformSelection::Invalid => {
            platform_str = "".to_string();
        },
    }

    let mut d = rl.begin_drawing(&thread);

    d.clear_background(Color::BLACK);
    d.draw_text("RETRO VENEER", 12, 12, 40, Color::GRAY);
    d.draw_text(&format!("v{}", VERSION), 580, 12, 20, Color::GRAY);
    d.draw_line(6,58,632,58, Color::GRAY);

    d.draw_text("CONFIGURATION", 12, 70, 40, Color::BLUE);

    y_base = 116;
    y2_base = y_base + 40;
    text_ln = "SELECT PLATFORM".to_string();
    if selection == ConfigSelection::SelectPlatform {
        spinner_y = y_base+s_y;
        d.draw_text(&text_ln,      62+s_x, y_base+s_y+2,  30, Color::RED);
        d.draw_text(&text_ln,      60+s_x, y_base+s_y,    30, Color::YELLOW);
        d.draw_text(&platform_str, 92+s_x, y2_base+s_y+2, 30, Color::BLUE);
    } else {
        d.draw_text(&text_ln,      60+s_x, y_base+s_y,    30, Color::WHITE);
    }
    d.draw_text(&platform_str,     90+s_x, y2_base+s_y,   30, Color::GRAY);

    y_base += 80;
    y2_base = y_base + 40;
    text_ln = "ENABLE/DISABLE AUTOSTART".to_string();
    if selection == ConfigSelection::ChangeAutostart {
        spinner_y = y_base+s_y;
        d.draw_text(&text_ln, 62+s_x,  y_base+s_y+2, 30, Color::RED);
        d.draw_text(&text_ln, 60+s_x,  y_base+s_y,   30, Color::YELLOW);
        if autostart_enabled {
            d.draw_text("* ENABLED",  92+s_x, y2_base+s_y+2, 30, Color::BLUE);
        } else {
            d.draw_text("* DISABLED", 92+s_x, y2_base+s_y+2, 30, Color::BLUE);
        }
    } else {
        d.draw_text(&text_ln,     60+s_x, y_base+s_y,  30, Color::WHITE);
    }
    if autostart_enabled {
        d.draw_text("* ENABLED",  90+s_x, y2_base+s_y, 30, Color::GRAY);
    } else {
        d.draw_text("* DISABLED", 90+s_x, y2_base+s_y, 30, Color::GRAY);
    }

    y_base += 80;
    text_ln = "UPDATE EVERYTHING".to_string();
    if selection == ConfigSelection::UpdateEverything {
        spinner_y = y_base+s_y;
        d.draw_text(&text_ln, 62+s_x, y_base+s_y+2, 30, Color::RED);
        d.draw_text(&text_ln, 60+s_x, y_base+s_y,   30, Color::YELLOW);
    } else {
        d.draw_text(&text_ln, 60+s_x, y_base+s_y,   30, Color::WHITE);
    }

    y_base += 40;
    text_ln = "ABOUT RETRO VENEER".to_string();
    if selection == ConfigSelection::AboutRetroVeneer {
        spinner_y = y_base+s_y;
        d.draw_text(&text_ln, 62+s_x, y_base+s_y+2, 30, Color::RED);
        d.draw_text(&text_ln, 60+s_x, y_base+s_y,   30, Color::YELLOW);
    } else {
        d.draw_text(&text_ln, 60+s_x, y_base+s_y,   30, Color::WHITE);
    }


    y_base = 360;
    text_ln = "LAUNCH EMULATOR".to_string();
    if selection == ConfigSelection::LaunchEmulator {
        spinner_y = y_base+s_y;
        d.draw_text(&text_ln, 62+s_x, y_base+2, 30, Color::RED);
        d.draw_text(&text_ln, 60+s_x, y_base,   30, Color::YELLOW);
    } else {
        d.draw_text(&text_ln, 60+s_x, y_base,   30, Color::GREEN);
    }

    y_base += 40;
    text_ln = "EXIT RETRO VENEER".to_string();
    if selection == ConfigSelection::ExitRetroVeneer {
        spinner_y = y_base+s_y;
        d.draw_text(&text_ln, 62+s_x, y_base+2, 30, Color::RED);
        d.draw_text(&text_ln, 60+s_x, y_base,   30, Color::YELLOW);
    } else {
        d.draw_text(&text_ln, 60+s_x, y_base,   30, Color::RED);
    }

    d.draw_line(6,442,632,442, Color::GRAY);
    d.draw_text("by: ModernRetroDev", 420, 450, 20, Color::GRAY);

    d.draw_text(&spinnertxt, spinner_x+2+s_x, spinner_y+2, 30, Color::RED);
    d.draw_text(&spinnertxt, spinner_x+s_x,   spinner_y,   30, Color::YELLOW);
}

fn mode_select_platform(rl: &mut RaylibHandle, thread: &RaylibThread) {
    let spinnertxt: String;
    let mut selection: PlatformSelection;
    let mut selection_new: Option<PlatformSelection> = None;
    let spinner_x: i32 =  16;
    let mut spinner_y: i32 = 116;
    let s_x: i32 = 35; // shift x
    let s_y: i32 = 70; // shift y

    unsafe {
        if SPINNER_NEXT != SPINNER {
            SPINNER = SPINNER_NEXT;
        }
    }
    unsafe {
        spinnertxt = match SPINNER {
            0 => "/".to_string(),
            1 => "--".to_string(),
            2 => "\\".to_string(),
            3 => " |".to_string(),
            _ => "/".to_string(),
        }
    }
    selection = glob_get_platform_selection();

    let pressed_key = rl.get_key_pressed();


    if let Some(pressed_key) = pressed_key {
        if pressed_key == KeyboardKey::KEY_DOWN {
            match selection {
                PlatformSelection::CommanderX16 => {
                    selection_new = Some(PlatformSelection::Tic80);
                },
                PlatformSelection::Tic80 => {},
                PlatformSelection::Invalid => {},
            }
        }
        if pressed_key == KeyboardKey::KEY_UP {
            match selection {
                PlatformSelection::CommanderX16 => {},
                PlatformSelection::Tic80 => {
                    selection_new = Some(PlatformSelection::CommanderX16);
                },
                PlatformSelection::Invalid => {},
            }
        }
        if pressed_key == KeyboardKey::KEY_ENTER {
            config_save_platform();
            switch_to_mode_configure();
        }
    }

    match selection_new {
        Some(selection_new) => {
            unsafe {
                selection = selection_new;
                PLATFORM_SELECTION = selection;
            }
        },
        None => {},
    }

    let mut d = rl.begin_drawing(&thread);

    d.clear_background(Color::BLACK);
    d.draw_text("RETRO VENEER", 12, 12, 40, Color::GRAY);
    d.draw_text(&format!("v{}", VERSION), 580, 12, 20, Color::GRAY);
    d.draw_line(6,58,632,58, Color::GRAY);

    d.draw_text("SELECT PLATFORM", 12, 70, 40, Color::BLUE);

    if selection == PlatformSelection::CommanderX16 {
        spinner_y = 116+s_y;
        d.draw_text("COMMANDER X16",  62+s_x, 118+s_y, 30, Color::RED);
        d.draw_text("COMMANDER X16",  60+s_x, 116+s_y, 30, Color::YELLOW);
    } else {
        d.draw_text("COMMANDER X16",  60+s_x, 116+s_y, 30, Color::WHITE);
    }

    if selection == PlatformSelection::Tic80 {
        spinner_y = 196+s_y;
        d.draw_text("TIC-80", 62+s_x,  198+s_y, 30, Color::RED);
        d.draw_text("TIC-80", 60+s_x,  196+s_y, 30, Color::YELLOW);
    } else {
        d.draw_text("TIC-80", 60+s_x,  196+s_y, 30, Color::WHITE);
    }

    d.draw_line(6,442,632,442, Color::GRAY);
    d.draw_text("by: ModernRetroDev", 420, 450, 20, Color::GRAY);

    d.draw_text(&spinnertxt, spinner_x+2+s_x, spinner_y+2, 30, Color::RED);
    d.draw_text(&spinnertxt, spinner_x+s_x, spinner_y, 30, Color::YELLOW);
}

fn mode_update_everything(rl: &mut RaylibHandle, thread: &RaylibThread) {
    let spinnertxt: String;
    let netwk_err: bool;
    let updates_avail: bool;
    let update_freeze: bool;
    let mut selection: UpdateSelection;
    let mut selection_new: Option<UpdateSelection> = None;
    let spinner_x: i32 =  16;
    let mut spinner_y: i32 = 116;
    let s_x: i32 = 35; // shift x
    let s_y: i32 = 70; // shift y
    let mut text_option: String;
    let mut y_base: i32;

    unsafe {
        if SPINNER_NEXT != SPINNER {
            SPINNER = SPINNER_NEXT;
        }
    }
    unsafe {
        spinnertxt = match SPINNER {
            0 => "/".to_string(),
            1 => "--".to_string(),
            2 => "\\".to_string(),
            3 => " |".to_string(),
            _ => "/".to_string(),
        }
    }
    unsafe {
        netwk_err     = NETWORKING_ERROR;
        updates_avail = UPDATES_AVAILABLE;
        update_freeze = INSTALL_FREEZE;
    }
    selection = glob_get_update_selection();

    let pressed_key = rl.get_key_pressed();

    if let Some(pressed_key) = pressed_key {
        if pressed_key == KeyboardKey::KEY_DOWN {
            match selection {
                UpdateSelection::InstallUpdates => {
                    selection_new = Some(UpdateSelection::Back);
                },
                UpdateSelection::Back => {},
            }
        }
        if pressed_key == KeyboardKey::KEY_UP {
            match selection {
                UpdateSelection::InstallUpdates => {},
                UpdateSelection::Back => {
                    if updates_avail {
                        selection_new = Some(UpdateSelection::InstallUpdates);
                    }
                },
            }
        }
        if pressed_key == KeyboardKey::KEY_ESCAPE {
            switch_to_mode_configure();
            return;
        }
        if pressed_key == KeyboardKey::KEY_ENTER {
            match selection {
                UpdateSelection::InstallUpdates => {
                    launch_updated_installer();
                    return;
                },
                UpdateSelection::Back => {
                    switch_to_mode_configure();
                    return;
                },
            }
        }
    }

    match selection_new {
        Some(selection_new) => {
            unsafe {
                selection = selection_new;
                UPDATE_SELECTION = selection;
            }
        },
        None => {},
    }

    let mut d = rl.begin_drawing(&thread);

    d.clear_background(Color::BLACK);
    d.draw_text("RETRO VENEER", 12, 12, 40, Color::GRAY);
    d.draw_text(&format!("v{}", VERSION), 580, 12, 20, Color::GRAY);
    d.draw_line(6,58,632,58, Color::GRAY);

    d.draw_text("UPDATE EVERYTHING", 12, 70, 40, Color::BLUE);


    y_base = 116;
    if netwk_err {
        text_option = "Network Error: Unable to check for Updates.".to_string();
        d.draw_text(&text_option,  15+s_x, y_base+s_y, 20, Color::RED);
    } else if update_freeze {
        text_option = "Updates Currently Frozen: Try again later.".to_string();
        d.draw_text(&text_option,  15+s_x, y_base+s_y, 20, Color::RED);

        text_option = "* Updates are likely being made to the installers *".to_string();
        d.draw_text(&text_option,  15+s_x, y_base+s_y+30, 20, Color::GRAY);
    } else if ! updates_avail {
        text_option = "Your RetroVeneer setup is up-to-date.".to_string();
        d.draw_text(&text_option,  15+s_x, y_base+s_y, 20, Color::WHITE);
    } else {
        text_option = "An updated RetroVeneer setup is available.".to_string();
        d.draw_text(&text_option,  15+s_x, y_base+s_y, 20, Color::GREEN);
    }

    y_base += 100;
    if updates_avail {
        text_option = "INSTALL UPDATES".to_string();
        if selection == UpdateSelection::InstallUpdates {
            spinner_y = y_base+s_y;
            d.draw_text(&text_option,  62+s_x, y_base+s_y+2, 30, Color::RED);
            d.draw_text(&text_option,  60+s_x, y_base+s_y, 30, Color::YELLOW);
        } else {
            d.draw_text(&text_option,  60+s_x, y_base+s_y, 30, Color::WHITE);
        }
    }

    y_base += 80;
    text_option = "BACK".to_string();
    if selection == UpdateSelection::Back {
        spinner_y = y_base+s_y;
        d.draw_text(&text_option,  62+s_x, y_base+s_y+2, 30, Color::RED);
        d.draw_text(&text_option,  60+s_x, y_base+s_y, 30, Color::YELLOW);
    } else {
        d.draw_text(&text_option,  60+s_x, y_base+s_y, 30, Color::GRAY);
    }


    d.draw_line(6,442,632,442, Color::GRAY);
    d.draw_text("by: ModernRetroDev", 420, 450, 20, Color::GRAY);

    d.draw_text(&spinnertxt, spinner_x+2+s_x, spinner_y+2, 30, Color::RED);
    d.draw_text(&spinnertxt, spinner_x+s_x, spinner_y, 30, Color::YELLOW);
}


fn launch_platform_emulator() {
    let homedir  = format!("{}", dirs::home_dir().unwrap().display());
    let temp_path = format!("{homedir}/retroveneer/.temp");
    let emu_dir:  String;
    let emu_name: String;
    let fs_path:  String;

    fs::create_dir_all(&temp_path).unwrap();

    let launcher = format!("{temp_path}/launcher.sh");

    let mut file_launcher = File::create(&launcher).unwrap();

    writeln!(&mut file_launcher, "#!/usr/bin/env bash").unwrap();
    writeln!(&mut file_launcher, "").unwrap();

    match glob_get_platform_selection() {
        PlatformSelection::CommanderX16 => {
            emu_name = "x16emu".to_string();
            emu_dir = format!("{homedir}/retroveneer/emulators/x16emu");
            fs_path = format!("{homedir}/retroveneer/data/x16emu");

            writeln!(&mut file_launcher, "cd {emu_dir}").unwrap();
            writeln!(&mut file_launcher, "while :; do").unwrap();
            writeln!(&mut file_launcher, 
                "    ./{emu_name} -fsroot {fs_path} -fullscreen").unwrap();
        },
        PlatformSelection::Tic80 => {
            emu_name = "tic80".to_string();
            emu_dir = format!("{homedir}/retroveneer/emulators/tic80");
            fs_path = format!("{homedir}/retroveneer/data/tic80");

            writeln!(&mut file_launcher, "cd {emu_dir}").unwrap();
            writeln!(&mut file_launcher, "while :; do").unwrap();
            writeln!(&mut file_launcher, 
                "    ./{emu_name} --fullscreen --fs={fs_path}").unwrap();
        },
        PlatformSelection::Invalid => {},
    }

    writeln!(&mut file_launcher, "done").unwrap();

    file_launcher.flush().unwrap();

    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("chmod +x {launcher}"))
        .output();

    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("{launcher} &"))
        .spawn();
}

fn launch_updated_installer() {
    let homedir = format!("{}", dirs::home_dir().unwrap().display());
    let temp_path = format!("{homedir}/retroveneer/.temp");
    let rvbs_path = format!("{temp_path}/rvbs.sh");
    let url_rvbs = format!("{URL_RETROVENEER}/rvbs.sh");

    fs::create_dir_all(&temp_path).unwrap();

    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("wget {url_rvbs} -P {temp_path}"))
        .output();

    //------------------------------------------------------------------------//
    // Set the executable flag for the rvbs script.                           //
    //------------------------------------------------------------------------//
    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("chmod +x {}", rvbs_path))
        .output();

    //------------------------------------------------------------------------//
    // Set the executable flag for the rvbs script.                           //
    //------------------------------------------------------------------------//
    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("{rvbs_path} --update &"))
        .spawn();

    unsafe {
        EXIT_NOW = true;
    }
}


#[tokio::main]
async fn main() {
    let (mut rl, thread) = raylib::init()
        .size(640, 480)
        .title("Retro Veneer")
        .build();

    tokio::spawn(update_spinner());

    unsafe {
        AUTOSTART_ENABLED = autostart_is_enabled();
        PLATFORM_SELECTION = config_get_platform();
    }

    if glob_get_platform_selection() == PlatformSelection::Invalid {
        unsafe {
            RUNMODE = RunMode::SelectPlatform;
            PLATFORM_SELECTION = PlatformSelection::CommanderX16;
            // ^^^ Should be first (TOP) in the selection list
        }
    }

    touch_rv_running_file();

    rl.set_exit_key(None);
    // ^^^ Disable KEY_ESCAPE to close window, X-button still works

    rl.set_target_fps(60);
    // rl.toggle_fullscreen();
    // while !rl.window_should_close() {
    loop {
        let rmode: RunMode;
        unsafe {
            rmode = RUNMODE;
        }
        match rmode {
            RunMode::EmulatorRunning => {
                mode_emulator_running(&mut rl, &thread);
            },
            RunMode::Countdown => {
                mode_countdown(&mut rl, &thread);
            },
            RunMode::Configure => {
                mode_configure(&mut rl, &thread);
            },
            RunMode::SelectPlatform => {
                mode_select_platform(&mut rl, &thread);
            },
            RunMode::About => {
                mode_about(&mut rl, &thread);
            },
            RunMode::AboutRetroVeneer => {
                mode_about_retroveneer(&mut rl, &thread);
            },
            RunMode::AboutLicense => {
                mode_about_license(&mut rl, &thread);
            },
            RunMode::AboutTheEmulators => {
                mode_about_the_emulators(&mut rl, &thread);
            },
            RunMode::AboutDirectories => {
                mode_about_directories(&mut rl, &thread);
            },
            RunMode::UpdateEverything => {
                mode_update_everything(&mut rl, &thread);
            },
        }

        unsafe {
            if EXIT_NOW {
                break;
            }
        }
    }

    remove_rv_running_file();
}