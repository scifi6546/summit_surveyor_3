use std::{
    collections::{BinaryHeap, HashMap},
    marker::PhantomData,
};
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub struct GridCoord(i64);
impl GridCoord {
    pub fn to_xy(&self) -> (i32, i32) {
        let x = self.0 >> 32;
        let y = (0x00_00_00_00_ff_ff_ff_ff) & self.0;
        (x as i32, y as i32)
    }
    pub fn from_xy(x: i32, y: i32) -> Self {
        Self(((x as i64) << 32) + y as i64)
    }
}
impl std::fmt::Debug for GridCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (x, y) = self.to_xy();
        f.debug_struct("GridCoord")
            .field("x", &x)
            .field("y", &y)
            .finish()
    }
}
pub trait GraphLayer<T> {
    /// type specifing special point
    type SpecialPoint;
    /// gets special points at coord
    fn get_special_pooints(&self) -> Vec<(Self::SpecialPoint, GridCoord)>;
    fn get_children(&self, coord: GridCoord) -> Vec<(GridCoord, T)>;
}
pub struct Grid<T: std::clone::Clone, SpecialType> {
    special_marker: PhantomData<SpecialType>,
    data: Vec<T>,
    dim_x: usize,
    dim_y: usize,
}
impl<T: std::clone::Clone, S> Grid<T, S> {
    /// Creates grid based off of initial value
    pub fn from_val(size: (u32, u32), val: T) -> Self {
        Self {
            special_marker: PhantomData,
            data: vec![val; size.0 as usize * size.1 as usize],
            dim_x: size.0 as usize,
            dim_y: size.1 as usize,
        }
    }
    pub fn from_fn<F: Fn(i32, i32) -> T>(size: (u32, u32), ctor: F) -> Self {
        let mut data = vec![];
        data.reserve(size.0 as usize * size.1 as usize);
        for x in 0..size.0 {
            for y in 0..size.1 {
                data.push(ctor(x as i32, y as i32));
            }
        }
        Self {
            special_marker: PhantomData,
            data,
            dim_x: size.0 as usize,
            dim_y: size.1 as usize,
        }
    }
    /// Returns dimensions of grid
    pub fn size(&self) -> (usize, usize) {
        (self.dim_x, self.dim_y)
    }
    pub fn get(&self, x: i32, y: i32) -> &T {
        &self.data[self.get_dim(x as usize, y as usize)]
    }
    /// Gets index in array
    fn get_dim(&self, x: usize, y: usize) -> usize {
        x * self.dim_y + y
    }
}
pub trait ToF32 {
    fn to_f32(&self) -> f32;
}

impl ToF32 for u32 {
    fn to_f32(&self) -> f32 {
        *self as f32
    }
}
impl<T: std::clone::Clone + ToF32, S> Grid<T, S> {
    /// Interpolate to floating point value
    /// link https://en.wikipedia.org/wiki/Bilinear_interpolation
    pub fn interpolate(&self, x: f32, y: f32) -> f32 {
        let x_0 = x.floor() as i32;
        let y_0 = y.floor() as i32;
        let x = x - x.floor();
        let y = y - y.floor();
        if y < -1.0 * x + 1.0 {
            // in bottom triangle

            let f_x0y0 = self.get(x_0, y_0).to_f32();
            let f_x0y1 = self.get(x_0, y_0 + 1).to_f32();
            let f_x1y0 = self.get(x_0 + 1, y_0).to_f32();

            let bottom_line = (f_x1y0 - f_x0y0) * x / (1.0) + f_x0y0;
            let diagonal = (f_x1y0 - f_x0y1) * x / 1.0 + f_x1y0;
            (diagonal - bottom_line) * y / (1.0 - x) + bottom_line
        } else {
            let f_x0y0 = self.get(x_0, y_0).to_f32();
            let f_x0y1 = self.get(x_0, y_0 + 1).to_f32();
            let f_x1y0 = self.get(x_0 + 1, y_0).to_f32();
            let f_x1y1 = self.get(x_0 + 1, y_0 + 1).to_f32();

            let diagonal = (f_x1y0 - f_x0y1) * x / 1.0 + f_x1y0;
            let top_line = (f_x1y1 - f_x0y1) * x / 1.0 + f_x0y1;
            (top_line - diagonal) * y / x + diagonal
        }
    }
}
impl<T: std::clone::Clone, S> GraphLayer<T> for Grid<T, S> {
    type SpecialPoint = S;
    fn get_special_pooints(&self) -> Vec<(Self::SpecialPoint, GridCoord)> {
        vec![]
    }
    fn get_children(&self, coord: GridCoord) -> Vec<(GridCoord, T)> {
        let (x, y) = coord.to_xy();
        let (size_x, size_y) = self.size();
        let mut out = vec![];
        out.reserve(4);
        if x - 1 >= 0 && x - 1 < size_x as i32 && y >= 0 && y < size_y as i32 {
            out.push((GridCoord::from_xy(x - 1, y), self.get(x - 1, y).clone()));
        }
        if x + 1 >= 0 && x + 1 < size_x as i32 && y >= 0 && y < size_y as i32 {
            out.push((GridCoord::from_xy(x + 1, y), self.get(x + 1, y).clone()));
        }
        if x >= 0 && x < size_x as i32 && y - 1 >= 0 && y - 1 < size_y as i32 {
            out.push((GridCoord::from_xy(x, y - 1), self.get(x, y - 1).clone()));
        }
        if x >= 0 && x < size_x as i32 && y + 1 >= 0 && y + 1 < size_y as i32 {
            out.push((GridCoord::from_xy(x, y + 1), self.get(x, y + 1).clone()));
        }

        return out;
    }
}

