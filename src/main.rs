use std::io::stdout;

use clap::{App, AppSettings, Arg, Shell, SubCommand};

mod subcommands;

pub mod traits;
use traits::WordPiecesApp;

static DEFAULT_CLAP_SETTINGS: &[AppSettings] = &[
    AppSettings::DontCollapseArgsInUsage,
    AppSettings::UnifiedHelpMessage,
    AppSettings::SubcommandRequiredElseHelp,
];

fn main() {
    // Known subapplications.
    let apps = vec![subcommands::StatsApp::app()];

    let cli = App::new("wordpieces")
        .settings(DEFAULT_CLAP_SETTINGS)
        .subcommands(apps)
        .subcommand(
            SubCommand::with_name("completions")
                .about("Generate completion scripts for your shell")
                .setting(AppSettings::ArgRequiredElseHelp)
                .arg(Arg::with_name("shell").possible_values(&Shell::variants())),
        );
    let matches = cli.clone().get_matches();

    match matches.subcommand_name().unwrap() {
        "completions" => {
            let shell = matches
                .subcommand_matches("completions")
                .unwrap()
                .value_of("shell")
                .unwrap();
            write_completion_script(cli, shell.parse::<Shell>().unwrap());
        }
        "stats" => subcommands::StatsApp::parse(matches.subcommand_matches("stats").unwrap()).run(),
        _unknown => unreachable!(),
    }
}

fn write_completion_script(mut cli: App, shell: Shell) {
    cli.gen_completions_to("finalfusion", shell, &mut stdout());
}
