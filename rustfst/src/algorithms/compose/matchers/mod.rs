use std::fmt::Debug;

use anyhow::Result;

use bitflags::bitflags;
pub use generic_matcher::GenericMatcher;
pub use multi_eps_matcher::{MultiEpsMatcher, MultiEpsMatcherFlags};
pub use sorted_matcher::SortedMatcher;

use crate::fst_traits::Fst;
use crate::semirings::Semiring;
use crate::{Label, StateId};
use crate::{Tr, EPS_LABEL, NO_LABEL};
use std::sync::Arc;

mod generic_matcher;
mod multi_eps_matcher;
mod sorted_matcher;

bitflags! {
    pub struct MatcherFlags: u32 {
        const REQUIRE_MATCH =  1u32;
        const INPUT_LOOKAHEAD_MATCHER =  1u32 << 4;
        const OUTPUT_LOOKAHEAD_MATCHER =  1u32 << 5;
        const LOOKAHEAD_WEIGHT =  1u32 << 6;
        const LOOKAHEAD_PREFIX =  1u32 << 7;
        const LOOKAHEAD_NON_EPSILONS =  1u32 << 8;
        const LOOKAHEAD_EPSILONS =  1u32 << 9;
        const LOOKAHEAD_NON_EPSILON_PREFIX =  1u32 << 10;

        const LOOKAHEAD_FLAGS = Self::INPUT_LOOKAHEAD_MATCHER.bits |
            Self::OUTPUT_LOOKAHEAD_MATCHER.bits |
            Self::LOOKAHEAD_WEIGHT.bits |
            Self::LOOKAHEAD_PREFIX.bits |
            Self::LOOKAHEAD_NON_EPSILONS.bits |
            Self::LOOKAHEAD_EPSILONS.bits |
            Self::LOOKAHEAD_NON_EPSILON_PREFIX.bits;

        const ILABEL_LOOKAHEAD_FLAGS = Self::INPUT_LOOKAHEAD_MATCHER.bits |
            Self::LOOKAHEAD_WEIGHT.bits |
            Self::LOOKAHEAD_PREFIX.bits |
            Self::LOOKAHEAD_EPSILONS.bits |
            Self::LOOKAHEAD_NON_EPSILON_PREFIX.bits;

        const OLABEL_LOOKAHEAD_FLAGS = Self::OUTPUT_LOOKAHEAD_MATCHER.bits |
            Self::LOOKAHEAD_WEIGHT.bits |
            Self::LOOKAHEAD_PREFIX.bits |
            Self::LOOKAHEAD_EPSILONS.bits |
            Self::LOOKAHEAD_NON_EPSILON_PREFIX.bits;
    }
}

pub static REQUIRE_PRIORITY: usize = std::usize::MAX;

#[derive(Copy, Debug, PartialOrd, PartialEq, Clone)]
/// Specifies matcher action
pub enum MatchType {
    /// Match input label
    MatchInput,
    /// Match output label
    MatchOutput,
    /// Match input or output label
    MatchBoth,
    /// Match anything
    MatchNone,
    /// Otherwise, match unknown
    MatchUnknown,
}

// Use this to avoid autoref
#[derive(Clone)]
pub enum IterItemMatcher<W: Semiring> {
    Tr(Tr<W>),
    EpsLoop,
}

impl<W: Semiring> IterItemMatcher<W> {
    pub fn into_tr(self, state: StateId, match_type: MatchType) -> Result<Tr<W>> {
        match self {
            IterItemMatcher::Tr(tr) => Ok(tr),
            IterItemMatcher::EpsLoop => eps_loop(state, match_type),
        }
    }
}

pub fn eps_loop<W: Semiring>(state: StateId, match_type: MatchType) -> Result<Tr<W>> {
    let tr = match match_type {
        MatchType::MatchInput => Tr::new(NO_LABEL, EPS_LABEL, W::one(), state),
        MatchType::MatchOutput => Tr::new(EPS_LABEL, NO_LABEL, W::one(), state),
        _ => bail!("Unsupported match_type : {:?}", match_type),
    };
    Ok(tr)
}

/// Matchers find and iterate through requested labels at FST states. In the
/// simplest form, these are just some associative map or search keyed on labels.
/// More generally, they may implement matching special labels that represent
/// sets of labels such as sigma (all), rho (rest), or phi (fail).
pub trait Matcher<W: Semiring>: Debug {
    type Iter: MatcherIterator<W, Item = IterItemMatcher<W>> + Sized;

    fn new(fst: &impl Fst<W>, match_type: MatchType) -> Result<Self>
    where
        Self: std::marker::Sized;
    fn iter(&self, fst: &impl Fst<W>, state: StateId, label: Label) -> Result<Self::Iter>;
    fn final_weight(&self, fst: &impl Fst<W>, state: StateId) -> Result<Option<W>>;
    fn match_type(&self, fst: &impl Fst<W>, test: bool) -> Result<MatchType>;
    fn flags(&self, fst: &impl Fst<W>) -> MatcherFlags;

    /// Indicates preference for being the side used for matching in
    /// composition. If the value is kRequirePriority, then it is
    /// mandatory that it be used. Calling this method without passing the
    /// current state of the matcher invalidates the state of the matcher.
    fn priority(&self, state: StateId) -> Result<usize>;
}

pub trait MatcherIterator<W: Semiring> : Sized {
    type Item;
    fn next(&mut self, fst: &impl Fst<W>) -> Option<Self::Item>;
    fn peekable(self) -> MatcherPeekable<W, Self> {
        MatcherPeekable {
            next: None,
            iter: self,
        }
    }
}

pub struct MatcherPeekable<W: Semiring, I: MatcherIterator<W>> {
    iter: I,
    next: Option<I::Item>,
}

impl<W: Semiring, I: MatcherIterator<W>> MatcherIterator<W> for MatcherPeekable<W, I> {
    type Item = I::Item;

    fn next(&mut self, fst: &impl Fst<W>) -> Option<Self::Item> {
        self.next.take().or_else(|| self.iter.next(fst))
    }

}

impl<W: Semiring, I: MatcherIterator<W>> MatcherPeekable<W, I> {
    pub fn peek(&mut self, fst: &impl Fst<W>) -> Option<&<Self as MatcherIterator<W>>::Item> {
        if self.next.is_none() {
            self.next = self.iter.next(fst)
        }
        self.next.as_ref()
    }
}
