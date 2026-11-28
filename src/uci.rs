use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::time::Duration;
use arrayvec::ArrayVec;
use crate::{Board, FenParseError, MoveList, PieceType, Square};
use crate::Move;

/// A Move parsed from UCI, without a flag.
#[derive(Debug)]
pub struct UciMove {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<PieceType>,
}
#[derive(Debug)]
pub enum UciMoveParseError {
    InvalidFromSquare,
    InvalidToSquare,
    InvalidPromotionPiece,
    InvalidUciFormat,
}

impl FromStr for UciMove {
    type Err = UciMoveParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !(4..=5).contains(&s.len()) || !s.is_ascii() {
            return Err(UciMoveParseError::InvalidUciFormat);
        }

        let from: Square = s[0..=1].parse().map_err(|_| UciMoveParseError::InvalidFromSquare)?;
        let to: Square = s[2..=3].parse().map_err(|_| UciMoveParseError::InvalidToSquare)?;
        let promo  = s.chars()
            .nth(4)
            .map(|c| PieceType::try_from(c))
            .transpose()
            .map_err(|_| UciMoveParseError::InvalidPromotionPiece)?;

        Ok(Self::new(from, to, promo))
    }
}

impl UciMove {
    pub fn new(from: Square, to: Square, promotion: Option<PieceType>) -> Self {
        Self { from, to, promotion }
    }

    /// Resolve a parsed UCI move to a Move struct applicable to a board.
    pub fn resolve(self, board: &Board) -> Result<Move, &'static str> {
        let mut legals = MoveList::new();
        board.generate_moves_into(&mut legals);

        for legal in legals {
            if legal.from() == self.from
                && legal.to() == self.to {

                if !legal.is_promotion() && self.promotion.is_some() {
                    continue;
                }

                if legal.is_promotion() {
                    if let Some(promo) = self.promotion
                        && legal.promotion_piece() == promo {
                        return Ok(legal);
                    }
                    continue;
                }
                return Ok(legal);
            }
        }
        Err("Illegal move")
    }
}

#[derive(Debug)]
pub struct UciMoveList(ArrayVec<UciMove, 256>);

impl UciMoveList {
    pub fn new() -> Self {
        Self(ArrayVec::new())
    }
    pub fn push(&mut self, mv: UciMove) {
        self.0.push(mv);
    }
    pub fn pop(&mut self) -> Option<UciMove> {
        self.0.pop()
    }
    /// Convert the parsed move list into a real Move list
    pub fn resolve(self, board: &Board) -> Result<MoveList, &'static str> {
        let mut move_list = MoveList::new();
        for uci in self.0 {
            let mv = uci.resolve(board)?;
            move_list.push_move(mv);
        }
        Ok(move_list)
    }
}

#[derive(Debug)]
pub struct SetOptionArgs {
    pub name: String,
    pub value: String,
}

#[derive(Debug)]
pub struct GoArgs {
    pub search_moves: Option<UciMoveList>,
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
    MissingArgument,
    InvalidMovetime(String),
    InvalidWtime(String),
    InvalidBtime(String),
    InvalidWinc(String),
    InvalidBinc(String),
    InvalidDepth(String),
    InvalidNodes(String),
    I
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
    Position(Board),
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

                match args_iter.next() {
                    Some("moves") => {
                        for s in args_iter {
                            if let Ok(mv) = Move::from_uci(s, &board) {
                                board.make_move(mv);
                            } else {
                                return Err(E::Position(PositionError::InvalidMove(s.to_string())));
                            }
                        }
                    }
                    None => {},
                    Some(other) => {
                        return Err(E::MalformedArgument {
                            cmd: cmd.to_string(),
                            arg: other.to_string(),
                        });
                    }
                };

