use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::{cmp::Reverse, time::Instant};

use crate::eval::{PEICE_VALUES, evaluate, phase};
use cozy_chess::{Board, GameStatus, Move, util::display_uci_move};
use cozy_chess::{BoardBuilder, Color, Piece, Rank};
use uci_parser::{UciInfo, UciResponse, UciScore};

const MAX_QUIESCENCE_PLY: usize = 32;
const NMP_REDUCTION: u8 = 2;

#[derive(Clone)]
struct TTEntry {
    key: u64,

    score: i32,
    bestmove: Option<Move>,
    depth: u8,
    bound: Bound,
}

#[derive(Clone, Copy)]
enum Bound {
    Exact,
    Lower,
    Upper,
}

struct TranspositionTable {
    table: Vec<Option<TTEntry>>,
    allocated: usize,
    size: usize,
}

impl TranspositionTable {
    fn new(size: usize) -> Self {
        Self {
            table: vec![None; size],
            allocated: 0,
            size,
        }
    }

    fn get_idx(&self, key: u64) -> usize {
        (key as usize) & (self.size - 1)
    }

    fn entry_at_key(&self, key: u64) -> &Option<TTEntry> {
        &self.table[self.get_idx(key)]
    }

    fn set(&mut self, entry: TTEntry) {
        let idx = self.get_idx(entry.key);
        if self.table[idx].is_none() {
            self.allocated += 1;
        }
        self.table[idx] = Some(entry);
    }
}

#[derive(Debug)]
pub struct SearchInfo {
    depth: u8,
    seldepth: usize,
    nodes: u64,
    score: i32,
    researches: u32,
}

impl SearchInfo {
    fn new() -> Self {
        Self {
            depth: 0,
            seldepth: 0,
            nodes: 0,
            score: 0,
            researches: 0,
        }
    }

    fn update_seldepth(&mut self, ply: usize) {
        if ply > self.seldepth {
            self.seldepth = ply;
        }
    }
}

pub struct Engine {
    t_table: TranspositionTable,
}

impl Engine {
    pub fn new(t_table_size: usize) -> Self {
        assert!(t_table_size.is_power_of_two());
        Self {
            t_table: TranspositionTable::new(t_table_size),
        }
    }

    pub fn iterative_search(
        board: Board,
        depth: u8,
        t_table_size: usize,
        verbose: bool,
        stop_flag: Arc<AtomicBool>,
    ) -> Move {
        let mut engine = Self::new(t_table_size);

        // Generate moves
        let mut moves: Vec<Move> = Vec::new();
        let mut info: SearchInfo;

        board.generate_moves(|mv| {
            moves.extend(mv);
            false
        });

        // Iteratively search, updating move order
        for d in 1..=depth {
            let start = Instant::now();

            let result =
                engine.search_fixed_depth(board.clone(), d, moves.clone(), Arc::clone(&stop_flag));

            if result.is_none() {
                break;
            }

            (moves, info) = result.unwrap();

            // Print info to terminal
            if verbose {
                let nps = (info.nodes as f32 / start.elapsed().as_secs_f32()) as u32;

                let uci_info = UciInfo::new()
                    .depth(d)
                    .seldepth(info.seldepth)
                    .nodes(info.nodes)
                    .score(UciScore::cp(info.score))
                    .nps(nps)
                    .currmove(display_uci_move(&board, moves[0]))
                    .time(start.elapsed().as_millis())
                    .hashfull((engine.t_table.allocated * 1000) / (engine.t_table.size));

                println!("{}", UciResponse::info(uci_info));
            }
        }

        moves[0]
    }

