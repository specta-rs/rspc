// use std::{
//     error::Error,
//     fmt::{Debug, Display},
// };

// pub enum InternalError {
//     DeserializingArg(serde_json::Error),
// }

// impl Debug for InternalError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         // match self {}
//         todo!()
//     }
// }

// impl Display for InternalError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         todo!()
//     }
// }

// impl Error for InternalError {
//     fn source(&self) -> Option<&(dyn Error + 'static)> {
//         None
//     }
// }
