extern crate clap;
#[macro_use]
extern crate duct;
#[macro_use]
extern crate failure;

type Result<T> = ::std::result::Result<T, failure::Error>;

fn main() {
    let code = {
        let matches = cli().get_matches();
        let window_name = matches.value_of("name").unwrap();
        let mut prog = matches.values_of("prog").unwrap();
        match jump_app(window_name, prog) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("{}", e);
                101
            }
        }
    };
    ::std::process::exit(code);
}

fn jump_app(window_name: &str, mut prog: clap::Values) -> Result<()> {
    let window_name = window_name.to_lowercase();
    let ws = list_windows()?;
    let matching = ws.into_iter()
        .find(|w| w.matches(&window_name));
    match matching {
        None => {
            let executable = prog.next().unwrap();
            let handle = duct::cmd(executable, prog)
                .stderr_null()
                .stdout_null()
                .start()?;
            drop(handle);
        }
        Some(ref w) => raise_or_hide(w)?
    };
    Ok(())
}

fn raise_or_hide(w: &Window) -> Result<()> {
    let focused = focused_window()?;
    let cmd = if focused == w.id {
        cmd!("xdotool", "getactivewindow", "windowminimize")
    } else {
        cmd!("wmctrl", "-i", "-a", format!("0x{:x}", w.id))
    };
    cmd.run()?;
    Ok(())
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

fn list_windows() -> Result<Vec<Window>> {
    let windows = cmd!("wmctrl", "-lx").read()?;
    windows.lines()
        .filter(|win| win.split_whitespace().nth(1) == Some("0"))
        .map(|win| {
            let id = win.split_whitespace().next()
                .ok_or(format_err!("unable to parse {:?}", win))?
                .to_string();
            let id = parse_window_id(&id)?;
            let name = win.split_whitespace().nth(2)
                .ok_or(format_err!("unable to parse {:?}", win))?
                .to_string();
            Ok(Window { id, name })
        })
        .collect()
}

fn focused_window() -> Result<u64> {
    let id = cmd!("xprop", "-root", "_NET_ACTIVE_WINDOW")
        .read()?
        .split_whitespace()
        .last()
        .ok_or(format_err!("Unable to get focused window"))?
        .to_string();
    parse_window_id(&id)
}

fn parse_window_id(id: &str) -> Result<u64> {
    let id = u64::from_str_radix(&id[2..], 16)?;
    Ok(id)
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