    pub fn search_fixed_depth(
        &mut self,
        board: Board,
        depth: u8,
        moves: Vec<Move>,
        stop_flag: Arc<AtomicBool>,
    ) -> Option<(Vec<Move>, SearchInfo)> {
        #[inline]
        fn lmr(depth: u8, move_idx: usize) -> u8 {
            if depth <= 4 || move_idx <= 4 {
                return 0;
            }

            let reduction = (0.20 * (depth as f32).ln() * (move_idx as f32).ln()) as u8;

            reduction.min(depth - 4)
        }

        #[inline]
        fn can_apply_lmr(board: &Board, mv: &Move) -> bool {
            // Root is in check:
            if !board.checkers().is_empty() {
                return false;
            }

            // Move is capture
            if board.piece_on(mv.to).is_some() {
                return false;
            }

            // Move is promotion
            if let Some(piece) = board.piece_on(mv.from) {
                let rank = mv.to.rank();
                if piece == Piece::Pawn && (rank == Rank::First || rank == Rank::Eighth) {
                    return false;
                }
            }

            /* Move gives check
            Creating new board only acceptable because this is only done at root.
            */
            let mut test_board = board.clone();
            test_board.play_unchecked(*mv);

            if !test_board.checkers().is_empty() {
                return false;
            }

            true
        }

        let mut info = SearchInfo::new();
        info.depth = depth;

        let alpha = -i32::MAX;
        let beta = i32::MAX;

        let mut values: Vec<i32> = Vec::with_capacity(moves.len());
        let mut max_score = -i32::MAX;

        // Loop over all moves
        for (move_idx, mv) in moves.iter().enumerate() {
            let reduction = if can_apply_lmr(&board, mv) {
                lmr(depth, move_idx)
            } else {
                0
            };

            // Create a new board and run negamax
            let mut new_board = board.clone();
            new_board.play_unchecked(*mv);

            // negamax search
            let result = self.negamax(
                new_board,
                depth - 1 - reduction,
                0,
                -beta,
                -alpha,
                &mut info,
                true,
                Arc::clone(&stop_flag),
            );

            result?;

            let mut score = -result.unwrap();

            // Research if reduced move triggered cutoff
            if reduction != 0 && score > alpha {
                info.researches += 1;

                let mut new_board = board.clone();
                new_board.play_unchecked(*mv);
                let result = self.negamax(
                    new_board,
                    depth - 1,
                    0,
                    -beta,
                    -alpha,
                    &mut info,
                    true,
                    Arc::clone(&stop_flag),
                );

                result?;

                score = -result.unwrap();
            }

            if score > max_score {
                max_score = score;
            }

            values.push(score);
        }

        info.score = max_score;

        // Update move order
        let mut indices: Vec<usize> = (0..moves.len()).collect();
        indices.sort_unstable_by_key(|&i| Reverse(values[i]));

        Some((indices.iter().map(|&i| moves[i]).collect(), info))
    }

