use std::{
    cmp::Reverse, time::Instant,
};

use cozy_chess::{Board, GameStatus, Move};
use crate::eval::evaluate;

#[derive(Clone)]
struct TTEntry {
    key: u64,
    score: i32,
    depth: u8,
}

struct TranspositionTable {
    table: Vec<Option<TTEntry>>,
    size: usize,
}

impl TranspositionTable {

    fn new(size: usize) -> Self {
        Self {
            table: vec![None; size],
            size,
        }
    }

    fn get_idx(&self, key: u64) -> usize{
        (key as usize) % self.size
    }

    fn entry_at_key(&self, key: u64) -> &Option<TTEntry> {
        &self.table[self.get_idx(key)]
    }

    fn set(&mut self, entry: TTEntry) {
        let idx = self.get_idx(entry.key);
        self.table[idx] = Some(entry);
    }
}

pub struct Engine {
    t_table: TranspositionTable,
}

impl Engine {
    pub fn new(t_table_size: usize) -> Self {
        Self {
            t_table: TranspositionTable::new(t_table_size),
        }
    }

    pub fn iterative_search(board: Board, depth: u8, t_table_size: usize, verbose: bool) -> Move {
        let mut engine = Self::new(t_table_size);
        // Create a vector of all moves
        let mut moves: Vec<Move> = Vec::new();
        

        board.generate_moves(|mv| {
            moves.extend(mv);
            false
        });

        // Iteratively search, updating move order
        for d in 1..=depth {
            let start = Instant::now();
            let nodes: u64;

            (moves, nodes) = engine.search_fixed_depth(board.clone(), d, moves);
            if verbose {
                let rate = nodes as f32 / start.elapsed().as_secs_f32();
                println!("depth {}: evaluated {} nodes in {:.2} seconds ({:.2}nps)", d, nodes, start.elapsed().as_secs_f32(), rate)
            }
        }

        moves[0]
    }

    pub fn search_fixed_depth(&mut self, board: Board, depth: u8, moves: Vec<Move>) -> (Vec<Move>, u64) {
        let mut alpha = -i32::MAX;
        let beta = i32::MAX;

        let mut nodes_evaluated: u64 = 0;
        let mut values: Vec<i32> = Vec::with_capacity(moves.len());
        let mut max_score = -i32::MAX;
        // Loop over all moves
        for mv in &moves {

            // Create a new board and run negamax
            let mut new_board = board.clone();
            new_board.play_unchecked(*mv);

            // Update values
            let score = -self.negamax(new_board, depth-1, -beta, -alpha, &mut nodes_evaluated);

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

        (indices.iter().map(|&i| moves[i]).collect(), nodes_evaluated)
    }


    // Take a board and depth, returning eval
    pub fn negamax(&mut self, board: Board, depth: u8, mut alpha: i32, beta: i32, mut nodes: &mut u64) -> i32 {

        // Return transposition table value if available and good
        let key = board.hash();
        let mut entry_needs_update = false;
        if let Some(entry) = self.t_table.entry_at_key(key) {
            if entry.key == key && entry.depth >= depth {
                *nodes += 1;
                return entry.score;
                
            } else {
                entry_needs_update = true;
            }
        }

        // Eval has cases for not ongoing state
        if board.status() != GameStatus::Ongoing {
            *nodes += 1;
            return evaluate(&board, depth);
        }

        // Evaluate at leaf, updating transposition table
        if depth == 0 {
            *nodes += 1;

            let eval = -self.quiescence_search(board, depth, -beta, -alpha, nodes);
            return eval;
        }
        


        let mut max_score: i32 = -i32::MAX;

        board.generate_moves(|moves| {
            for mv in moves {
                let mut new_board= board.clone();
                new_board.play_unchecked(mv);
                let score = -self.negamax(new_board, depth-1, -beta, -alpha, nodes);

                if score > max_score {
                    max_score = score;
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

        if entry_needs_update {
            self.t_table.set(TTEntry {
                key,
                depth,
                score: max_score,
            });
        }

        max_score
    }

    fn quiescence_search(&self, board: Board, depth: u8, mut alpha: i32, beta: i32, mut nodes: &mut u64) -> i32{
        let static_eval = evaluate(&board, depth);
        *nodes += 1;

        let mut best_value = static_eval;

        // Stand Pat
        if best_value >= beta {
            return best_value;
        }

        if best_value > alpha {
            alpha = best_value;
        }

        let enemy_pieces = board.colors(!board.side_to_move());
        board.generate_moves(|moves| {

            for mv in moves {
                // Extend search on captures and promotions
                if !mv.promotion.is_none() || !(mv.to.bitboard() & enemy_pieces).is_empty() {
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