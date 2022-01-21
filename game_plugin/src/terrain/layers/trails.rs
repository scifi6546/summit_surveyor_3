use super::{SpecialPoint, TerrainPoint};
use bevy::prelude::*;
use generational_arena::{Arena, Index as ArenaIndex};
use nalgebra::Vector2;
use slana::{GraphLayer, GridCoord};
use std::collections::BTreeSet;
/// todo change to better type
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TrailRef {
    index: ArenaIndex,
}
#[derive(Clone)]
pub enum TrailSegment {
    Segment {
        radius: f32,
    },
    Connection {
        /// segment that the trail  isconnected to
        connected: TrailRef,
        /// how far down the trail is the segment connected
        connected_distance: f32,
        radius: f32,
    },
}
#[derive(Clone)]
struct TrailNode {
    /// coordinates of grid
    coord: GridCoord,
    /// trails that refrence this node
    references: Vec<TrailRef>,
    /// segment of trail
    segment: TrailSegment,
}
impl TrailSegment {
    pub fn approx_eq(&self, other: &Self) -> bool {
        todo!("approx eq")
    }
}
#[derive(Debug, Clone)]
pub enum TrailError {
    RefDoesNotExist(TrailRef),
    InvalidSegmentType,
    ToFarDownTrail,
}
/// collection of trails
#[derive(Component)]
pub struct TrailCollection {
    arena: Arena<TrailNode>,
}
pub struct PointIter<'a> {
    iter: generational_arena::Iter<'a, TrailNode>,
}
impl<'a> std::iter::Iterator for PointIter<'a> {
    type Item = (GridCoord, f32);
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((_idx, n)) = self.iter.next() {
            Some((
                n.coord,
                match n.segment {
                    TrailSegment::Connection { radius, .. } => radius,
                    TrailSegment::Segment { radius, .. } => radius,
                },
            ))
        } else {
            None
        }
    }
}
pub struct PathIter<'a> {
    current_node: Option<(&'a TrailNode, TrailRef)>,
    ref_index: usize,
    iter: generational_arena::Iter<'a, TrailNode>,
    visited: BTreeSet<(TrailRef, TrailRef)>,
    collection: &'a TrailCollection,
}
impl<'a> std::iter::Iterator for PathIter<'a> {
    type Item = (Vector2<f32>, Vector2<f32>);
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((node, trail_ref)) = self.current_node {
            if self.ref_index < node.references.len() {
                let end_node = self
                    .collection
                    .arena
                    .get(node.references[self.ref_index].index)
                    .expect("failed to get node");
                if !self
                    .visited
                    .contains(&(trail_ref, node.references[self.ref_index]))
                {
                    if match node.segment {
                        TrailSegment::Connection { .. } => false,
                        _ => true,
                    } {
                        self.visited
                            .insert((trail_ref, node.references[self.ref_index]));
                        self.visited
                            .insert((node.references[self.ref_index], trail_ref));
                        self.ref_index += 1;
                        let (x1, y1) = node.coord.to_xy();
                        let (x2, y2) = end_node.coord.to_xy();
                        Some((
                            Vector2::new(x1 as f32, y1 as f32),
                            Vector2::new(x2 as f32, y2 as f32),
                        ))
                    } else {
                        todo!("connection")
                    }
                } else {
                    self.ref_index += 1;
                    return self.next();
                }
            } else {
                if let Some((index, next)) = self.iter.next() {
                    self.current_node = Some((next, TrailRef { index }));
                    self.ref_index = 0;
                    return self.next();
                } else {
                    None
                }
            }
        } else {
            if let Some((index, next)) = self.iter.next() {
                self.current_node = Some((next, TrailRef { index }));
                self.ref_index = 0;
                return self.next();
            } else {
                None
            }
        }
    }
}
impl TrailCollection {
    /// adds small set off trails to map
    pub fn add_trail(
        &mut self,
        start: GridCoord,
        start_radius: f32,
        end: GridCoord,
        end_radius: f32,
    ) -> (TrailRef, TrailRef) {
        let start_segment = TrailSegment::Segment {
            radius: start_radius,
        };
        let start_index = TrailRef {
            index: self.arena.insert(TrailNode {
                coord: start,
                references: vec![],
                segment: start_segment,
            }),
        };
        let end_segment = TrailSegment::Segment { radius: end_radius };
        let end_index = TrailRef {
            index: self.arena.insert(TrailNode {
                coord: end,
                references: vec![start_index],
                segment: end_segment,
            }),
        };
        self.arena
            .get_mut(start_index.index)
            .unwrap()
            .references
            .push(end_index);
        (start_index, end_index)
    }
    /// connect trail to another trail
    pub fn connect_trail(
        &mut self,
        src_segment: TrailRef,
        connected_to: TrailRef,
        distance: f32,
        radius: f32,
    ) -> Result<TrailRef, TrailError> {
        let end_segment_coord = if let Some(s) = self.arena.get(connected_to.clone().index) {
            s.coord.clone()
        } else {
            return Err(TrailError::RefDoesNotExist(connected_to));
        };
        let connection = TrailSegment::Connection {
            connected_distance: distance,
            connected: connected_to.clone(),
            radius,
        };
        let trail_connection = TrailRef {
            index: self.arena.insert(TrailNode {
                coord: end_segment_coord,
                references: vec![connected_to, src_segment.clone()],
                segment: connection,
            }),
        };
        let start_segment = if let Some(s) = self.arena.get_mut(src_segment.index) {
            s
        } else {
            return Err(TrailError::RefDoesNotExist(src_segment));
        };
        start_segment.references.push(trail_connection);
        Ok(trail_connection)
    }
    pub fn append_trail(
        &mut self,
        src: TrailRef,
        coord: GridCoord,
        radius: f32,
    ) -> Result<TrailRef, TrailError> {
        let new_node = TrailRef {
            index: self.arena.insert(TrailNode {
                coord,
                references: vec![src],
                segment: TrailSegment::Segment { radius },
            }),
        };
        if let Some(src_node) = self.arena.get_mut(src.index) {
            src_node.references.push(new_node);
            Ok(new_node)
        } else {
            todo!("error handeling")
        }
    }
    pub fn iter_trails(&self) -> PointIter<'_> {
        PointIter {
            iter: self.arena.iter(),
        }
    }
    /// Iterates through all paths in the trail collection.
    pub fn iter_paths(&self) -> PathIter<'_> {
        PathIter {
            current_node: None,
            iter: self.arena.iter(),
            visited: BTreeSet::new(),
            ref_index: 0,
            collection: &self,
        }
    }
}
impl GraphLayer<TerrainPoint> for TrailCollection {
    type SpecialPoint = SpecialPoint;
    fn get_special_pooints(&self) -> Vec<(Self::SpecialPoint, GridCoord)> {
        self.arena
            .iter()
            .map(|(_idx, node)| (SpecialPoint::Trail, node.coord))
            .collect()
    }
    fn get_children(&self, coord: GridCoord) -> Vec<(GridCoord, TerrainPoint)> {
        if let Some((_index, node)) = self
            .arena
            .iter()
            .filter(|(_idx, node)| node.coord == coord)
            .next()
        {
            node.references
                .iter()
                .map(|trail_ref| {
                    self.arena
                        .get(trail_ref.index)
                        .expect("invalid refrence")
                        .coord
                })
                .map(|coord| (coord, TerrainPoint::Trail))
                .collect()
        } else {
            vec![]
        }
    }
    /// get specific point
    fn get_node(&self, coord: GridCoord) -> Option<TerrainPoint> {
        if self
            .arena
            .iter()
            .filter(|(_idx, node)| node.coord == coord)
            .next()
            .is_some()
        {
            Some(TerrainPoint::Trail)
        } else {
            None
        }
    }
}
impl Default for TrailCollection {
    fn default() -> Self {
        Self {
            arena: Arena::new(),
        }
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use slana::GridCoord;
    #[test]
    fn make_collection() {
        let _c = TrailCollection::default();
    }
    #[test]
    fn add_trail() {
        let mut c = TrailCollection::default();
        let (_s, _e) = c.add_trail(GridCoord::from_xy(0, 0), 1.0, GridCoord::from_xy(0, 1), 1.0);
    }
    #[test]
    fn append_trail() {
        let mut c = TrailCollection::default();
        let (_s, end) = c.add_trail(GridCoord::from_xy(0, 0), 1.0, GridCoord::from_xy(0, 1), 1.0);
        c.append_trail(end, GridCoord::from_xy(0, 2), 1.0)
            .expect("should be successfull");
    }
    #[test]
    fn connect_trail() {
        let mut c = TrailCollection::default();
        let (start, _e) = c.add_trail(GridCoord::from_xy(0, 0), 1.0, GridCoord::from_xy(0, 2), 1.0);
        let (start_2, _end2) =
            c.add_trail(GridCoord::from_xy(1, 1), 1.0, GridCoord::from_xy(1, 2), 1.0);
        c.connect_trail(start_2, start, 1.0, 1.0);
    }
    #[test]
    fn get_children() {
        let mut c = TrailCollection::default();
        let (start, end) =
            c.add_trail(GridCoord::from_xy(0, 0), 1.0, GridCoord::from_xy(0, 1), 1.0);
        assert_eq!(
            c.get_children(GridCoord::from_xy(0, 0)),
            vec![(GridCoord::from_xy(0, 1), TerrainPoint::Trail)]
        );
        assert_eq!(
            c.get_children(GridCoord::from_xy(0, 1)),
            vec![(GridCoord::from_xy(0, 0), TerrainPoint::Trail)]
        );
    }
}
