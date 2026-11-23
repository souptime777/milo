use std::{fmt::Display, ops::{Deref, DerefMut}, slice};

use crate::*;
use arrayvec::ArrayVec;

const MAX_MOVES_PER_POSITION: usize = 218;

#[repr(C, align(8))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoredMove {
    mv: Move,
    score: i32,
}

impl ScoredMove {
    #[inline(always)]
    pub fn new(mv: Move, score: i32) -> Self {
        Self { mv, score }
    }
}

#[derive(Clone, Debug)]
pub struct MoveList(ArrayVec<ScoredMove, MAX_MOVES_PER_POSITION>);

impl Default for MoveList {
    #[inline(always)]
    fn default() -> Self {
        MoveList::new()
    }
}

pub struct MoveListIter (arrayvec::IntoIter<ScoredMove, 218>);

impl Iterator for MoveListIter {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.0.next()?.mv)
    }
}

impl IntoIterator for MoveList {
    type Item = Move;
    type IntoIter = MoveListIter;

    fn into_iter(self) -> Self::IntoIter {
        MoveListIter(self.0.into_iter())
    }
}

impl Deref for MoveList {
    type Target = [ScoredMove];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MoveList {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for MoveList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for scored_move in self.iter() {
            writeln!(f, "{} {} ", scored_move.mv, scored_move.score)?;
        }
        Ok(())
    }
}

impl MoveList {
    #[inline(always)]
    pub fn new() -> Self {
        Self ( ArrayVec::new() )
    }

    #[inline(always)]
    pub fn push(&mut self, entry: ScoredMove) {
        self.0.push(entry);
    }

    #[inline(always)]
    pub fn push_move(&mut self, mv: Move) {
        debug_assert!(self.len() < MAX_MOVES_PER_POSITION);

        unsafe { self.0.push_unchecked(ScoredMove { mv, score: 0 }) };
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    #[inline(always)]
    pub fn pop_best(&mut self) -> Option<Move> {
        let (max_idx, _) = self.0
            .iter()
            .enumerate()
            .max_by_key(|(_, scored_move)| scored_move.score)?;

        Some(self.0.swap_remove(max_idx).mv)
    }

    #[inline(always)]
    pub fn pop_move(&mut self) -> Option<Move> {
        Some(self.0.pop()?.mv)
    }

    pub fn score_with<F>(&mut self, mut scorer: F) 
    where 
        F: FnMut(&Move) -> i32 
    {
        for scored_move in self.0.iter_mut() {
            scored_move.score = scorer(&scored_move.mv)
        }
    }

    #[inline(always)]
    pub fn iter(&self) -> slice::Iter<'_, ScoredMove> {
        self.0.iter()
    }

    #[inline(always)]
    pub fn iter_mut(&mut self) -> slice::IterMut<'_, ScoredMove> {
        self.0.iter_mut()
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
