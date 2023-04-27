use std::collections::HashMap;

use crate::{exchange::KlineNumber, pick::Pick};

pub type Raw = Vec<HashMap<Pick, KlineNumber>>;
