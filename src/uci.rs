use std::str::FromStr;
use std::time::Duration;
use crate::{Board, FenParseError};
use crate::Move;
use crate::uci::UciCommand::Position;

#[derive(Debug)]
pub struct SetOptionArgs {
    pub name: String,
    pub value: String,
}

#[derive(Debug)]
pub struct GoArgs {
    pub search_moves: Vec<Move>,
    pub move_time: Option<Duration>,
    pub wtime: Option<Duration>,
    pub btime: Option<Duration>,
    pub winc: Option<Duration>,
    pub binc: Option<Duration>,
    pub depth: Option<u8>,
    pub nodes: Option<u64>,
    pub infinite: bool,
}

#[derive(Debug)]
pub enum GoError {
    UnrecognizedArgument(String),
    InvalidMove(String),
    InvalidWtime(String),
    InvalidBtime(String),
    InvalidWinc(String),
    InvalidBinc(String),
    InvalidDepth(String),
    InvalidNodes(String),
    I
}

#[derive(Debug)]
pub struct PositionArgs {
    pub position: Board,
    pub moves: Option<Vec<Move>>,
}

#[derive(Debug)]
pub enum PositionError {
    NoRootPosition,
    InvalidFen {
        error: FenParseError,
        fen: String,
    },
    InvalidMove(String),
}

#[derive(Debug)]
pub enum UciCommand {
    Uci,
    Debug(bool),
    SetOption(SetOptionArgs),
    IsReady,
    UciNewGame,
    Position(PositionArgs),
    Go(GoArgs),
    Stop,
}

#[derive(Debug)]
pub enum UciParseError {
    Empty,
    UnrecognizedCommand(String),
    WrongNumberOfArguments {
        cmd: String,
        expected: usize,
        got: usize,
    },
    MalformedArgument{
        cmd: String,
        arg: String,
    },
    Position(PositionError),
    Go(GoError),
}

impl FromStr for UciCommand {
    type Err = UciParseError;


    fn from_str(s: &str) -> Result<Self, Self::Err> {
        macro_rules! check_num_args {
            ($cmd:ident, $args:ident, $expected:expr) => {
                let len = $args.len();
                if len != $expected {
                    return Err(UciParseError::WrongNumberOfArguments {
                        cmd: $cmd.to_string(),
                        expected: $expected,
                        got: len,
                    })
                }
            };
        }

        use UciParseError as E;
        use UciCommand as Cmd;

        let mut split = s.split_whitespace();

        let cmd = split.next().ok_or(E::Empty)?;
        let args = split.collect::<Vec<&str>>();

        match cmd {
            "uci" => {
                check_num_args!(cmd, args, 0);
                Ok(Cmd::Uci)
            }
            "debug" => {
                check_num_args!(cmd, args, 1);

                let debug = args[0].parse::<bool>().map_err(
                    |_| E::MalformedArgument {
                        cmd: cmd.to_string(),
                        arg: args[0].to_string(),
                    }
                )?;

                Ok(Cmd::Debug(debug))
            }
            "setoption" => {
                check_num_args!(cmd, args, 2);
                let name = args[0].to_string();
                let value = args[1].to_string();

                Ok(Cmd::SetOption(SetOptionArgs{name, value}))
            }
            "isready" => {
                check_num_args!(cmd, args, 0);
                Ok(Cmd::IsReady)
            },
            "ucinewgame" => {
                check_num_args!(cmd, args, 0);
                Ok(Cmd::UciNewGame)
            },
            "position" => {
                let mut args_iter = args.into_iter();

                let mut board = match args_iter.next() {
                    Some("startpos") => Board::default(),
                    Some("fen") => {
                        let fen = args_iter
                            .by_ref()
                            .take(6)
                            .collect::<Vec<_>>()
                            .join(" ");

                        Board::from_fen(&fen).map_err(|error| {
                            E::Position(PositionError::InvalidFen {
                                error,
                                fen,
                            })
                        })?
                    }
                    _ => return Err(E::Position(PositionError::NoRootPosition)),
                };
                
                let moves = match args_iter.next() {
                    Some("moves") => {
                        let mut moves = Vec::new();

                        for mv in args_iter {
                            todo!("Parse move {mv}");
                        }

                        Some(moves)
                    }
                    None => None,
                    Some(other) => {
                        return Err(E::MalformedArgument {
                            cmd: cmd.to_string(),
                            arg: other.to_string(),
                        });
                    }
                };
                
                Ok(Position(PositionArgs {
                    position: board,
                    moves,
                }))
            }
            _ => Err(E::UnrecognizedCommand(cmd.to_string())),
        }
    }
}
