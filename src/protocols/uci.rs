//! Universal Chess Interface implementation for Zorro.
//!
//! We aim for full compliance. Stockfish-like debugging features are also
//! present.

use crate::chess::{Board, Color, Coordinate, Move, Square};
use crate::core::Zorro;
use crate::err::Error as ChessErr;
use crate::eval::Eval;
use crate::search::*;
use crate::time::TimeControl;
use crate::version::{VERSION, VERSION_WITH_BUILD_INFO};
use std::fmt;
use std::io;
use std::str::FromStr;
use std::time::Duration;
use strum::IntoEnumIterator;

#[derive(Debug, PartialEq)]
pub enum State {
    Alive,
    Shutdown,
}

pub fn uci(
    zorro: &mut Zorro,
    input: impl io::BufRead,
    mut output: impl io::Write,
) -> io::Result<()> {
    // Greet the user with some information about the engine.
    writeln!(output, "# Zorro {}", VERSION_WITH_BUILD_INFO)?;
    writeln!(output, "# Copyright (c) 2019-2020, Filippo Costa")?;
    writeln!(output, "# Process ID: {}", std::process::id())?;
    for line in input.lines() {
        match handle_line(zorro, line?, &mut output) {
            Ok(State::Alive) => (),
            Ok(State::Shutdown) => break,
            // Don't exit on error.
            Err(err) => writeln!(output, "{}", err)?,
        }
    }
    Ok(())
}

pub fn handle_line(
    zorro: &mut Zorro,
    line: impl AsRef<str>,
    mut output: impl io::Write,
) -> Result<State> {
    let mut tokens = line.as_ref().split_whitespace();
    match tokens.next().unwrap_or("") {
        "" => (),
        "cleart" => clean_terminal_screen(output)?,
        "d" => cmd::debug_engine_state(&zorro, tokens, output)?,
        "debug" => cmd::set_debug_mode(zorro, tokens)?,
        "eval" => cmd::eval(&zorro.board, output)?,
        "gentables" => cmd::gen_tables(output)?,
        "go" => cmd::go(zorro, tokens, output)?,
        "isready" => writeln!(output, "readyok")?,
        "listmagics" => cmd::list_magics(output)?,
        "magic" => cmd::magic(tokens, output)?,
        "perft" => cmd::perft(zorro, tokens, output)?,
        "position" => cmd::position(zorro, tokens)?,
        "quit" | "stop" => return Ok(State::Shutdown),
        "setoption" => cmd::set_option(zorro, tokens)?,
        "uci" => cmd::uci(output)?,
        "ucispec" => cmd::open_uci_docs(output)?,
        "ucinewgame" => zorro.cache.clear(),
        s => return Err(Error::UnknownCommand(s.to_string())),
    }
    Ok(State::Alive)
}

fn clean_terminal_screen(mut output: impl io::Write) -> io::Result<()> {
    writeln!(output, "{}[2J", 27 as char)
}

/// UCI commands handlers.
mod cmd {
    use super::*;

