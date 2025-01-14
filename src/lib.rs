// Simple library that allows for simple method of asking for screenshots from various Linux/BSD desktops

use std::env;
use std::fs;
use std::process::Command;

static SELECTION_TEMPORARY_FILE: &str = "/tmp/selection-tmp.png";

pub enum ScreenshotKind {
    Area,
    Window,
    Full,
}

enum SessionKind {
    Wayland,
    X11,
    Macos,
}

enum DesktopKind {
    GNOME,
    KDE,
    Sway,
    Generic,
    Macos,
}

fn session_type() -> SessionKind {
    return match env::var("XDG_SESSION_TYPE") {
        Ok(ok) => match ok.to_lowercase().as_ref() {
            "wayland" => SessionKind::Wayland,
            _ => SessionKind::X11,
        },
        Err(_) => {
            if cfg!(target_os = "macos") {
                SessionKind::Macos
            } else {
                SessionKind::X11
            }
        }
    };
}

fn screenshot_tool_selection(session: SessionKind) -> DesktopKind {
    return match session {
        SessionKind::Wayland => match Command::new("grim").arg("--version").spawn() {
            Ok(_) => DesktopKind::Sway,
            Err(_) => match Command::new("spectacle").arg("--version").spawn() {
                Ok(_) => DesktopKind::KDE,
                Err(_) => match Command::new("gnome-screenshot").arg("--version").spawn() {
                    Ok(_) => DesktopKind::GNOME,
                    Err(_) => panic!("Uncompatible Wayland desktop"),
                },
            },
        },
        SessionKind::X11 => match Command::new("spectacle").arg("--version").spawn() {
            Ok(_) => DesktopKind::KDE,
            Err(_) => match Command::new("gnome-screenshot").arg("--version").spawn() {
                Ok(_) => DesktopKind::GNOME,
                Err(_) => match Command::new("scrot").arg("--version").spawn() {
                    Ok(_) => DesktopKind::Generic,
                    Err(_) => panic!("Uncompatible X11 desktop (install scrot)"),
                },
            },
        },
        SessionKind::Macos => DesktopKind::Macos,
    };
}

