use cfg::function::{Function};
use cfg::dot;
use fxhash::FxHashMap;
use graph::{
    algorithms::{dominators::*, *},
    NodeId, Edge, Graph
};

mod conditional;
mod r#loop;
mod jump;

struct GraphStructurer<'a> {
    function: Function<'a>,
    root: NodeId,
    idoms: &'a FxHashMap<NodeId, NodeId>,
    back_edges: Vec<Edge>,
}

impl<'a> GraphStructurer<'a> {
    fn new(
        function: Function<'a>,
        graph: Graph,
        blocks: FxHashMap<NodeId, ast::Block<'a>>,
        root: NodeId,
        idoms: &'a FxHashMap<NodeId, NodeId>,
    ) -> Self {
        let back_edges = back_edges(&graph, root).unwrap();
        let post_dom_tree = post_dominator_tree(&graph, &dfs_tree(&graph, root));
        let root = function.entry().unwrap();
        Self {
            function,
            root,
            idoms,
            back_edges,
        }
    }

    fn loop_header(&self, mut node: NodeId) -> Option<NodeId> {
        while !self.back_edges.iter().any(|edge| edge.1 == node) {
            if let Some(&idom) = self.idoms.get(&node) {
                node = idom;
            } else {
                return None;
            }
        }
        Some(node)
    }

    fn block_is_no_op(block: &ast::Block) -> bool {
        block
            .iter()
            .filter(|stmt| stmt.as_comment().is_some())
            .count()
            == block.len()
    }

    fn try_match_pattern(&mut self, node: NodeId) {
        let successors = self.function.graph().successors(node);

        if self.try_collapse_loop(node) {
            return;
        }

        match successors.len() {
            0 => { }
            1 => {
                // remove unnecessary jumps to allow pattern matching
                self.match_jump(node, successors[0]);
            }
            2 => {
                let (then_edge, else_edge) = self
                    .function
                    .block(node)
                    .unwrap()
                    .terminator
                    .as_ref()
                    .unwrap()
                    .as_conditional()
                    .unwrap();
                let (then_node, else_node) = (then_edge.node, else_edge.node);
                self.match_conditional(node, then_node, else_node);
            }
            _ => unreachable!(),
        };
        dot::render_to(&self.function, &mut std::io::stdout());
    }

    fn collapse(&mut self) {
        let dfs = dfs_tree(self.function.graph(), self.root);
        for node in self
            .function
            .graph()
            .nodes()
            .iter()
            .filter(|&&node| !dfs.has_node(node))
            .cloned()
            .collect::<Vec<_>>()
        {
            self.function.remove_block(node);
        }

        for node in dfs.post_order(self.root) {
            println!("matching {}", node);
            self.try_match_pattern(node);
        }

        let nodes = self.function.graph().nodes().len();
        if self.function.graph().nodes().len() != 1 {
            println!("failed to collapse! total nodes: {}", nodes);
        }
    }

    fn structure(mut self) -> ast::Block<'a> {
        self.collapse();
        self.function.remove_block(self.root).unwrap().ast
    }
}

pub fn lift(function: cfg::function::Function) {
    let graph = function.graph().clone();
    let root = function.entry().unwrap();
    let dfs = dfs_tree(&graph, root);
    let idoms = compute_immediate_dominators(&graph, root, &dfs);

    //dot::render_to(&graph, &mut std::io::stdout());

    let blocks = function
        .blocks()
        .iter()
        .map(|(&node, block)| (node, block.ast.clone()))
        .collect();

    let structurer = GraphStructurer::new(function, graph, blocks, root, &idoms);
    let block = structurer.structure();
    println!("{}", block);
}
