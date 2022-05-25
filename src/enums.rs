#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    None = 0,
    Black = 1,
    White = 2,
}

impl Status {
    pub fn from(value: i8) -> Self {
        match value {
            0 => Status::None,
            1 => Status::Black,
            2 => Status::White,
            _ => Status::None,
        }
    }

    pub fn into(&self) -> i8 {
        match self {
            Status::None => 0,
            Status::Black => 1,
            Status::White => 2,
        }
    }

    pub fn display(&self) -> &str {
        match self {
            Status::None => "normal",
            Status::Black => "black",
            Status::White => "white",
        }
    }
}