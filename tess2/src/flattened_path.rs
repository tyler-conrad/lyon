use path::builder::*;
use math::*;

use std::ops::Range;
use std::mem;

#[derive(Clone, Debug)]
struct SubPathInfo {
    range: Range<usize>,
    is_closed: bool,
}

/// A path data structure for pre-flattened paths and polygons.
#[derive(Clone)]
pub struct FlattenedPath {
    points: Vec<Point>,
    sub_paths: Vec<SubPathInfo>,
}

impl FlattenedPath {
    /// Creates an empty path.
    pub fn new() -> Self {
        FlattenedPath {
            points: Vec::new(),
            sub_paths: Vec::new(),
        }
    }

    /// Creates a builder for flattened paths.
    pub fn builder() -> Builder {
        Builder::new()
    }

    /// Returns whether the path is empty.
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Returns a slice of all the points in the path.
    pub fn points(&self) -> &[Point] {
        &self.points
    }

    /// Returns an iterator over the sub-paths.
    pub fn sub_paths(&self) -> SubPaths {
        SubPaths {
            points: &self.points,
            sub_paths: &self.sub_paths,
        }
    }

    /// Returns the nth sub-paths.
    pub fn sub_path(&self, index: usize) -> SubPath {
        SubPath {
            points: &self.points[self.sub_paths[index].range.clone()],
            is_closed: self.sub_paths[index].is_closed,
        }
    }

    /// The number of sub-paths.
    pub fn num_sub_paths(&self) -> usize {
        self.sub_paths.len()
    }
}

/// An iterator of the sub paths of a flattened path.
pub struct SubPaths<'l> {
    points: &'l[Point],
    sub_paths: &'l[SubPathInfo],
}

impl<'l> SubPaths<'l> {
    pub fn all_points(&self) -> &[Point] {
        &self.points[self.sub_paths[0].range.clone()]
    }

    pub fn sub_path(&self, index: usize) -> SubPath<'l> {
        SubPath {
            points: &self.points[self.sub_paths[index].range.clone()],
            is_closed: self.sub_paths[index].is_closed,
        }
    }

    pub fn num_sub_paths(&self) -> usize {
        self.sub_paths.len()
    }
}

impl<'l> Iterator for SubPaths<'l> {
    type Item = SubPath<'l>;
    fn next(&mut self) -> Option<SubPath<'l>> {
        if self.sub_paths.is_empty() {
            return None;
        }

        let sp = self.sub_paths[0].clone();
        self.sub_paths = &self.sub_paths[1..];

        Some(SubPath{
            points: &self.points[sp.range],
            is_closed: sp.is_closed,
        })
    }
}

/// An iterator over the points of a sub-path.
pub struct SubPath<'l> {
    points: &'l[Point],
    is_closed: bool,
}

impl<'l> SubPath<'l> {
    /// Returns a slice of the points of this sub-path.
    pub fn points(&self) -> &'l[Point] {
        self.points
    }

    /// Returns whether this sub-path is closed.
    pub fn is_closed(&self) -> bool {
        self.is_closed
    }
}

/// A builder for flattened paths.
pub struct Builder {
    points: Vec<Point>,
    sub_paths: Vec<SubPathInfo>,
    sp_start: usize,
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            points: Vec::new(),
            sub_paths: Vec::new(),
            sp_start: 0,
        }
    }

    pub fn build(self) -> FlattenedPath {
        FlattenedPath {
            points: self.points,
            sub_paths: self.sub_paths,
        }
    }

    pub fn with_svg(self, tolerance: f32) -> SvgPathBuilder<FlatteningBuilder<Self>> {
        SvgPathBuilder::new(FlatteningBuilder::new(self, tolerance))
    }

    /// Add a closed polygon to the path.
    pub fn polygon(&mut self, points: &[Point]) {
        if points.is_empty() {
            return;
        }

        let start = self.points.len();
        self.points.extend_from_slice(points);
        let end = self.points.len();

        self.sub_paths.push(SubPathInfo {
            range: start..end,
            is_closed: true,
        });
    }
}

impl FlatPathBuilder for Builder {
    type PathType = FlattenedPath;

    fn move_to(&mut self, to: Point) {
        nan_check(to);
        let sp_end = self.points.len();
        if self.sp_start != sp_end {
            self.sub_paths.push(SubPathInfo {
                range: self.sp_start..sp_end,
                is_closed: false,
            });
        }
        self.sp_start = sp_end;
        self.points.push(to);
    }

    fn line_to(&mut self, to: Point) {
        nan_check(to);
        self.points.push(to);
    }

    fn close(&mut self) {
        let sp_end = self.points.len();
        if self.sp_start != sp_end {
            self.sub_paths.push(SubPathInfo {
                range: self.sp_start..sp_end,
                is_closed: true,
            });
        }
        self.sp_start = sp_end;
    }

    fn current_position(&self) -> Point {
        self.points.last().cloned().unwrap_or(Point::new(0.0, 0.0))
    }

    fn build(self) -> FlattenedPath {
        FlattenedPath {
            points: self.points,
            sub_paths: self.sub_paths,
        }
    }

    fn build_and_reset(&mut self) -> FlattenedPath {
        self.sp_start = 0;
        FlattenedPath {
            points: mem::replace(&mut self.points, Vec::new()),
            sub_paths: mem::replace(&mut self.sub_paths, Vec::new()),
        }
    }
}

#[inline]
fn nan_check(p: Point) {
    debug_assert!(!p.x.is_nan());
    debug_assert!(!p.y.is_nan());
}