                Ok(Cmd::Position(board))
            }
            "go" => {
                let mut args_iter = args.into_iter().peekable();

                let mut search_moves: Option<UciMoveList> = None;
                let mut move_time: Option<Duration> = None;
                let mut wtime: Option<Duration> = None;
                let mut btime: Option<Duration> = None;
                let mut winc: Option<Duration> = None;
                let mut binc: Option<Duration> = None;
                let mut depth: Option<u8> = None;
                let mut nodes: Option<u64> = None;
                let mut infinite = false;

                macro_rules! time_set {
                    ($var:ident, $err:ident) => {
                        let arg = args_iter.next()
                            .ok_or(E::Go(GoError::MissingArgument))?;

                        $var = Some(Duration::from_millis(
                            arg.parse::<u64>()
                                .map_err(|_| E::Go(GoError::$err(arg.to_string())))?
                        ));
                    };
                }
                while let Some(arg) = args_iter.next() {
                    match arg {
                        "searchmoves" => {
                            let mut uci_move_list = UciMoveList::new();
                            while let Some(uci) = args_iter.peek() {
                                if let Ok(mv) = uci.parse::<UciMove>() {
                                    uci_move_list.push(mv);
                                    // Consume only if valid move
                                    let _ = args_iter.next();
                                    continue;
                                }
                                break;
                            }
                            search_moves = Some(uci_move_list);
                        }
                        "movetime" => { time_set!(move_time, InvalidMovetime); },
                        "wtime" => { time_set!(wtime, InvalidWtime); },
                        "btime" => { time_set!(btime, InvalidBtime); },
                        "winc" => { time_set!(winc, InvalidWinc); },
                        "binc" => { time_set!(binc, InvalidBinc); },
                        "depth" => {
                            let arg = args_iter.next()
                                .ok_or(E::Go(GoError::MissingArgument))?;
                            depth = Some(arg.parse::<u8>().map_err(|_| E::Go(GoError::InvalidDepth(arg.to_string())))?);
                        }
                        "nodes" => {
                            let arg = args_iter.next()
                                .ok_or(E::Go(GoError::MissingArgument))?;
                            nodes = Some(arg.parse::<u64>().map_err(|_| E::Go(GoError::InvalidNodes(arg.to_string())))?);
                        }
                        "infinite" => { infinite = true; },

                        _ => { return Err(E::Go(GoError::UnrecognizedArgument(arg.to_string()))) },
                    }
                }
                Ok(Cmd::Go(GoArgs {
                    search_moves,
                    move_time,
                    wtime,
                    winc,
                    btime,
                    binc,
                    depth,
                    nodes,
                    infinite,
                }))
            }
            _ => Err(E::UnrecognizedCommand(cmd.to_string())),
        }
    }
}

pub struct UciBestMove {
    pub best_move: Move,
    pub ponder: Option<Move>,
}
impl Display for UciBestMove {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(ponder) = &self.ponder {
            write!(f, "bestmove {} ponder {}", self.best_move, ponder)?;
        } else {
            write!(f, "bestmove {}", self.best_move)?;
        }
        Ok(())
    }
}
impl UciBestMove {
    pub fn new(best_move: Move) -> Self {
        Self {
            best_move,
            ponder: None,
        }
    }
    pub fn ponder(mut self, ponder: Move) -> Self {
        self.ponder = Some(ponder);
        self
    }
}

pub enum UciScore {
    Centipawns(i32),
    Mate(u8),
}

pub struct UciInfo {
    pub depth: Option<u8>,
    pub seldepth: Option<u8>,
    pub time: Option<Duration>,
    pub nodes: Option<u64>,
    pub pv: Option<Vec<Move>>,
    pub score: Option<UciScore>,
    pub currmove: Option<Move>,
    pub hashfull: Option<u16>,
    pub nps: Option<u64>,
    pub tbhits: Option<u64>,
    pub sbhits: Option<u64>,
    pub string: Option<String>,
}

impl Display for UciInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "info ")?;
        if let Some(depth) = self.depth {
            write!(f, "depth {} ", depth)?;
        }
        if let Some(seldepth) = self.seldepth {
            write!(f, "seldepth {} ", seldepth)?;
        }
        if let Some(time) = self.time {
            write!(f, "time {} ", time.as_millis())?;
        }
        if let Some(nodes) = self.nodes {
            write!(f, "nodes {} ", nodes)?;
        }
        if let Some(pv) = &self.pv {
            write!(f, "pv ")?;
            for mv in pv {
                write!(f, "{} ", mv)?;
            }
        }
        if let Some(score) = &self.score {
            write!(f, "score ")?;
            match score {
                UciScore::Centipawns(c) => {
                    write!(f, "cp {} ", c)?;
                }
                UciScore::Mate(m) => {
                    write!(f, "mate {} ", m)?;
                }
            }
        }
        if let Some(currmove) = &self.currmove {
            write!(f, "currmove {} ", currmove)?;
        }
        if let Some(hashfull) = self.hashfull {
            write!(f, "hashfull {} ", hashfull)?;
        }
        if let Some(nps) = self.nps {
            write!(f, "nps {} ", nps)?;
        }
        if let Some(tbhits) = self.tbhits {
            write!(f, "tbhits {} ", tbhits)?;
        }
        if let Some(sbhits) = self.sbhits {
            write!(f, "sbhits {} ", sbhits)?;
        }
        if let Some(string) = &self.string {
            write!(f, "string {} ", string)?;
        }
        Ok(())
    }
}

