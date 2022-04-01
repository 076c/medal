use enum_dispatch::enum_dispatch;
use graph::NodeId;

// A trait implemented for branching instructions
#[enum_dispatch]
pub(crate) trait BranchInfo {
    // Returns the branches the instruction can take
    fn branches(&self) -> Box<[NodeId]>;

    // Returns the branches the instruction can take
    fn branches_mut(&mut self) -> Box<[&mut NodeId]>;

    // Replaces a branch to `old` with `new`
    // Caller is responsible for correctness!
    fn replace_branch(&mut self, old: usize, new: usize) {
        for value in self.branches_mut().iter_mut() {
            if **value == old {
                **value = new;
            }
        }
    }
}
