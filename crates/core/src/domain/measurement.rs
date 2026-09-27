use serde::{Deserialize, Serialize};
use time::{Date, OffsetDateTime};

/// What was measured. Each kind has one unit, so the unit is never stored beside the value.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MeasurementKind {
    Waist,
    Hips,
    Chest,
    Neck,
    Arm,
    Thigh,
    /// Kept as a measurement so the waist-to-height ratio needs nothing else.
    Height,
    BodyFat,
}

/// The unit a [`MeasurementKind`] is recorded in.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MeasurementUnit {
    Centimetre,
    Percent,
}

impl MeasurementKind {
    pub const ALL: [Self; 8] = [
        Self::Waist,
        Self::Hips,
        Self::Chest,
        Self::Neck,
        Self::Arm,
        Self::Thigh,
        Self::Height,
        Self::BodyFat,
    ];

    pub fn unit(self) -> MeasurementUnit {
        match self {
            Self::BodyFat => MeasurementUnit::Percent,
            _ => MeasurementUnit::Centimetre,
        }
    }

    /// The name the kind is stored and sent under.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Waist => "waist",
            Self::Hips => "hips",
            Self::Chest => "chest",
            Self::Neck => "neck",
            Self::Arm => "arm",
            Self::Thigh => "thigh",
            Self::Height => "height",
            Self::BodyFat => "body-fat",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.as_str() == value)
    }
}

/// One measurement of one kind. A date holds at most one per kind, like a weigh-in.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Measurement {
    #[serde(with = "super::format::iso_date")]
    pub date: Date,
    pub kind: MeasurementKind,
    pub value: f64,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_round_trips_through_its_stored_name() {
        for kind in MeasurementKind::ALL {
            assert_eq!(MeasurementKind::parse(kind.as_str()), Some(kind));
        }
    }

    #[test]
    fn the_stored_name_is_the_wire_name() {
        for kind in MeasurementKind::ALL {
            let wire = serde_json::to_value(kind).unwrap();
            assert_eq!(wire, kind.as_str());
        }
    }
}