    // Take a board and depth, returning eval
    pub fn negamax(
        &mut self,
        board: Board,
        depth: u8,
        ply: usize,
        mut alpha: i32,
        mut beta: i32,
        info: &mut SearchInfo,
        allow_null: bool,
        stop_flag: Arc<AtomicBool>,
    ) -> Option<i32> {
        #[inline]
        fn calculate_extensions(board: &Board) -> u8 {
            let mut extension = 0;

            if !board.checkers().is_empty() {
                extension += 1;
            }
            extension
        }

        if info.nodes & 1023 == 0 && stop_flag.load(Ordering::Relaxed) {
            return None;
        }

        info.nodes += 1;
        info.update_seldepth(ply);

        // Eval has cases for not ongoing state
        if board.status() != GameStatus::Ongoing {
            return Some(evaluate(&board, ply));
        }

        // Null Move Pruning
        if board.checkers().is_empty()
            && allow_null
            && depth > NMP_REDUCTION + 1
            && phase(&board) > 128
        {
            let mut builder = BoardBuilder::from_board(&board);

            builder.side_to_move = !board.side_to_move();
            builder.en_passant = None;

            if let Ok(null_board) = builder.build() {
                let null_value = self.negamax(
                    null_board,
                    depth - NMP_REDUCTION - 1,
                    ply + 1,
                    -beta,
                    alpha,
                    info,
                    false,
                    Arc::clone(&stop_flag),
                )?;
                if null_value >= beta {
                    return Some(null_value);
                }
            }
        }

        // Return transposition table value if available and good
        let key = board.hash();
        let alpha_original = alpha;
        let beta_original = beta;

        let mut bestmove: Option<Move> = None;

        if let Some(entry) = self.t_table.entry_at_key(key) {
            if entry.key == key {
                bestmove = entry.bestmove;
                if entry.depth >= depth {
                    match entry.bound {
                        Bound::Exact => return Some(entry.score),
                        Bound::Lower => alpha = alpha.max(entry.score),
                        Bound::Upper => beta = beta.min(entry.score),
                    }
                    if alpha >= beta {
                        return Some(entry.score);
                    }
                }
            }
        }

        // Quiesce at leaf, updating transposition table
        if depth == 0 {
            return self.quiesce(board, ply + 1, 0, -beta, -alpha, info, &stop_flag);
        }

        // Run search on all moves, with alphabeta pruning
        let extension = calculate_extensions(&board);

        let mut max_score: i32 = -i32::MAX;
        if let Some(tt_move) = bestmove {
            let mut new_board = board.clone();
            new_board.play_unchecked(tt_move);

            let result = self.negamax(
                new_board,
                depth - 1 + extension,
                ply + 1,
                -beta,
                -alpha,
                info,
                allow_null,
                Arc::clone(&stop_flag),
            );

            if result.is_none() {
                return None;
            }

            let score = -result.unwrap();

            if score > max_score {
                max_score = score;
            }
            if max_score > alpha {
                alpha = max_score;
            }
            if alpha >= beta {
                return Some(max_score);
            }
        }

        let mut any_none = false;

        board.generate_moves(|moves| {
            // Sort

            for mv in moves {
                if let Some(tt_move) = bestmove {
                    if tt_move == mv {
                        continue;
                    }
                }

                let mut new_board = board.clone();
                new_board.play_unchecked(mv);
                let result = self.negamax(
                    new_board,
                    depth - 1 + extension,
                    ply + 1,
                    -beta,
                    -alpha,
                    info,
                    allow_null,
                    Arc::clone(&stop_flag),
                );

                if result.is_none() {
                    any_none = true;
                    return true;
                }

                let score = -result.unwrap();

                if score > max_score {
                    max_score = score;
                    bestmove = Some(mv);
                }
                if max_score > alpha {
                    alpha = max_score;
                }
                if alpha >= beta {
                    return true;
                }
            }
            false
        });

        if any_none {
            return None;
        }

        self.t_table.set(TTEntry {
            key,

            bestmove,
            depth,
            score: max_score,
            bound: if max_score <= alpha_original {
                Bound::Upper
            } else if max_score >= beta_original {
                Bound::Lower
            } else {
                Bound::Exact
            },
        });

        Some(max_score)
    }

    fn quiesce(
        &self,
        board: Board,
        ply: usize,
        quiescence_ply: usize,
        mut alpha: i32,
        beta: i32,
        info: &mut SearchInfo,
        stop_flag: &AtomicBool,
    ) -> Option<i32> {
        info.nodes += 1;
        info.update_seldepth(ply);

        if info.nodes & 1023 == 0 && stop_flag.load(Ordering::Relaxed) {
            return None;
        }

        if board.status() != GameStatus::Ongoing {
            return Some(evaluate(&board, ply));
        }

        let static_eval = evaluate(&board, ply);
        if quiescence_ply >= MAX_QUIESCENCE_PLY {
            return Some(static_eval);
        }

        let in_check = !board.checkers().is_empty();
        let mut best_value = if in_check { -i32::MAX } else { static_eval };

        if !in_check {
            // Stand Pat
            if best_value >= beta {
                return Some(best_value);
            }

            if best_value > alpha {
                alpha = best_value;
            }
        }

        let enemy_pieces = board.colors(!board.side_to_move());
        let mut interrupted = false;
        board.generate_moves(|moves| {
            for mv in moves {
                // Extend captures, promotions, and all check evasions.
                if in_check
                    || !mv.promotion.is_none()
                    || !(mv.to.bitboard() & enemy_pieces).is_empty()
                {
                    let mut new_board = board.clone();
                    new_board.play_unchecked(mv);
                    let result = self.quiesce(
                        new_board,
                        ply + 1,
                        quiescence_ply + 1,
                        -beta,
                        -alpha,
                        info,
                        stop_flag,
                    );
                    let Some(score) = result else {
                        interrupted = true;
                        return true;
                    };
                    let score = -score;

                    if score > best_value {
                        best_value = score;
                    }

                    if score > alpha {
                        alpha = score;
                    }

                    if score >= beta {
                        return true;
                    }
                }
            }
            false
        });

        if interrupted { None } else { Some(best_value) }
    }
}
