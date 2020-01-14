use std::convert::TryFrom;
use std::fs::File;
use std::io::{BufRead, BufReader};

use clap::{App, AppSettings, Arg, ArgMatches};
use conllx::graph::Node;
use conllx::io::{ReadSentence, Reader};
use stdinout::OrExit;
use wordpieces::{WordPiece, WordPieces};

use crate::traits::WordPiecesApp;

static DEFAULT_CLAP_SETTINGS: &[AppSettings] = &[
    AppSettings::DontCollapseArgsInUsage,
    AppSettings::UnifiedHelpMessage,
];

pub struct StatsApp {
    wordpieces: String,
    corpus: String,
}

impl WordPiecesApp for StatsApp {
    fn app() -> App<'static, 'static> {
        App::new("stats")
            .settings(DEFAULT_CLAP_SETTINGS)
            .arg(
                Arg::with_name("WORDPIECES")
                    .help("Word pieces file")
                    .index(1)
                    .required(true),
            )
            .arg(
                Arg::with_name("CORPUS")
                    .help("Corpus in CoNLL-X format")
                    .index(2)
                    .required(true),
            )
    }

    fn parse(matches: &ArgMatches) -> Self {
        StatsApp {
            wordpieces: matches.value_of("WORDPIECES").unwrap().to_owned(),
            corpus: matches.value_of("CORPUS").unwrap().to_owned(),
        }
    }

    fn run(&self) {
        let wordpieces_file = File::open(&self.wordpieces).or_exit(
            format!("Cannot open word pieces file: {}", self.wordpieces),
            1,
        );
        let word_pieces = WordPieces::try_from(BufReader::new(wordpieces_file).lines()).or_exit(
            format!("Cannot read word pieces from: {}", self.wordpieces),
            1,
        );

        let conll_file =
            File::open(&self.corpus).or_exit(format!("Cannot open corpus: {}", self.corpus), 1);
        let reader = Reader::new(BufReader::new(conll_file));

        let mut counts = Vec::new();
        let mut unknowns = 0;
        let mut suffix_unknowns = 0;
        let mut n_tokens = 0;

        for sentence in reader.sentences() {
            let sentence = sentence.or_exit("Cannot read sentence", 1);

            for token in sentence.iter().filter_map(Node::token) {
                let pieces: Vec<_> = word_pieces.split(token.form()).collect();
                if pieces[0] == WordPiece::Missing {
                    unknowns += 1;
                } else if *pieces.last().unwrap() == WordPiece::Missing {
                    suffix_unknowns += 1;
                } else {
                    counts.push(pieces.len());
                }
                n_tokens += 1;
            }
        }

        counts.sort();

        eprintln!(
            "Unknown: {:.2}%",
            (unknowns as f64 / n_tokens as f64) * 100.
        );
        eprintln!(
            "Unknown suffix: {:.2}%",
            (suffix_unknowns as f64 / n_tokens as f64) * 100.
        );
        eprintln!(
            "Average length: {:.2}",
            counts.iter().sum::<usize>() as f64 / counts.len() as f64
        );
        eprintln!("Median length: {}", counts[counts.len() / 2]);
    }
}
