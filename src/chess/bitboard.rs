use core::ops::*;
use core::fmt;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash, Debug)]
/// A bitboard, represented by [u64].
/// For more information, see <https://www.chessprogramming.org/Bitboards>
pub struct Bitboard(
    /// The inner bit representation.
    pub u64
);

macro_rules! impl_bitwise_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait for Bitboard {
            type Output = Self;

            #[inline(always)]
            fn $fn(self, rhs: Self) -> Self::Output {
                Self($trait::$fn(self.0, rhs.0))
            }
        }
    )*};
}

impl_bitwise_ops! {
    BitAnd, bitand;
    BitOr, bitor;
    BitXor, bitxor;
}

macro_rules! impl_bitwise_assign_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait for Bitboard {

            #[inline(always)]
            fn $fn(&mut self, rhs: Self) {
                ($trait::$fn(&mut self.0, rhs.0))
            }
        }
    )*};
}

impl_bitwise_assign_ops! {
    BitAndAssign, bitand_assign;
    BitOrAssign, bitor_assign;
    BitXorAssign, bitxor_assign;
}

impl Not for Bitboard {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl Sub for Bitboard {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        self & !rhs
    }
}

impl SubAssign for Bitboard {
    
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const OCCUPIED_STR: &str = "X ";
        const UNOCCUPIED_STR: &str = ". ";

        for bit_idx in 0..64 { 

            // Newline every 8 bits
            if bit_idx % 8 == 0 && bit_idx != 0 {
                writeln!(f)?;
            }
            
            // Print bit
            let bit = (self.0 >> bit_idx) & 1;
            f.write_str(if bit == 1 {OCCUPIED_STR} else {UNOCCUPIED_STR})?;
        }

        Ok(())
    }
}

impl Bitboard {
    /// An empty [Bitboard].
    pub const EMPTY: Self = Self(0);

    /// A full [Bitboard].
    pub const FULL: Self = Self(!0);

    /// A [Bitboard] containing all light squares.
    pub const LIGHT_SQUARES: Self = bitboard!(
        X . X . X . X .
        . X . X . X . X
        X . X . X . X .
        . X . X . X . X
        X . X . X . X .
        . X . X . X . X
        X . X . X . X .
        . X . X . X . X
    );

    /// A [Bitboard] containing all dark squares.
    pub const DARK_SQUARES: Self = bitboard!(
        . X . X . X . X
        X . X . X . X .
        . X . X . X . X
        X . X . X . X .
        . X . X . X . X
        X . X . X . X .
        . X . X . X . X
        X . X . X . X .
    );

    /// A [Bitboard] containing all squares on the edge.
    pub const EDGES: Self = bitboard!(
        X X X X X X X X
        X . . . . . . X
        X . . . . . . X
        X . . . . . . X
        X . . . . . . X
        X . . . . . . X
        X . . . . . . X
        X X X X X X X X
    );

    /// A [Bitboard] containing the A and H files.
    pub const EDGES_FILES: Self = bitboard!(
        X . . . . . . X
        X . . . . . . X
        X . . . . . . X
        X . . . . . . X
        X . . . . . . X
        X . . . . . . X
        X . . . . . . X
        X . . . . . . X
    );

    /// A [Bitboard] containing the First and Eighth ranks.
    pub const EDGES_RANKS: Self = bitboard!(
        X X X X X X X X
        . . . . . . . .
        . . . . . . . .
        . . . . . . . .
        . . . . . . . .
        . . . . . . . .
        . . . . . . . .
        X X X X X X X X
    );

    /// Checks whether the [Bitboard] is empty.
    /// 
    /// # Examples
    /// 
    /// ``` 
    /// # use milo_core::Bitboard;
    /// let bb = Bitboard::EMPTY;
    /// assert!(bb.is_empty());
    /// ```
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// Checks whether the [Bitboard] is full.
    /// 
    /// # Examples
    /// 
    /// ``` 
    /// # use milo_core::Bitboard;
    /// let bb = Bitboard::FULL;
    /// assert!(bb.is_full());
    /// ```
    #[inline(always)]
    pub const fn is_full(&self) -> bool {
        self.0 == !0
    }


}



#[macro_export]
macro_rules! bitboard {
    (
        $a8:tt $b8:tt $c8:tt $d8:tt $e8:tt $f8:tt $g8:tt $h8:tt
        $a7:tt $b7:tt $c7:tt $d7:tt $e7:tt $f7:tt $g7:tt $h7:tt
        $a6:tt $b6:tt $c6:tt $d6:tt $e6:tt $f6:tt $g6:tt $h6:tt
        $a5:tt $b5:tt $c5:tt $d5:tt $e5:tt $f5:tt $g5:tt $h5:tt
        $a4:tt $b4:tt $c4:tt $d4:tt $e4:tt $f4:tt $g4:tt $h4:tt
        $a3:tt $b3:tt $c3:tt $d3:tt $e3:tt $f3:tt $g3:tt $h3:tt
        $a2:tt $b2:tt $c2:tt $d2:tt $e2:tt $f2:tt $g2:tt $h2:tt
        $a1:tt $b1:tt $c1:tt $d1:tt $e1:tt $f1:tt $g1:tt $h1:tt
    ) => {
        $crate::bitboard! { @__inner
            $a8 $b8 $c8 $d8 $e8 $f8 $g8 $h8
            $a7 $b7 $c7 $d7 $e7 $f7 $g7 $h7
            $a6 $b6 $c6 $d6 $e6 $f6 $g6 $h6
            $a5 $b5 $c5 $d5 $e5 $f5 $g5 $h5
            $a4 $b4 $c4 $d4 $e4 $f4 $g4 $h4
            $a3 $b3 $c3 $d3 $e3 $f3 $g3 $h3
            $a2 $b2 $c2 $d2 $e2 $f2 $g2 $h2
            $a1 $b1 $c1 $d1 $e1 $f1 $g1 $h1
        }
    };
    (@__inner $($occupied:tt)*) => {{
        const BITBOARD: $crate::Bitboard = {
            let mut index = 0;
            let mut bitboard = $crate::Bitboard::EMPTY;
            $(
                if $crate::bitboard!(@__square $occupied) {
                    bitboard.0 |= 1 << index;
                }
                index += 1;
            )*
            let _ = index;
            bitboard
        };
        BITBOARD
    }};
    (@__square X) => { true };
    (@__square .) => { false };
    (@__square $token:tt) => {
        compile_error!(
            concat!(
                "Expected only `X` or `.` tokens, found `",
                stringify!($token),
                "`"
            )
        )
    };
    ($($token:tt)*) => {
        compile_error!("Expected 64 squares")
    };
}
pub use bitboard as bitboard;