pub fn screenshot_area(file: String, freeze: bool) {
    match screenshot_tool_selection(session_type()) {
        DesktopKind::GNOME => gnome(ScreenshotKind::Area, file, freeze),
        DesktopKind::KDE => kde(ScreenshotKind::Area, file),
        DesktopKind::Sway => sway(ScreenshotKind::Area, file),
        DesktopKind::Generic => scrot(ScreenshotKind::Area, file, freeze),
        DesktopKind::Macos => mac(ScreenshotKind::Area, file),
    }
}
pub fn screenshot_window(file: String) {
    match screenshot_tool_selection(session_type()) {
        DesktopKind::GNOME => gnome(ScreenshotKind::Window, file, false),
        DesktopKind::KDE => kde(ScreenshotKind::Window, file),
        DesktopKind::Sway => sway(ScreenshotKind::Window, file),
        DesktopKind::Generic => scrot(ScreenshotKind::Window, file, false),
        DesktopKind::Macos => mac(ScreenshotKind::Window, file),
    }
}
pub fn screenshot_full(file: String) {
    match screenshot_tool_selection(session_type()) {
        DesktopKind::GNOME => gnome(ScreenshotKind::Full, file, false),
        DesktopKind::KDE => kde(ScreenshotKind::Full, file),
        DesktopKind::Sway => sway(ScreenshotKind::Full, file),
        DesktopKind::Generic => scrot(ScreenshotKind::Full, file, false),
        DesktopKind::Macos => mac(ScreenshotKind::Full, file),
    }
}
fn gnome(option: ScreenshotKind, file: String, freeze: bool) {
    match option {
        ScreenshotKind::Area => {
            let mut feh = match Command::new("feh").arg("--version").spawn() {
                Ok(_) => {
                    Command::new("gnome-screenshot")
                        .args(&["-f", SELECTION_TEMPORARY_FILE])
                        .output()
                        .expect("gnome-screenshot did not launch");
                    Command::new("feh")
                        .args(&[SELECTION_TEMPORARY_FILE, "-F"])
                        .spawn()
                        .expect("'feh' did not launch to pause screen for selection")
                }
                Err(_) => Command::new("sh")
                    .arg("-c")
                    .arg("echo Feh does not exist")
                    .spawn()
                    .unwrap(),
            };
            Command::new("gnome-screenshot")
                .args(&["-a", "-f", &file])
                .output()
                .expect("gnome-screenshot did not launch");
            if freeze {
                match fs::remove_file(SELECTION_TEMPORARY_FILE) {
                    Ok(ok) => ok,
                    Err(_) => eprintln!("Unable to remove temporary selection file"),
                };
                match feh.kill() {
                    Ok(ok) => ok,
                    Err(_) => eprintln!("Unable to kill feh, must have already been closed"),
                };
            }
        }
        ScreenshotKind::Window => {
            Command::new("gnome-screenshot")
                .args(&["-w", "-e", "shadow", "-f", &file])
                .output()
                .expect("gnome-screenshot did not launch");
        }
        ScreenshotKind::Full => {
            Command::new("gnome-screenshot")
                .args(&["-f", &file])
                .output()
                .expect("gnome-screenshot did not launch");
        }
    };
}
fn kde(option: ScreenshotKind, file: String) {
    match option {
        ScreenshotKind::Area => {
            Command::new("spectacle")
                .args(&["-rbno", &file])
                .output()
                .expect("spectacle did not launch");
        }
        ScreenshotKind::Window => {
            Command::new("spectacle")
                .args(&["-abno", &file])
                .output()
                .expect("spectacle did not launch");
        }
        ScreenshotKind::Full => {
            Command::new("spectacle")
                .args(&["-fbno", &file])
                .output()
                .expect("spectacle did not launch");
        }
    };
}
fn sway(option: ScreenshotKind, file: String) {
    match option {
        ScreenshotKind::Area => {
            let slurp = Command::new("slurp")
                .output()
                .expect("slurp did not launch");
            Command::new("grim")
                .args(&["-g", &String::from_utf8(slurp.stdout).unwrap(), &file])
                .output()
                .expect("grim did not launch");
        }
        ScreenshotKind::Window => {
            let slurp = Command::new("slurp")
                .output()
                .expect("slurp did not launch");
            Command::new("grim")
                .args(&["-g", &String::from_utf8(slurp.stdout).unwrap(), &file])
                .output()
                .expect("grim did not launch");
        }
        ScreenshotKind::Full => {
            Command::new("grim")
                .arg(&file)
                .output()
                .expect("grim did not launch");
        }
    };
}
fn mac(option: ScreenshotKind, file: String) {
    match option {
        ScreenshotKind::Area => {
            Command::new("screencapture")
                .args(&["-s", &file])
                .output()
                .expect("screencapture did not launch");
        }
        ScreenshotKind::Window => {
            Command::new("screencapture")
                .args(&["-w", &file])
                .output()
                .expect("screencapture did not launch");
        }
        ScreenshotKind::Full => {
            Command::new("screencapture")
                .args(&["-S", &file])
                .output()
                .expect("screencapture did not launch");
        }
    };
}
fn scrot(option: ScreenshotKind, file: String, freeze: bool) {
    match option {
        ScreenshotKind::Area => {
            let mut feh = match Command::new("feh").arg("--version").spawn() {
                Ok(_) => {
                    Command::new("scrot")
                        .arg(SELECTION_TEMPORARY_FILE)
                        .output()
                        .expect("scrot did not launch");
                    Command::new("feh")
                        .args(&[SELECTION_TEMPORARY_FILE, "-F"])
                        .spawn()
                        .expect("'feh' did not launch to pause screen for selection")
                }
                Err(_) => Command::new("sh")
                    .arg("-c")
                    .arg("echo Feh does not exist")
                    .spawn()
                    .unwrap(),
            };
            Command::new("scrot")
                .args(&["--select", &file])
                .output()
                .expect("scrot did not launch");
            if freeze {
                match fs::remove_file(SELECTION_TEMPORARY_FILE) {
                    Ok(ok) => ok,
                    Err(_) => eprintln!("Unable to remove temporary selection file"),
                };
                match feh.kill() {
                    Ok(ok) => ok,
                    Err(_) => eprintln!("Unable to kill feh, must have already been closed"),
                };
            }
        }
        ScreenshotKind::Window => {
            Command::new("scrot")
                .args(&["--border", "--focused", &file])
                .output()
                .expect("gnome-screenshot did not launch");
        }
        ScreenshotKind::Full => {
            Command::new("scrot")
                .args(&[&file])
                .output()
                .expect("gnome-screenshot did not launch");
        }
    };
}
