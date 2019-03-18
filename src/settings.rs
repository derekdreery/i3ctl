use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::{fmt, io, path::Path};
use structopt::StructOpt;

// CLI options
// -----------

/// Command line options.
#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(subcommand)]
    pub cmd: Subcommand,
    #[structopt(flatten)]
    pub verbosity: clap_verbosity_flag::Verbosity,
    ///// Save any overriden settings set via command lines or environment variables to the config
    ///// file.
    //#[structopt(long = "--save-settings")]
    //pub save_settings: bool,
}

#[derive(Debug, StructOpt)]
pub enum Subcommand {
    /// Open a terminal at the current directory.
    #[structopt(name = "term")]
    Term {
        /// The type of terminal (one of "alacritty", "urxvt", "gnome-terminal")
        terminal: Terminal,
    },
    /// Print out the config location
    #[structopt(name = "config-location")]
    PrintConfigLocation,
}

// Settings
// --------

#[derive(Debug, Clone, Serialize, Deserialize, StructOpt)]
pub enum Terminal {
    Alacritty,
    Urxvt,
    GnomeTerminal,
    XTerm,
    Custom {
        /// The template for starting the terminal at the given location.
        template: String,
    },
}

impl Default for Terminal {
    fn default() -> Self {
        Terminal::Alacritty
    }
}

impl std::str::FromStr for Terminal {
    type Err = &'static str;
    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        use Terminal::*;
        // spot types we recognise
        match s {
            "alacritty" => return Ok(Alacritty),
            "urxvt" => return Ok(Urxvt),
            "gnome-terminal" => return Ok(GnomeTerminal),
            "xterm" => return Ok(XTerm),
            _ => (), // fall thru
        }
        // parse a custom template
        if s.starts_with("custom(") && s.ends_with(")") {
            s = &s["custom(".len()..(s.len() - ")".len())];
        } else {
            return Err("custom command must be of form \"custom(.. your command here ..)\"");
        }
        Ok(Custom {
            template: s.to_owned(),
        })
    }
}

impl Terminal {
    pub fn display<'a>(&'a self, escaped_path: &'a str) -> TerminalTemplate<'a> {
        TerminalTemplate {
            terminal: self,
            path: escaped_path,
        }
    }
}

#[derive(Debug)]
pub struct TerminalTemplate<'a> {
    terminal: &'a Terminal,
    path: &'a str,
}

impl fmt::Display for TerminalTemplate<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Terminal::*;
        match self.terminal {
            Alacritty => write!(f, "alacritty --working-directory '{}'", self.path),
            Urxvt => write!(f, "urxvt --chdir '{}'", self.path),
            GnomeTerminal => write!(f, "gnome-terminal --working-directory='{}'", self.path),
            //XTerm => write!(f, r#"xterm -e "cd '{}'; bash""#, self.path),
            XTerm => {
                log::error!(
                    "I can't get the escaping to work for xterm so just opening a \
                     terminal. PRs welcome!"
                );
                write!(f, "xterm")
            }
            Custom { template: _ } => unimplemented!(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    /// Terminal to use for creating a new terminal.
    #[serde(default)]
    pub create_terminal: Terminal,
}

impl Settings {
    pub fn from_system(config_loc: &Path) -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.refresh()?; // workaround for config-rs#issue-60
        s.merge(File::from(config_loc.join("config")).required(false))?;
        s.merge(Environment::with_prefix("I3CTL"))?;
        s.try_into()
    }

    pub fn merge_opt(&mut self, opt: &Opt) {
        match &opt.cmd {
            Subcommand::Term { terminal } => {
                self.create_terminal.clone_from(terminal);
            }
            _ => (), // ignore other matches - we don't need to alter any settings for them.
        }
    }
}

// Errors
// ------

pub enum LoadSettingsError {
    Io(io::Error),
    Config(ConfigError),
}

impl From<io::Error> for LoadSettingsError {
    fn from(e: io::Error) -> Self {
        LoadSettingsError::Io(e)
    }
}

impl From<ConfigError> for LoadSettingsError {
    fn from(e: ConfigError) -> Self {
        LoadSettingsError::Config(e)
    }
}

impl fmt::Display for LoadSettingsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoadSettingsError::Io(e) => fmt::Display::fmt(e, f),
            LoadSettingsError::Config(e) => fmt::Display::fmt(e, f),
        }
    }
}

#[derive(Debug)]
pub enum TemplateError {
    Fmt,
}

impl From<fmt::Error> for TemplateError {
    fn from(_: fmt::Error) -> Self {
        TemplateError::Fmt
    }
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TemplateError::Fmt => fmt::Display::fmt(&fmt::Error, f),
        }
    }
}

impl std::error::Error for TemplateError {}
