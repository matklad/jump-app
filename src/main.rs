extern crate clap;
#[macro_use]
extern crate duct;

fn main() {
    let matches = cli().get_matches();
    let window_name = matches.value_of("name").unwrap().to_lowercase();
    let mut prog = matches.values_of("prog").unwrap();

    let ws = list_windows();
    let matching = ws.into_iter()
        .find(|w| w.matches(&window_name));
    match matching {
        None => {
            let executable = prog.next().unwrap();
            let handle = duct::cmd(executable, prog)
                .stderr_null()
                .stdout_null()
                .start()
                .unwrap();
            drop(handle);
        }
        Some(ref w) => raise_or_hide(w)
    }
}

fn raise_or_hide(w: &Window) {
    let focused = focused_window();
    let cmd = if focused == w.id {
        cmd!("xdotool", "getactivewindow", "windowminimize")
    } else {
        cmd!("wmctrl", "-i", "-a", format!("0x{:x}", w.id))
    };
    let out = cmd.run().unwrap();
}


#[derive(Debug)]
struct Window {
    id: u64,
    name: String,
}

impl Window {
    fn matches(&self, name: &str) -> bool {
        self.name.to_lowercase().contains(name)
    }
}

fn list_windows() -> Vec<Window> {
    let windows = cmd!("wmctrl", "-lx").read().unwrap();
    windows.lines()
        .filter(|win| win.split_whitespace().nth(1) == Some("0"))
        .map(|win| {
            let id = win.split_whitespace().next().unwrap().to_string();
            let id = parse_window_id(&id);
            let name = win.split_whitespace().nth(2).unwrap().to_string();
            Window { id, name }
        })
        .collect()
}

fn focused_window() -> u64 {
    let id = cmd!("xprop", "-root", "_NET_ACTIVE_WINDOW")
        .read()
        .unwrap()
        .split_whitespace()
        .last()
        .unwrap()
        .to_string();
    parse_window_id(&id)
}

fn parse_window_id(id: &str) -> u64 {
    u64::from_str_radix(&id[2..], 16)
        .unwrap()
}


fn cli() -> clap::App<'static, 'static> {
    clap::App::new("jump-app")
        .arg(
            clap::Arg::with_name("name")
                .required(true)
        )
        .arg(
            clap::Arg::with_name("prog")
                .last(true)
                .multiple(true)
        )
}
