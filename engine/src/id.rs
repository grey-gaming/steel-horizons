//! Identity types for entities, resources, and commands.
//!
//! Each ID is a newtype wrapping a string, serialized transparently as a plain
//! JSON string.  IDs are compared and ordered by their inner string value,
//! giving a deterministic byte ordering for `BTreeMap`/`BTreeSet` keys.
//!
//! Generated IDs use the reserved prefixes defined in GDD 13 §Identifiers and
//! Resources; authored IDs must avoid those prefixes (validated in P1-04).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

macro_rules! id_newtype {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl $name {
            /// Return the inner string reference.
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Convert to the inner string, consuming this wrapper.
            pub fn into_string(self) -> String {
                self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

id_newtype!(
    /// Unique identifier for a celestial body (e.g. `planet_haven`).
    BodyId
);
id_newtype!(
    /// Unique identifier for a station (e.g. `hub_haven`).
    StationId
);
id_newtype!(
    /// Unique identifier for a ship (e.g. `ship_builder_1`).
    ShipId
);
id_newtype!(
    /// Unique identifier for a build order.
    BuildOrderId
);
id_newtype!(
    /// Unique identifier for a salvage cache.
    SalvageId
);
id_newtype!(
    /// Unique identifier for a logistics reservation.
    ReservationId
);
id_newtype!(
    /// Unique identifier for a survey order.
    SurveyOrderId
);
id_newtype!(
    /// Unique identifier for a technology.
    TechId
);
id_newtype!(
    /// Unique identifier for a recipe.
    RecipeId
);
id_newtype!(
    /// Unique identifier for a starting scenario.
    ScenarioId
);

#[cfg(test)]
mod tests {
    use super::*;

    /// IDs serialize and deserialize as plain JSON strings.
    #[test]
    fn id_round_trip() {
        let body = BodyId("planet_haven".into());
        let json = serde_json::to_string(&body).unwrap();
        assert_eq!(json, "\"planet_haven\"");
        let back: BodyId = serde_json::from_str(&json).unwrap();
        assert_eq!(body, back);
    }

    /// IDs compare by their inner string.
    #[test]
    fn id_ordering() {
        let a = BodyId("alpha".into());
        let b = BodyId("beta".into());
        let c = BodyId("alpha".into());
        assert!(a < b);
        assert_eq!(a, c);
    }

    /// An empty-string ID is valid (placeholder use only).
    #[test]
    fn empty_string_id() {
        let id = StationId(String::new());
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"\"");
        let back: StationId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, back);
    }
}
