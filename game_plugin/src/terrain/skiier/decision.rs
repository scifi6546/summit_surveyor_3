use super::SpecialPoint;
use slana::{GraphLayer, GridCoord, Path};
use std::{cmp::Reverse, collections::BinaryHeap};
pub enum DecisionResult {
    /// Go to point
    Goto(GridCoord),
    /// Despawn entity
    Despawn,
}
pub trait Decision {
    fn get_cost(
        &self,
        layers: &[&dyn GraphLayer<u32, SpecialPoint = SpecialPoint>],
        start: GridCoord,
    ) -> (DecisionResult, u32, Path);
    fn clone_box(&self) -> Box<dyn Decision>;
}

struct DecisionNode {
    /// refrence to decision in node
    decision: usize,
    end: GridCoord,
    total_cost: u32,
    search_depth: u32,
}

impl std::cmp::Ord for DecisionNode {
    fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
        self.total_cost.cmp(&rhs.total_cost)
    }
}
impl std::cmp::PartialOrd for DecisionNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl std::cmp::Eq for DecisionNode {}
impl std::cmp::PartialEq for DecisionNode {
    fn eq(&self, right: &Self) -> bool {
        self.total_cost.eq(&right.total_cost)
    }
}
struct DecisionItem {
    decision: Box<dyn Decision>,
    cost: u32,
    path: Path,
    /// refrence to previous decision
    previous_decision: Option<usize>,
}
const SEARCH_DEPTH: u32 = 3;

pub fn get_best_decision(
    layers: &[&dyn GraphLayer<u32, SpecialPoint = SpecialPoint>],
    start: GridCoord,
) -> Vec<Box<dyn Decision>> {
    let mut priority = BinaryHeap::new();
    let mut decision_vec = vec![];
    let mut decisions = get_decisions(layers);
    for (i, decision) in decisions.drain(..).enumerate() {
        let (result, cost, path) = decision.get_cost(layers, start);
        let end = path.get_end();
        decision_vec.push(DecisionItem {
            decision,
            path,
            cost,
            previous_decision: None,
        });
        priority.push(Reverse(DecisionNode {
            decision: i,
            end,
            search_depth: 1,
            total_cost: cost,
        }));
    }
    while let Some(rev_decision) = priority.pop() {
        let decision_node = rev_decision.0;
        if decision_node.search_depth == SEARCH_DEPTH {
            let mut out = vec![];
            let mut current_idx = decision_node.decision;
            loop {
                out.push(decision_vec[current_idx].decision.clone_box());
                if let Some(idx) = decision_vec[current_idx].previous_decision {
                    current_idx = idx;
                } else {
                    out.reverse();
                    return out;
                }
            }
        } else {
            let mut decisions = get_decisions(layers);
            for (i, decision) in decisions.drain(..).enumerate() {
                let (result, cost, path) = decision.get_cost(layers, decision_node.end);
                let end = path.get_end();
                decision_vec.push(DecisionItem {
                    decision,
                    path,
                    cost,
                    previous_decision: Some(decision_node.decision),
                });
                priority.push(Reverse(DecisionNode {
                    decision: i,
                    end,
                    search_depth: decision_node.search_depth + 1,
                    total_cost: decision_node.total_cost + cost,
                }));
            }
        }
    }
    panic!("should never reach this point")
}

pub fn get_decisions(
    layers: &[&dyn GraphLayer<u32, SpecialPoint = SpecialPoint>],
) -> Vec<Box<dyn Decision>> {
    GoToLiftBottom::new(layers)
        .drain(..)
        .map(|d| d as Box<dyn Decision>)
        .collect()
}
#[derive(Clone)]
pub struct GoToLiftBottom {
    lift_bottom: GridCoord,
}
impl GoToLiftBottom {
    pub fn new(layers: &[&dyn GraphLayer<u32, SpecialPoint = SpecialPoint>]) -> Vec<Box<Self>> {
        todo!()
    }
}
impl Decision for GoToLiftBottom {
    fn get_cost(
        &self,
        layers: &[&dyn GraphLayer<u32, SpecialPoint = SpecialPoint>],
        start: GridCoord,
    ) -> (DecisionResult, u32, Path) {
        todo!()
    }
    fn clone_box(&self) -> Box<dyn Decision> {
        Box::new(self.clone())
    }
}
