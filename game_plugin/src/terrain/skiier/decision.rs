use super::SpecialPoint;
use bevy::prelude::*;
use slana::{dijkstra, GraphView, GridCoord, Path};
use std::{
    cmp::{max, min, Reverse},
    collections::{BinaryHeap, HashMap},
};
#[derive(Debug, PartialEq, Eq)]
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
    result: DecisionResult,
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
    /// refrence to previous decision
    previous_decision: Option<usize>,
}
const SEARCH_DEPTH: u32 = 3;

pub fn get_best_decision(
    view: &GraphView<u32, SpecialPoint>,
    start: GridCoord,
    base_cost: u32,
) -> Vec<Box<dyn Decision>> {
    let mut priority = BinaryHeap::new();
    let mut decision_vec = vec![];
    let mut decisions = get_decisions(view, start);
    for (i, decision) in decisions.drain(..).enumerate() {
        let (result, mut cost, path) = decision.get_cost(view, start);
        let end = path.get_end();
        cost += base_cost;
        decision_vec.push(DecisionItem {
            decision,
            previous_decision: None,
        });
        priority.push(Reverse(DecisionNode {
            decision: i,
            end,
            search_depth: 0,
            total_cost: cost,
            result,
        }));
    }
    while let Some(rev_decision) = priority.pop() {
        let decision_node = rev_decision.0;
        if decision_node.search_depth == SEARCH_DEPTH
            || decision_node.result == DecisionResult::Despawn
        {
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
                    previous_decision: Some(decision_node.decision),
                });
                priority.push(Reverse(DecisionNode {
                    decision: index,
                    end,
                    search_depth: decision_node.search_depth + 1,
                    total_cost: decision_node.total_cost + cost,
                    result,
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
        .chain(
            GoToParkingLot::new(view, position)
                .drain(..)
                .map(|d| d as Box<dyn Decision>),
        )
        .chain(
            GoUpLift::new(view, position)
                .drain(..)
                .map(|d| d as Box<dyn Decision>),
        )
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
            .filter(|(_type, position, _index)| *position != start)
            .filter_map(|(point_type, lift_bottom, _index)| match point_type {
                SpecialPoint::LiftBottom => Some(Box::new(Self {
                    lift_bottom: *lift_bottom,
                })),
                _ => None,
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
        let cost = max(path.cost(), 100);
        (DecisionResult::Goto(self.lift_bottom), cost, path)
    }
    fn clone_box(&self) -> Box<dyn Decision> {
        Box::new(self.clone())
    }
}
#[derive(Debug, Clone)]
pub struct GoUpLift {
    lift_bottom: GridCoord,
    lift_top: GridCoord,
}
struct PartialLift {
    lift_bottom: Option<GridCoord>,
    lift_top: Option<GridCoord>,
}
impl GoUpLift {
    pub fn new(view: &GraphView<u32, SpecialPoint>, start: GridCoord) -> Vec<Box<Self>> {
        let mut lifts: HashMap<usize, PartialLift> = HashMap::new();
        view.special_points()
            .iter()
            .filter(|(point_type, _position, _index)| match point_type {
                SpecialPoint::LiftTop => true,
                SpecialPoint::LiftBottom => true,
                _ => false,
            })
            .for_each(|(point_type, position, index)| {
                if let Some(lift) = lifts.get_mut(index) {
                    match point_type {
                        SpecialPoint::LiftTop => lift.lift_top = Some(*position),
                        SpecialPoint::LiftBottom => lift.lift_bottom = Some(*position),
                        _ => error!("invalid point type: {:#?}", point_type),
                    }
                } else {
                    match point_type {
                        SpecialPoint::LiftTop => {
                            lifts.insert(
                                *index,
                                PartialLift {
                                    lift_top: Some(*position),
                                    lift_bottom: None,
                                },
                            );
                        }
                        SpecialPoint::LiftBottom => {
                            lifts.insert(
                                *index,
                                PartialLift {
                                    lift_top: None,
                                    lift_bottom: Some(*position),
                                },
                            );
                        }
                        _ => error!("invalid point type: {:#?}", point_type),
                    }
                }
            });

        lifts
            .iter()
            .map(|(_index, lift)| {
                Box::new(GoUpLift {
                    lift_bottom: lift.lift_bottom.unwrap(),
                    lift_top: lift.lift_top.unwrap(),
                })
            })
            .filter(|lift| lift.lift_bottom == start)
            .collect()
    }
}
impl Decision for GoUpLift {
    fn get_cost(
        &self,
        view: &GraphView<u32, SpecialPoint>,
        start: GridCoord,
    ) -> (DecisionResult, u32, Path) {
        if self.lift_bottom != start {
            error!("invalid state for go to lift bottom, start==end");
        }
        let path = dijkstra(&view, start, self.lift_top);

        (
            DecisionResult::Goto(self.lift_top),
            max(path.cost(), 100),
            path,
        )
    }
    fn clone_box(&self) -> Box<dyn Decision> {
        Box::new(self.clone())
    }
}
#[derive(Debug, Clone)]
pub struct GoToParkingLot {
    position: GridCoord,
}
impl GoToParkingLot {
    pub fn new(view: &GraphView<u32, SpecialPoint>, _start: GridCoord) -> Vec<Box<Self>> {
        view.special_points()
            .iter()
            .filter_map(|(point_type, position, _index)| match point_type {
                SpecialPoint::ParkingLot => Some(Box::new(Self {
                    position: *position,
                })),
                _ => None,
            })
            .collect()
    }
}
impl Decision for GoToParkingLot {
    fn get_cost(
        &self,
        view: &GraphView<u32, SpecialPoint>,
        start: GridCoord,
    ) -> (DecisionResult, u32, Path) {
        let path = dijkstra(&view, start, self.position);
        let cost = min(path.cost(), 1);
        (DecisionResult::Despawn, cost, path)
    }
    fn clone_box(&self) -> Box<dyn Decision> {
        Box::new(self.clone())
    }
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn ordering() {
        let n1 = DecisionNode {
            decision: 0,
            end: GridCoord::from_xy(0, 0),
            total_cost: 1,
            search_depth: 0,
            result: DecisionResult::Goto(GridCoord::from_xy(1, 1)),
        };
        let n2 = DecisionNode {
            decision: 0,
            end: GridCoord::from_xy(0, 0),
            total_cost: 2,
            search_depth: 0,
            result: DecisionResult::Goto(GridCoord::from_xy(1, 1)),
        };
        assert!(n2 > n1);
        assert!(n2 >= n1);
        assert!(n1 < n2);
        assert!(n1 <= n2);
        let new_n1 = DecisionNode {
            decision: 0,
            end: GridCoord::from_xy(0, 0),
            total_cost: 1,
            search_depth: 0,
            result: DecisionResult::Goto(GridCoord::from_xy(1, 1)),
        };
        assert!(n1 == new_n1)
    }
}
