use super::SpecialPoint;
use bevy::prelude::*;
use slana::{dijkstra, GraphLayer, GraphView, GridCoord, Path};
use std::{
    cmp::{max, Reverse},
    collections::BinaryHeap,
};
#[derive(Debug)]
pub enum DecisionResult {
    /// Go to point
    Goto(GridCoord),
    /// Despawn entity
    Despawn,
}
pub trait Decision: std::fmt::Debug {
    fn get_cost(
        &self,
        view: &GraphView<u32, SpecialPoint>,
        start: GridCoord,
    ) -> (DecisionResult, u32, Path);
    fn clone_box(&self) -> Box<dyn Decision>;
}
#[derive(Debug)]
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
#[derive(Debug)]
struct DecisionItem {
    decision: Box<dyn Decision>,
    cost: u32,
    path: Path,
    search_depth: u32,
    /// refrence to previous decision
    previous_decision: Option<usize>,
}
const SEARCH_DEPTH: u32 = 3;

pub fn get_best_decision(
    view: &GraphView<u32, SpecialPoint>,
    start: GridCoord,
) -> Vec<Box<dyn Decision>> {
    let mut priority = BinaryHeap::new();
    let mut decision_vec = vec![];
    let mut decisions = get_decisions(view, start);
    for (i, decision) in decisions.drain(..).enumerate() {
        let (result, cost, path) = decision.get_cost(view, start);
        let end = path.get_end();
        decision_vec.push(DecisionItem {
            decision,
            path,
            cost,
            previous_decision: None,
            search_depth: 0,
        });
        priority.push(Reverse(DecisionNode {
            decision: i,
            end,
            search_depth: 0,
            total_cost: cost,
        }));
    }
    while let Some(rev_decision) = priority.pop() {
        let decision_node = rev_decision.0;
        info!("processing node: {:#?}", decision_node);
        if decision_node.search_depth == SEARCH_DEPTH {
            for d in decision_vec.iter() {
                info!("decsion vec: {:#?}", d);
            }
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
            let mut decisions = get_decisions(view, decision_node.end);
            for decision in decisions.drain(..) {
                let (result, cost, path) = decision.get_cost(view, decision_node.end);
                let end = path.get_end();
                let index = decision_vec.len();
                decision_vec.push(DecisionItem {
                    decision,
                    path,
                    cost,
                    search_depth: decision_node.search_depth + 1,
                    previous_decision: Some(decision_node.decision),
                });
                priority.push(Reverse(DecisionNode {
                    decision: index,
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
    view: &GraphView<u32, SpecialPoint>,
    position: GridCoord,
) -> Vec<Box<dyn Decision>> {
    GoToLiftBottom::new(view, position)
        .drain(..)
        .map(|d| d as Box<dyn Decision>)
        .collect()
}
#[derive(Clone, Debug)]
pub struct GoToLiftBottom {
    lift_bottom: GridCoord,
}
impl GoToLiftBottom {
    pub fn new(view: &GraphView<u32, SpecialPoint>, start: GridCoord) -> Vec<Box<Self>> {
        view.special_points()
            .iter()
            .filter(|(_type, position)| *position != start)
            .filter_map(|(point_type, lift_bottom)| match point_type {
                SpecialPoint::LiftBottom => Some(Box::new(Self {
                    lift_bottom: *lift_bottom,
                })),
                SpecialPoint::LiftTop => None,
            })
            .collect()
    }
}
impl Decision for GoToLiftBottom {
    fn get_cost(
        &self,
        view: &GraphView<u32, SpecialPoint>,
        start: GridCoord,
    ) -> (DecisionResult, u32, Path) {
        if self.lift_bottom == start {
            error!("invalid state for go to lift bottom, start==end");
        }
        let path = dijkstra(&view, start, self.lift_bottom);
        let cost = max(path.cost(), 1);
        (DecisionResult::Goto(self.lift_bottom), cost, path)
    }
    fn clone_box(&self) -> Box<dyn Decision> {
        Box::new(self.clone())
    }
}