    pub fn debug_engine_state<'s>(
        zorro: &Zorro,
        mut tokens: impl Iterator<Item = &'s str>,
        mut output: impl io::Write,
    ) -> Result<()> {
        match tokens.next() {
            None => write!(output, "{}", zorro.board)?,
            Some("fen") => writeln!(output, "{}", zorro.board.fmt_fen(' '))?,
            Some("time") => {
                for color in Color::iter() {
                    writeln!(output)?;
                    writeln!(
                        output,
                        "{}",
                        match color {
                            Color::W => "White:",
                            Color::B => "Black:",
                        }
                    )?;
                    writeln!(
                        output,
                        "Initial:   {}s",
                        zorro.time_controls[color].time_limit.as_secs()
                    )?;
                    writeln!(
                        output,
                        "Remaining: {}s",
                        zorro.time_controls[color].time_limit.as_secs()
                    )?;
                    writeln!(
                        output,
                        "Increment: {}s",
                        zorro.time_controls[color].time_limit.as_secs()
                    )?;
                    writeln!(
                        output,
                        "Delay:     {}s",
                        zorro.time_controls[color].time_limit.as_secs()
                    )?;
                }
            }
            Some("lichess") => {
                let url = zorro.board.lichess_url();
                writeln!(output, "{}", url)?;
                webbrowser::open(url.as_str()).map_err(|_| Error::Other)?;
            }
            _ => return Err(Error::Syntax),
        }
        Ok(())
    }

    pub fn uci(mut output: impl io::Write) -> Result<()> {
        // See http://www.talkchess.com/forum3/viewtopic.php?start=0&t=4230?;
        writeln!(
            output,
            "id name Zorro {}\n\
             id author Filippo Costa\n\
             option name Clear Hash type button\n\
             option name Contempt type spin default 20 min -100 max 100\n\
             option name Hash type spin default 64 min 0 max 131072\n\
             option name Minimum Thinking Time type spin default 20 min 0 max 5000\n\
             option name nodestime type spin default 0 min 0 max 10000\n\
             option name Skill Level type spin default 20 min 0 max 20\n\
             option name Slow Mover type spin default 84 min 10 max 1000\n\
             option name Threads type spin default 1 min 1 max 512\n\
             option name Move Overhead type spin default 30 min 0 max 60000\n\
             uciok",
            VERSION,
        )?;
        Ok(())
    }

    pub fn eval(board: &Board, mut output: impl io::Write) -> Result<()> {
        let eval = Eval::new(board);
        writeln!(
            output,
            "Total evaluation: {} ({} side)",
            eval.score,
            if eval.score >= 0 { "white" } else { "black" }
        )?;
        Ok(())
    }

    pub fn gen_tables<W: io::Write>(mut output: W) -> Result<()> {
        use crate::chess::tables;
        writeln!(output, "KING ATTACKS")?;
        for bb in (*tables::boxed_king_attacks()).iter() {
            writeln!(output, "0x{:016x}", bb)?;
        }
        writeln!(output, "KNIGHT ATTACKS")?;
        for bb in (*tables::boxed_knight_attacks()).iter() {
            writeln!(output, "0x{:016x}", bb)?;
        }
        Ok(())
    }

    pub fn go<'s>(
        zorro: &mut Zorro,
        mut tokens: impl Iterator<Item = &'s str>,
        mut output: impl io::Write,
    ) -> Result<()> {
        // We must preserve a freezed copy of current configuration options in
        // case the user changes things while searching.
        let mut config = zorro.config.clone();
        while let Some(token) = tokens.next() {
            let mut next = || tokens.next().ok_or(Error::Syntax);
            match token {
                "searchmoves" => {
                    // A for loop will cause ownership issues. FIXME?
                    while let Some(s) = tokens.next() {
                        config.restrict_search.push(Move::from_str(s)?);
                    }
                }
                "wtime" | "btime" | "winc" | "binc" => {
                    let color = Color::from_str(token)?;
                    let time_control = &mut zorro.time_controls[color];
                    let dur = Duration::from_millis(str::parse(next()?)?);
                    if &token[1..] == "time" {
                        time_control.time_limit = dur;
                    } else {
                        time_control.increment = dur;
                    }
                }
                "movestogo" => config.moves_to_go = Some(str::parse(next()?)?),
                "depth" => config.max_depth = Some(str::parse(next()?)?),
                "nodes" => config.max_nodes = Some(str::parse(next()?)?),
                "mate" => (),
                "movetime" => {
                    let color = zorro.board.color_to_move;
                    let time_control = &mut zorro.time_controls[color];
                    let dur = Duration::from_millis(str::parse(next()?)?);
                    time_control.time_limit = dur;
                    time_control.delay = Duration::default();
                }
                "infinite" => {
                    for c in Color::iter() {
                        zorro.time_controls[c] = TimeControl::infinite();
                    }
                }
                "ponder" => config.ponder = true,
                "perft" => return perft(zorro, tokens, output),
                _ => return Err(Error::Syntax),
            }
        }
        writeln!(output, "bestmove {}", iter_search(zorro).best_move)?;
        Ok(())
    }

    pub fn list_magics(mut output: impl io::Write) -> Result<()> {
        use crate::chess::Magic;
        for magic in Magic::by_file().iter() {
            writeln!(output, "{}", magic)?;
        }
        Ok(())
    }

    pub fn magic<'s>(
        mut tokens: impl Iterator<Item = &'s str>,
        mut output: impl io::Write,
    ) -> Result<()> {
        use crate::chess::tables;
        use crate::chess::Magic;
        let square = Square::from_str(tokens.next().unwrap()).unwrap();
        let kind = tokens.next().unwrap();
        let mut bb = tokens.next().unwrap_or("0").parse().unwrap();
        match kind {
            "file" => {
                bb = (*Magic::by_file())[square.i()].magify(bb);
                writeln!(output, "0x{:x}", bb)?;
            }
            "knight" => {
                bb = tables::KNIGHT_ATTACKS[square.i()];
                writeln!(output, "0x{:x}", bb)?;
            }
            _ => {}
        };
        Ok(())
    }

    pub fn open_uci_docs(mut output: impl io::Write) -> Result<()> {
        let url = "http://wbec-ridderkerk.nl/html/UCIProtocol.html";
        writeln!(output, "{}", url)?;
        webbrowser::open(url).map_err(|_| Error::Other)?;
        Ok(())
    }

    pub fn perft<'s>(
        zorro: &mut Zorro,
        mut tokens: impl Iterator<Item = &'s str>,
        mut output: impl io::Write,
    ) -> Result<()> {
        let token = tokens.next().unwrap_or("1");
        let depth = str::parse::<usize>(token)?;
        write!(
            output,
            "{}",
            crate::chess::perft::perft(&mut zorro.board, depth)
        )?;
        Ok(())
    }

    pub fn position<'s>(
        zorro: &mut Zorro,
        mut tokens: impl Iterator<Item = &'s str>,
    ) -> Result<()> {
        match tokens.next().unwrap_or("") {
            "startpos" => zorro.board = Board::default(),
            "fen" => zorro.board = Board::from_fen(&mut tokens)?,
            "960" => unimplemented!(),
            "current" => (),
            _ => return Err(Error::Syntax),
        }
        for token in tokens.skip_while(|s| *s == "moves") {
            zorro.board.do_move(Move::from_str(token)?);
        }
        Ok(())
    }

    fn option_check<S: AsRef<str>>(s: S) -> Result<bool> {
        match s.as_ref() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(Error::Syntax),
        }
    }

    fn option_spin<S: AsRef<str>>(s: S) -> Result<i32> {
        Ok(s.as_ref().parse::<i32>()?)
    }

    pub fn set_option<'s>(
        zorro: &mut Zorro,
        mut tokens: impl Iterator<Item = &'s str>,
    ) -> Result<()> {
        if tokens.next() != Some("name") {
            return Err(Error::Syntax);
        }
        let mut option_name = String::new();
        while let Some(token) = tokens.next() {
            if token == "value" {
                break;
            } else {
                option_name.push_str(token);
            }
        }
        let option_value = tokens.fold(String::new(), |mut base, s| {
            if !base.is_empty() {
                base.push_str(" ");
            }
            base.push_str(s);
            base
        });
        // Option support is quite hairy and messy. I don't want to break
        // pre-existing scripts and configs originally written for
        // other engines.
        //
        // Please see:
        //  - https://komodochess.com/Komodo-11-README.html
        //  - http://www.rybkachess.com/index.php?auswahl=Engine+parameters
        //
        // No worries in case the links above die, just search for a list of
        // UCI settings for popular chess engines. I don't commit to
        // 100% feature parity with any engine; I just try and use my
        // better judgement.
        match option_name.as_str() {
            "hash" => {
                //let cache_size =
                // ByteSize::mib(option_value.parse().unwrap());
                // zorro.config.cache_size = cache_size;
            }
            "ponder" => zorro.config.ponder = option_check(option_value)?,
            "nalimovpath" => {}
            "nalimovcache" => {}
            "ownbook" => {}
            "multipv" => {
                zorro.config.show_n_best = option_spin(option_value)? as i32
            }
            "uci_showcurrline" => {}
            "uci_showrefutations" => {}
            "uci_elo" => {}
            "uci_limitstrength" => {}
            "uci_opponent" => {
                use crate::elo::{expected_score, OWN_ELO};
                let mut tokens = option_value.split_whitespace();
                let _title = tokens.next();
                let elo = tokens.next().unwrap_or("none").parse::<u16>()?;
                zorro.config.contempt = expected_score(OWN_ELO, elo);
            }
            _ => (),
        };
        Ok(())
    }

    pub fn set_debug_mode<'a>(
        zorro: &mut Zorro,
        mut tokens: impl Iterator<Item = &'a str>,
    ) -> Result<()> {
        zorro.config.debug = match tokens.next() {
            Some("on") => true,
            Some("off") => false,
            _ => return Err(Error::Syntax),
        };
        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    Syntax,
    UnknownCommand(String),
    Chess(crate::err::Error),
    Io(io::Error),
    Other,
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<ChessErr> for Error {
    fn from(err: ChessErr) -> Self {
        Error::Chess(err)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(_err: std::num::ParseIntError) -> Self {
        Error::Syntax
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Syntax => write!(f, "[ERROR] Invalid command syntax"),
            Error::Other => {
                write!(f, "[ERROR] An unspecified error has happened")
            }
            Error::UnknownCommand(s) => {
                write!(f, "[ERROR] Unknown command '{}'", s)
            }
            Error::Chess(ChessErr::InvalidFen) => {
                write!(f, "[ERROR] Invalid FEN string")
            }
            Error::Chess(ChessErr::InvalidColor) => {
                write!(f, "[ERROR] Invalid color string")
            }
            Error::Chess(ChessErr::InvalidSquare) => {
                write!(f, "[ERROR] Invalid square string")
            }
            Error::Io(err) => {
                write!(f, "[ERROR] Fatal I/O condition ({})", err)
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::buf_to_str;

    #[test]
    fn d_fen_prints_fen() {
        let zorro = &mut Zorro::default();
        let mut output = vec![];
        handle_line(zorro, "uci", io::sink()).unwrap();
        handle_line(zorro, "position current moves b1c3", io::sink()).unwrap();
        let fen = zorro.board.fmt_fen(' ').to_string();
        handle_line(zorro, "d fen", &mut output).unwrap();
        assert_eq!(buf_to_str(&mut output), fen);
    }

    #[test]
    fn stop_cmd_triggers_shutdown() {
        assert_eq!(
            handle_line(&mut Zorro::default(), "stop", io::sink()).unwrap(),
            State::Shutdown
        );
    }

    #[test]
    fn quit_cmd_triggers_shutdown() {
        assert_eq!(
            handle_line(&mut Zorro::default(), "quit", io::sink()).unwrap(),
            State::Shutdown
        );
    }

    #[test]
    fn readyok_always_follows_isready() {
        let zorro = &mut Zorro::default();
        let mut output = vec![];
        handle_line(zorro, "uci", io::sink()).unwrap();
        handle_line(zorro, "isready", &mut output).unwrap();
        assert_eq!(buf_to_str(&mut output), "readyok");
    }
}
