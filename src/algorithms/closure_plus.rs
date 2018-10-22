use arc::Arc;
use fst_traits::{CoreFst, ExpandedFst, FinalStatesIterator, MutableFst};
use semirings::Semiring;
use Result;
use EPS_LABEL;

/// This operation computes the concatenative closure.
/// If A transduces string `x` to `y` with weight `a`,
/// then the closure transduces `x` to `y` with weight `a`,
/// `xx` to `yy` with weight `a ⊗ a`, `xxx` to `yyy` with weight `a ⊗ a ⊗ a`, etc.
pub fn closure_plus<F>(fst: &mut F) -> Result<()>
where
    F: ExpandedFst + MutableFst,
{
    // Add an epsilon arc from each final states to the start state
    if let Some(start_state) = fst.start() {
        let final_states_id: Vec<_> = fst.final_states_iter().map(|u| u.state_id).collect();
        for final_state_id in final_states_id {
            fst.add_arc(
                &final_state_id,
                Arc::new(EPS_LABEL, EPS_LABEL, <F as CoreFst>::W::one(), start_state),
            )?;
        }
    }
    Ok(())
}