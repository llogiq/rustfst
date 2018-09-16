use arc::Arc;
use semirings::Semiring;
use Label;
use StateId;

pub trait Fst<W: Semiring> {
    type Iter: Iterator<Item = Arc<W>>;
    //type Symtab: IntoIterator<Item=String>;
    fn start(&self) -> Option<StateId>;
    fn final_weight(&self, &StateId) -> Option<W>;
    fn arc_iter(&self, &StateId) -> Self::Iter;
    //fn get_isyms(&self) -> Option<Self::Symtab>;
    //fn get_osyms(&self) -> Option<Self::Symtab>;
    fn is_final(&self, &StateId) -> bool;
    fn num_arcs(&self) -> usize;
}

pub trait MutableFst<W: Semiring>: Fst<W> {
    fn new() -> Self;
    fn set_start(&mut self, &StateId);
    fn add_state(&mut self) -> StateId;
    fn del_state(&mut self, &StateId);
    fn del_states<T: IntoIterator<Item = StateId>>(&mut self, states: T);
    fn add_arc(
        &mut self,
        source: &StateId,
        target: &StateId,
        ilabel: Label,
        olabel: Label,
        weight: W,
    );
    fn set_final(&mut self, id: &StateId, finalweight: W);
    // fn set_isyms<T: IntoIterator<Item=String>>(&mut self, symtab: T);
    // fn set_osyms<T: IntoIterator<Item=String>>(&mut self, symtab: T);
}

pub trait ExpandedFst<W: Semiring>: Fst<W> {
    fn num_states(&self) -> usize;
}
