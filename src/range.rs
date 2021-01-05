use serde::{Deserialize, Serialize};
use std::ops::Add;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Point {
    pub row: usize,
    pub column: usize,
}

impl Add<(usize, usize)> for Point {
    type Output = Point;

    fn add(self, rhs: (usize, usize)) -> Self::Output {
        Self {
            row: self.row + rhs.0,
            column: self.column + rhs.1,
        }
    }
}

impl Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Self::Output {
        Self {
            row: self.row + rhs.row,
            column: self.column + rhs.column,
        }
    }
}

impl Point {
    pub fn new(row: usize, column: usize) -> Self {
        Self { row, column }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Range {
    pub start: Point,
    pub end: Point,
}

impl Add<usize> for Range {
    type Output = Range;

    fn add(self, rhs: usize) -> Self::Output {
        Self {
            start: self.start + (0, rhs),
            end: self.end,
        }
    }
}

impl Add<Range> for Range {
    type Output = Range;

    fn add(self, rhs: Range) -> Self::Output {
        Self {
            start: self.start + rhs.start,
            end: self.end + rhs.end,
        }
    }
}

impl Range {
    pub fn new(start: (usize, usize), end: (usize, usize)) -> Self {
        Self {
            start: Point::new(start.0, start.1),
            end: Point::new(end.0, end.1),
        }
    }
}
