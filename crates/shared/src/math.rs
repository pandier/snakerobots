use std::fmt;

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

#[cfg(feature = "lang")]
impl crate::lang::util::arg_convertor::ValueLike for Point {
    fn into_convertable(self) -> crate::lang::util::arg_convertor::ValueConvertable {
        let mut s = crate::lang::util::arg_convertor::struct_convertor("Point");
        s.add_field(self.x);
        s.add_field(self.y);
        s.convert()
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

    pub fn vec_to_string(vec: &Vec<Self>) -> String {
        vec.iter().map(|d| char::from(*d)).collect()
    }

    pub fn try_vec_from_string(string: String) -> Result<Vec<Self>, DirFromCharError> {
        string.chars().map(|c| Direction::try_from(c)).collect()
    }
}

#[cfg(feature = "lang")]
impl crate::lang::util::arg_convertor::ValueLike for Direction {
    fn into_convertable(self) -> crate::lang::util::arg_convertor::ValueConvertable {
        match self {
            Self::Up => 0.into_convertable(),
            Self::Down => 1.into_convertable(),
            Self::Left => 2.into_convertable(),
            Self::Right => 3.into_convertable(),
        }
    }
}

#[derive(Debug)]
pub struct DirFromCharError(char);

impl fmt::Display for DirFromCharError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid move: {}", self.0)
    }
}

impl std::error::Error for DirFromCharError {}

impl TryFrom<char> for Direction {
    type Error = DirFromCharError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'u' | 'U' => Ok(Self::Up),
            'd' | 'D' => Ok(Self::Down),
            'l' | 'L' => Ok(Self::Left),
            'r' | 'R' => Ok(Self::Right),
            _ => Err(DirFromCharError(value)),
        }
    }
}

impl From<Direction> for char {
    fn from(value: Direction) -> Self {
        match value {
            Direction::Up => 'u',
            Direction::Down => 'd',
            Direction::Left => 'l',
            Direction::Right => 'r',
        }
    }
}
