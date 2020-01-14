use clap::{App, ArgMatches};

pub trait WordPiecesApp {
    fn app() -> App<'static, 'static>;

    fn parse(matches: &ArgMatches) -> Self;

    fn run(&self);
}
