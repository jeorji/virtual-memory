use serde::{Deserialize, Serialize};

pub trait Item<'a>: Deserialize<'a> + Serialize {}
