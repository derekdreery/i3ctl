use i3ipc::I3Connection;
use std::error::Error;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(subcommand)]
    cmd: Subcommand,
}

#[derive(Debug, StructOpt)]
enum Subcommand {
    /// Open a terminal at the current directory.
    #[structopt(name = "term")]
    Term,
}

fn main() {
    let opt = Opt::from_args();
    if let Err(e) = match opt.cmd {
        Subcommand::Term => term(),
    } {
        eprintln!("Error: {}", e);
    }
}

fn term() -> Result<(), Box<dyn Error>> {
    let cwd = std::env::current_dir()?;
    let mut conn = I3Connection::connect()?;
    // probably won't work with wierd chars in the dir name (`"` or `\`)
    conn.run_command(&format!(
        "exec alacritty --working-directory \"{}\"",
        cwd.display()
    ))?;
    Ok(())
}
