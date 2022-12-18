mod choregraphy;
mod choregraphy_inspect;
mod choregraphy_ls;
mod choregraphy_process;
mod fs;
mod inject;
mod process;
use anyhow::{Context, Result};
use choregraphy::{choregraphy_options, ChoregraphyOptions};
use clap::{App, IntoApp, Parser};
use clap_complete::{
  generate,
  shells::{Bash, Elvish, Fish, PowerShell, Zsh},
  Generator,
};
use fs::{fs, FsOptions};
use inject::inject;
use node_rs::settings::Settings;
use process::{process_options, ProcessOptions};
use std::io;

/// Dancer -- an command interface for the choregrapher
#[derive(Parser, Debug)]
#[clap(
  version = "1.0.0",
  author = "Valvassori M. <moise.valvassori@gmail.com>"
)]
pub struct Opts {
  // /// Sets a custom config file. Could have been an Option<T> with no default too
  // #[clap(short, long, default_value = "default.conf")]
  // config: String,
  /// A level of verbosity, and can be used multiple times
  #[clap(short, long, parse(from_occurrences))]
  pub verbose: i32,
  /// Print debug info
  #[clap(short, long)]
  pub debug: bool,

  #[clap(long)]
  /// Mongo db connection url
  pub mongo_url: Option<String>,

  #[clap(long)]
  /// Mongo db database name
  pub mongo_database: Option<String>,

  #[clap(long)]
  /// Rabbitmq url
  pub amqp_url: Option<String>,

  #[clap(subcommand)]
  subcmd: SubCommand,
}

#[derive(Parser, Debug)]
struct CompletionsOptions {
  #[clap(subcommand)]
  choice: GeneratorChoice,
}

#[derive(Parser, Debug, PartialEq)]
enum GeneratorChoice {
  /// Generate bash completion
  ///
  ///  Completion files are commonly stored in `/etc/bash_completion.d/` for
  ///    system-wide commands, but can be stored in
  ///    `~/.local/share/bash-completion/completions` for user-specific commands.
  ///    Run the command:
  ///
  ///        $ mkdir -p ~/.local/share/bash-completion/completions
  ///        $ dancer completions bash > ~/.local/share/bash-completion/completions/dancer
  ///
  ///    This installs the completion script. You may have to log out and
  ///    log back in to your shell session for the changes to take effect.
  Bash,
  Elvish,
  Fish,
  #[clap(name = "powershell")]
  PowerShell,
  Zsh,
}

#[derive(Parser, Debug)]
enum SubCommand {
  #[clap(version = "0.1.0")]
  Inject(InjectOptions),
  #[clap(version, author)]
  Choregraphy(ChoregraphyOptions),
  #[clap(version, author)]
  Process(ProcessOptions),
  /// File system management.
  #[clap(version, author)]
  Fs(FsOptions),

  /// Generate shell completions.
  ///
  /// `dancer completions bash >  ~/.local/share/bash-completion/completions/dancer`
  #[clap(version = "1.0.0")]
  Completions(CompletionsOptions),
}

/// A subcommand for injecting process in the dance.
///
/// Example:
///
/// dancer inject fact '{"o":{"n":10,"fact":1}}'
#[derive(Parser, Debug)]
#[clap()]
pub struct InjectOptions {
  /// Graph
  graph: String,
  /// JSON data to inject
  data: String,
}

fn print_completions<G: Generator>(gen: G, app: &mut App) -> Result<()> {
  generate(gen, app, app.get_name().to_string(), &mut io::stdout());
  Ok(())
}

fn main() -> Result<()> {
  pretty_env_logger::init_timed();
  let opts: Opts = Opts::parse();
  //  println!("{:#?}", opts);
  // Gets a value for config if supplied by user, or defaults to "default.conf"
  //  println!("Value for config: {}", opts.config);
  //  println!("Using input file: {:?}", opts.input);

  // Vary the output based on how many times the user used the "verbose" flag
  // (i.e. 'myprog -v -v -v' or 'myprog -vvv' vs 'myprog -v'
  // match opts.verbose {
  //   0 => println!("No verbose info"),
  //   1 => println!("Some verbose info"),
  //   2 => println!("Tons of verbose info"),
  //   3 | _ => println!("Don't be crazy"),
  // }

  let mut settings = Settings::load().context("Can't load the default configuration")?;
  if let Some(mongo_url) = &opts.mongo_url {
    settings
      .set("database.url", mongo_url.clone())
      .context("Can't set the database url.")?;
  }
  if let Some(mongo_database) = &opts.mongo_database {
    settings
      .set("database.database", mongo_database.clone())
      .context("Can't set the database.")?;
  }
  if let Some(amqp_url) = &opts.amqp_url {
    settings
      .set("rabbitmq.url", amqp_url.clone())
      .context("Can't set the AMQP URL.")?;
  }
  let settings: Settings = settings.try_into().context("Can't load configuration")?;

  // You can handle information about subcommands by requesting their matches by name
  // (as below), requesting just the name used, or both at the same time
  match &opts.subcmd {
    SubCommand::Inject(injection) => inject(injection, &opts, &settings),
    SubCommand::Completions(completions) => {
      let mut app = Opts::into_app();
      eprintln!("Generating completion file for {:?}...", completions.choice);
      match completions.choice {
        GeneratorChoice::Bash => print_completions(Bash, &mut app),
        GeneratorChoice::Elvish => print_completions(Elvish, &mut app),
        GeneratorChoice::Fish => print_completions(Fish, &mut app),
        GeneratorChoice::PowerShell => print_completions(PowerShell, &mut app),
        GeneratorChoice::Zsh => print_completions(Zsh, &mut app),
      }
    }
    SubCommand::Choregraphy(options) => choregraphy_options(options, &opts, &settings),
    SubCommand::Process(options) => process_options(options, &opts, &settings),
    SubCommand::Fs(options) => fs(options, &opts, &settings),
  }
  // more program logic goes here...
}
