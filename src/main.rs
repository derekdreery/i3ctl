mod settings;

use directories::ProjectDirs;
use i3ipc::I3Connection;
use settings::{Opt, Settings, Subcommand};
use std::error::Error;
use structopt::StructOpt;

#[cfg(windows)]
compiler_error!("Windows is not supported");

fn main() {
    let opt = Opt::from_args();
    pretty_env_logger::formatted_timed_builder()
        .filter(None, opt.verbosity.log_level().to_level_filter())
        .init();
    let project_dirs = match ProjectDirs::from("", "", "i3ctl") {
        Some(p) => p,
        None => {
            eprintln!("Error: could not determine home directory");
            std::process::exit(1);
        }
    };
    let mut settings = match Settings::from_system(project_dirs.config_dir()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };
    settings.merge_opt(&opt);
    if let Err(e) = match opt.cmd {
        Subcommand::Term { .. } => term(&settings),
        Subcommand::PrintConfigLocation => Ok(println!("{}", project_dirs.config_dir().display())),
    } {
        eprintln!("Error: {}", e);
    }
}

fn term(settings: &Settings) -> Result<(), Box<dyn Error>> {
    // Get the current directory
    let cwd = {
        use std::ffi::OsString;
        let cwd = OsString::from(std::env::current_dir()?);
        let cwd = match cwd.into_string() {
            Ok(s) => s,
            Err(_) => return Err("non-utf8 directory paths are not currently supported".into()),
        };
        if cwd.chars().any(|ch| !ch.is_ascii_graphic()) {
            eprintln!(
                "Warning: I'm not sure whether non-ascii-graphic is handled correctly \
                 by i3, check you're in the right directory"
            );
        }
        if cwd.chars().any(|ch| ch == '\'') {
            return Err("i3 does not support paths containing \"'\" (single quote)".into());
        }
        escape_control(&cwd)
    };
    let mut conn = I3Connection::connect()?;
    // probably won't work with wierd chars in the dir name (`"` or `\`)
    let cmd = format!("exec {}", settings.create_terminal.display(&cwd));
    log::debug!("running \"{}\"", cmd);
    conn.run_command(&cmd)?;
    Ok(())
}

/// Replace control characters with escape sequences.
///
/// Only escape tab, newline, quotes, and backslash.
fn escape_control(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\n' => output.push_str(r#"\n"#),
            '\t' => output.push_str(r#"\t"#),
            '"' => output.push_str(r#"\""#),
            '\'' => unreachable!(),
            '\\' => output.push_str(r#"\\"#),
            ch => output.push(ch),
        }
    }
    output
}
