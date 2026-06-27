use nightshade::prelude::serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "nightshade::prelude::serde")]
pub struct Marker;