impl UciInfo {
    pub fn new() -> Self {
        Self {
            depth: None,
            seldepth: None,
            time: None,
            nodes: None,
            pv: None,
            score: None,
            currmove: None,
            hashfull: None,
            nps: None,
            tbhits: None,
            sbhits: None,
            string: None,
        }
    }
    pub fn depth(mut self, depth: u8) -> Self {
        self.depth = Some(depth);
        self
    }
    pub fn seldepth(mut self, seldepth: u8) -> Self {
        self.seldepth = Some(seldepth);
        self
    }
    pub fn time(mut self, time: Duration) -> Self {
        self.time = Some(time);
        self
    }
    pub fn nodes(mut self, nodes: u64) -> Self {
        self.nodes = Some(nodes);
        self
    }
    pub fn pv(mut self, pv: impl IntoIterator<Item = Move>) -> Self {
        self.pv = Some(pv.into_iter().collect());
        self
    }
    pub fn score(mut self, score: UciScore) -> Self {
        self.score = Some(score);
        self
    }
    pub fn currmove(mut self, currmove: Move) -> Self {
        self.currmove = Some(currmove);
        self
    }
    pub fn hashfull(mut self, hashfull: u16) -> Self {
        self.hashfull = Some(hashfull);
        self
    }
    pub fn nps(mut self, nps: u64) -> Self {
        self.nps = Some(nps);
        self
    }
    pub fn tbhits(mut self, tbhits: u64) -> Self {
        self.tbhits = Some(tbhits);
        self
    }
    pub fn sbhits(mut self, sbhits: u64) -> Self {
        self.sbhits = Some(sbhits);
        self
    }
    pub fn string(mut self, string: String) -> Self {
        self.string = Some(string);
        self
    }
}

pub enum UciOptionType {
    Check {
        default: bool,
    },
    Spin {
        default: i32,
        min: i32,
        max: i32,
    },
    Combo {
        default: String,
        var: Vec<String>,
    },
    Button,
    String {
        default: String,
    },
}

pub struct UciOption {
    pub name: String,
    pub option_type: UciOptionType,
}

impl Display for UciOption {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "option name {} ", self.name)?;
        match &self.option_type {
            UciOptionType::Check { default } => {
                write!(f, "type check ")?;
                write!(f, "default {} ", default)?;
            }
            UciOptionType::Spin { default, min, max } => {
                write!(f, "type spin ")?;
                write!(f, "default {} ", default)?;
                write!(f, "min {} max {} ", min, max)?;

            }
            UciOptionType::Combo { default, var } => {
                write!(f, "type combo ")?;
                write!(f, "default {} ", default)?;
                for var in var {
                    write!(f, "var {} ", var)?;
                }
            }
            UciOptionType::Button => {
                write!(f, "type button ")?;
            }
            UciOptionType::String { default } => {
                write!(f, "type string ")?;
                write!(f, "default {} ", default)?;
            }
        }
        Ok(())
    }
}
impl UciOption {
    pub fn new(name: &str, option_type: UciOptionType) -> Self {
        Self {
            name: name.to_string(),
            option_type,
        }
    }
}

pub enum UciResponse {
    Id {
        name: String,
        author: String,
    },
    UciOk,
    ReadyOk,
    Option(UciOption),
    BestMove(UciBestMove),
    Info(UciInfo),
}

impl Display for UciResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UciResponse::Id { name, author } => {
                writeln!(f, "id name {}", name)?;
                write!(f, "id author {}", author)?;
            }
            UciResponse::UciOk => {
                write!(f, "uciok")?;
            }
            UciResponse::ReadyOk => {
                write!(f, "readyok")?;
            }
            UciResponse::Option(option) => {
                write!(f, "{}", option)?;
            }
            UciResponse::BestMove(bestmove) => {
                write!(f, "{}", bestmove)?;
            }
            UciResponse::Info(info) => {
                write!(f, "{}", info)?;
            }
        }
        Ok(())
    }
}

