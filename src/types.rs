use std::collections::HashMap;

use crate::{exchange::KlineNumber, pick::Pick};

pub type Row = Vec<HashMap<Pick, KlineNumber>>;
