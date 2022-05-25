use crate::enums::Status;

#[derive(Debug, Clone)]
pub struct Reason {
    pub uid: i64,
    pub op: Status,
    pub op_role: String,
    pub reason: String,
    pub op_time: i64,
}

#[derive(Debug, Clone)]
pub struct User {
    pub uid: i64,
    pub status: Status,
    pub last_reason: Option<String>,
}

// impl User {
//     pub fn is_black(&self) -> bool {
//         self.status == Status::Black
//     }

//     pub fn is_white(&self) -> bool {
//         self.status == Status::White
//     }

//     pub fn is_none(&self) -> bool {
//         self.status == Status::None
//     }

//     pub fn reason(&self) -> &str {
//         match self.last_reason {
//             Some(ref reason) => reason,
//             None => "无",
//         }
//     }

//     pub fn new(uid: i64) -> Self {
//         User {
//             uid,
//             status: Status::None,
//             last_reason: None,
//         }
//     }
// }
