use std::{
    cmp::Reverse, time::Instant,
};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use cozy_chess::{Board, GameStatus, Move, util::display_uci_move};
use uci_parser::{UciInfo, UciResponse};
use crate::eval::{PEICE_VALUES, evaluate};

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
        (key as usize) & (self.size-1)
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

// Return a score for move ordering to increase pruning
fn heuristic_move_score(board: &Board, mv: &Move) -> i32 {
    let mut score: i32 = 0;
    

    // Promotion bonus
    if let Some(promotion) = mv.promotion {
        score += 800 + PEICE_VALUES[1][promotion as usize];
    }

    // Prioritise capturing high value piece with low value piece
    if let Some(victim) = board.piece_on(mv.to) {
        if let Some(attacker) = board.piece_on(mv.from) {
            score += 5000 + 10 * 
            (PEICE_VALUES[1][victim as usize] - PEICE_VALUES[1][attacker as usize]);
        }
    }

    score
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

    pub fn iterative_search(board: Board, depth: u8, t_table_size: usize, verbose: bool, stop_flag: Arc<AtomicBool>) -> Move {
        let mut engine = Self::new(t_table_size);

        // Generate moves
        let mut moves: Vec<Move> = Vec::new();

        board.generate_moves(|mv| {
            moves.extend(mv);
            false
        });

        // Iteratively search, updating move order
        for d in 1..=depth {
            let start = Instant::now();
            let nodes: u64;

            let result = engine.search_fixed_depth(board.clone(), d, moves.clone(), Arc::clone(&stop_flag));

            if result.is_none() {
                break;
            }

            (moves, nodes) = result.unwrap();

            // Print info to terminal
            if verbose {
                let nps = (nodes as f32 / start.elapsed().as_secs_f32()) as u32;

                let info = UciInfo::new()
                .depth(d)
                .nodes(nodes)
                .nps(nps)
                .currmove(display_uci_move(&board, moves[0]))
                .time(start.elapsed().as_millis())
                .hashfull((engine.t_table.allocated * 1000) / (engine.t_table.size));
                
                

                println!("{}", UciResponse::info(info));
            }
        }

        moves[0]
    }

    pub fn search_fixed_depth(&mut self, board: Board, depth: u8, moves: Vec<Move>, stop_flag: Arc<AtomicBool>) -> Option<(Vec<Move>, u64)> {

        #[inline]
        fn lmr(depth: u8, move_idx: usize) -> u8 {
            if depth < 3 || move_idx < 3 {
                return 0;
            }

            let reduction = (0.3 * (depth as f32).ln() * (move_idx as f32).ln()) as u8;

            reduction.min(depth - 2)
        }

        let mut alpha = -i32::MAX;
        let beta = i32::MAX;
        let mut nodes_evaluated: u64 = 0;
        let mut values: Vec<i32> = Vec::with_capacity(moves.len());
        let mut max_score = -i32::MAX;
        // Loop over all moves
        for (i, mv) in moves.iter().enumerate() {

            let reduction = lmr(depth, i);

            let final_depth: u8 = depth - reduction;

        
            // Create a new board and run negamax
            let mut new_board = board.clone();
            new_board.play_unchecked(*mv);

            // Update values
            let result = self.negamax(new_board, final_depth-1, -beta, -alpha, &mut nodes_evaluated, Arc::clone(&stop_flag));

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

            values.push(score);
        }

        // Update move order
        let mut indices: Vec<usize> = (0..moves.len()).collect();
        indices.sort_unstable_by_key(|&i| Reverse(values[i]));

        Some((indices.iter().map(|&i| moves[i]).collect(), nodes_evaluated))
    }

    // Take a board and depth, returning eval
    pub fn negamax(&mut self, board: Board, depth: u8, mut alpha: i32, mut beta: i32, nodes: &mut u64, stop_flag: Arc<AtomicBool>) -> Option<i32> {
        if *nodes & 1023 == 0 && stop_flag.load(Ordering::Relaxed) {
            return None;
        }
        *nodes += 1;
        
        // Eval has cases for not ongoing state
        if board.status() != GameStatus::Ongoing {
            return Some(evaluate(&board, depth));
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

        // Evaluate at leaf, updating transposition table
        if depth == 0 {
            return Some(self.quiescence_search(board, depth, -beta, -alpha, nodes));
        }
        

        // Run search on all moves, with alphabeta pruning
        let mut max_score: i32 = -i32::MAX;
        if let Some(tt_move) = bestmove {
            let mut new_board= board.clone();
            new_board.play_unchecked(tt_move);

            let result = self.negamax(new_board, depth-1, -beta, -alpha, nodes, Arc::clone(&stop_flag));

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

                let mut new_board= board.clone();
                new_board.play_unchecked(mv);
                let result = self.negamax(new_board, depth-1, -beta, -alpha, nodes, Arc::clone(&stop_flag));

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
                    break;
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

    fn quiescence_search(&self, board: Board, depth: u8, mut alpha: i32, beta: i32, nodes: &mut u64) -> i32{
        let static_eval = evaluate(&board, depth);
        *nodes += 1;

        let in_check = !board.checkers().is_empty();
        let mut best_value = if in_check { -i32::MAX } else { static_eval };

        if !in_check {
            // Stand Pat
            if best_value >= beta {
                return best_value;
            }

            if best_value > alpha {
                alpha = best_value;
            }
        }

        let enemy_pieces = board.colors(!board.side_to_move());
        board.generate_moves(|moves| {

            for mv in moves {
                // Extend search on captures, promotions and checks
                if in_check || (!mv.promotion.is_none() || !(mv.to.bitboard() & enemy_pieces).is_empty()) {
                    let mut new_board = board.clone();
                    new_board.play_unchecked(mv);
                    let score = -self.quiescence_search(new_board, depth, -beta, -alpha, nodes);

                    if score >= beta {
                        return true;
                    }

                    if score > best_value {
                        best_value = score;
                    }

                    if score > alpha {
                        alpha = score;
                    }
                }
                
                
                
            }
            false
        });

        best_value
    }
}