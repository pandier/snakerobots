#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn direction(mut self, dir: Direction) -> Self {
        dir.apply(&mut self);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

impl Size {
    pub fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }

    pub fn area(&self) -> i32 {
        self.width * self.height
    }

    pub fn contains(&self, p: &Point) -> bool {
        p.x >= 0 && p.y >= 0 && p.x < self.width && p.y < self.height
    }

    pub fn iter(&self) -> impl Iterator<Item = Point> {
        (0..self.height).flat_map(|y| (0..self.width).map(move |x| Point::new(x, y)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub const ALL: [Direction; 4] = [
        Direction::Up,
        Direction::Down,
        Direction::Left,
        Direction::Right,
    ];

    pub fn opposite(&self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Right => Direction::Left,
            Direction::Left => Direction::Right,
        }
    }

    pub fn apply(&self, vec: &mut Point) {
        match self {
            Direction::Up => vec.y -= 1,
            Direction::Down => vec.y += 1,
            Direction::Left => vec.x -= 1,
            Direction::Right => vec.x += 1,
        }
    }
}