#[derive(PartialEq, Debug)]
pub struct Path {
    pub total_cost: u32,
    pub points: Vec<GridCoord>,
}
impl Path {
    pub fn cost(&self) -> u32 {
        self.total_cost
    }
    pub fn get_end(&self) -> GridCoord {
        self.points[self.points.len() - 1]
    }
    /// appends path onto end of self
    pub fn append(&mut self, mut other: Self) {
        self.total_cost += other.total_cost;
        for item in other.points.drain(..) {
            let last = self.points[self.points.len() - 1];
            if last != item {
                self.points.push(item)
            }
        }
    }
}
pub struct GraphView<'a, T, S> {
    layers: Vec<&'a dyn GraphLayer<T, SpecialPoint = S>>,
}
impl<'a, T, S> From<Vec<&'a dyn GraphLayer<T, SpecialPoint = S>>> for GraphView<'a, T, S> {
    fn from(layers: Vec<&'a dyn GraphLayer<T, SpecialPoint = S>>) -> GraphView<T, S> {
        Self { layers }
    }
}
impl<'a, T, S> GraphView<'a, T, S> {
    pub fn get_children(&self, coord: GridCoord) -> Vec<(GridCoord, T)> {
        self.layers
            .iter()
            .flat_map(|l| l.get_children(coord))
            .collect()
    }
    pub fn special_points(&self) -> Vec<(S, GridCoord, usize)> {
        self.layers
            .iter()
            .enumerate()
            .flat_map(|(i, l)| {
                l.get_special_pooints()
                    .drain(..)
                    .map(|(p, coord)| (p, coord, i))
                    .collect::<Vec<(S, GridCoord, usize)>>()
            })
            .collect()
    }
}
#[derive(PartialEq, Eq)]
pub struct State {
    cost: u32,
    position: GridCoord,
    previous: Option<GridCoord>,
}
impl std::cmp::Ord for State {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.position.cmp(&other.position))
    }
}
impl std::cmp::PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
/// Gets lowest cost path from start to end
pub fn dijkstra<S>(graph: &GraphView<u32, S>, start: GridCoord, end: GridCoord) -> Path {
    let mut dist: HashMap<GridCoord, (u32, Option<GridCoord>)> = HashMap::new();
    dist.insert(start, (0u32, None));
    let mut heap = BinaryHeap::new();
    heap.push(State {
        cost: 0,
        position: start,
        previous: None,
    });
    while let Some(State {
        cost,
        position,
        mut previous,
    }) = heap.pop()
    {
        if position == end {
            let mut points = vec![position];
            loop {
                if previous.is_some() {
                    if previous.unwrap() == start {
                        break;
                    }
                    points.push(previous.unwrap());
                    previous = dist.get(&previous.unwrap()).unwrap().1;
                } else {
                    break;
                }
            }
            let points = points.drain(..).rev().collect();

            return Path {
                total_cost: cost,
                points,
            };
        }
        for (child_position, child_cost) in graph.get_children(position).iter() {
            let next = State {
                cost: cost + *child_cost,
                position: *child_position,
                previous: Some(position),
            };
            let old_cost = if let Some((c, _)) = dist.get(child_position) {
                *c
            } else {
                u32::MAX
            };
            if next.cost < old_cost {
                dist.insert(*child_position, (next.cost, Some(position)));
                heap.push(next);
            }
        }
    }

    todo!("can not get to end from start")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new() {
        let grid: Grid<usize, u8> = Grid::from_val((100, 100), 0usize);
        assert_eq!(*grid.get(10, 10), 0);
    }
    #[test]
    fn grid_point() {
        let point = GridCoord::from_xy(10, 4);
        assert_eq!(point.to_xy(), (10, 4));
    }
    #[test]
    fn find_null_path() {
        let grid: Grid<u32, u32> = Grid::from_val((100, 100), 1u32);
        let path = dijkstra(
            &vec![&grid as &dyn GraphLayer<u32, SpecialPoint = _>].into(),
            GridCoord::from_xy(0, 0),
            GridCoord::from_xy(0, 0),
        );
        assert_eq!(
            path,
            Path {
                total_cost: 0,
                points: vec![GridCoord::from_xy(0, 0)]
            }
        );
    }
    #[test]
    fn grid_children() {
        let grid: Grid<u32, u32> = Grid::from_val((100, 100), 1u32);
        let mut center_children = grid.get_children(GridCoord::from_xy(1, 1));
        center_children.sort();
        let mut test_data = vec![
            (GridCoord::from_xy(1, 0), 1),
            (GridCoord::from_xy(0, 1), 1),
            (GridCoord::from_xy(2, 1), 1),
            (GridCoord::from_xy(1, 2), 1),
        ];
        test_data.sort();
        assert_eq!(center_children, test_data);
        let mut edge = grid.get_children(GridCoord::from_xy(0, 1));
        edge.sort();
        let mut test_data = vec![
            (GridCoord::from_xy(0, 0), 1),
            (GridCoord::from_xy(0, 2), 1),
            (GridCoord::from_xy(1, 1), 1),
        ];
        test_data.sort();
        assert_eq!(edge, test_data);
    }
    #[test]
    fn find_two_path() {
        let grid: Grid<u32, u32> = Grid::from_val((100, 100), 1u32);
        let path = dijkstra(
            &vec![&grid as &dyn GraphLayer<u32, SpecialPoint = u32>].into(),
            GridCoord::from_xy(0, 0),
            GridCoord::from_xy(2, 0),
        );
        assert_eq!(
            path,
            Path {
                total_cost: 2,
                points: vec![GridCoord::from_xy(1, 0), GridCoord::from_xy(2, 0)]
            }
        );
    }
}
