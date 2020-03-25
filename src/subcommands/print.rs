use std::convert::TryFrom;
use std::fs::File;
use std::io::{BufRead, BufReader};

use clap::{App, AppSettings, Arg, ArgMatches};
use conllu::graph::Node;
use conllu::io::{ReadSentence, Reader};
use stdinout::OrExit;
use wordpieces::{WordPiece, WordPieces};

use crate::traits::WordPiecesApp;

static DEFAULT_CLAP_SETTINGS: &[AppSettings] = &[
    AppSettings::DontCollapseArgsInUsage,
    AppSettings::UnifiedHelpMessage,
];

pub struct PrintApp {
    wordpieces: String,
    corpus: String,
}

impl WordPiecesApp for PrintApp {
    fn app() -> App<'static, 'static> {
        App::new("print")
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
        PrintApp {
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

        for sentence in reader.sentences() {
            let sentence = sentence.or_exit("Cannot read sentence", 1);

            let mut pieces = Vec::new();
            for token in sentence.iter().filter_map(Node::token) {
                match word_pieces
                    .split(token.form())
                    .add_continuation_markers("##")
                    .collect::<Option<Vec<_>>>()
                {
                    Some(word_pieces) => pieces.extend(word_pieces),
                    None => pieces.push("[UNK]".to_owned()),
                }
            }

            println!("{}", pieces.join(" "));
        }
    }
}

// Add continuation markers for every piece except the first.
trait AddContinuationMarkers {
    type Iter;

    /// Return an iterator that adds continuation markers.
    fn add_continuation_markers(self, marker: impl Into<String>) -> Self::Iter;
}

impl<'a, I> AddContinuationMarkers for I
where
    I: Iterator<Item = WordPiece<'a>>,
{
    type Iter = ContinuationMarkerIter<I>;

    fn add_continuation_markers(self, marker: impl Into<String>) -> Self::Iter {
        ContinuationMarkerIter {
            initial: true,
            inner: self,
            marker: marker.into(),
        }
    }
}

struct ContinuationMarkerIter<I> {
    initial: bool,
    inner: I,
    marker: String,
}

impl<'a, I> Iterator for ContinuationMarkerIter<I>
where
    I: Iterator<Item = WordPiece<'a>>,
{
    type Item = Option<String>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.next()?;

        let item = if self.initial {
            self.initial = false;
            item.piece().map(ToOwned::to_owned)
        } else {
            item.piece()
                .map(|piece| format!("{}{}", self.marker, piece))
        };

        Some(item)
    }
}
