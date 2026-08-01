use serde::{Deserialize, Serialize};

use crate::types::LimitInfo;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum NotificationColor {
    Green,
    Yellow,
    Orange,
    Red,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LimitNotificationKind {
    Remaining75,
    Remaining50,
    Remaining25,
    Remaining10,
}

impl LimitNotificationKind {
    pub const ALL: [Self; 4] = [
        Self::Remaining75,
        Self::Remaining50,
        Self::Remaining25,
        Self::Remaining10,
    ];

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "75" => Ok(Self::Remaining75),
            "50" => Ok(Self::Remaining50),
            "25" => Ok(Self::Remaining25),
            "10" => Ok(Self::Remaining10),
            _ => Err("expected one of: 75, 50, 25, 10".to_string()),
        }
    }

    pub fn remaining_percent(self) -> u8 {
        match self {
            Self::Remaining75 => 75,
            Self::Remaining50 => 50,
            Self::Remaining25 => 25,
            Self::Remaining10 => 10,
        }
    }

    pub fn color(self) -> NotificationColor {
        match self {
            Self::Remaining75 => NotificationColor::Green,
            Self::Remaining50 => NotificationColor::Yellow,
            Self::Remaining25 => NotificationColor::Orange,
            Self::Remaining10 => NotificationColor::Red,
        }
    }

    pub fn emoji(self) -> &'static str {
        match self {
            Self::Remaining75 => "🟢",
            Self::Remaining50 => "🟡",
            Self::Remaining25 => "🟠",
            Self::Remaining10 => "🔴",
        }
    }
}

pub(super) fn matching_kind(remaining_percent: f64) -> Option<LimitNotificationKind> {
    let remaining = remaining_percent.clamp(0.0, 100.0);

    if remaining <= 10.0 {
        Some(LimitNotificationKind::Remaining10)
    } else if remaining <= 25.0 {
        Some(LimitNotificationKind::Remaining25)
    } else if remaining <= 50.0 {
        Some(LimitNotificationKind::Remaining50)
    } else if remaining <= 75.0 {
        Some(LimitNotificationKind::Remaining75)
    } else {
        None
    }
}

pub(super) fn remaining_percent(limit: &LimitInfo) -> Option<f64> {
    limit
        .remaining_percent
        .or_else(|| limit.used_percent.map(|used| 100.0 - used))
        .map(|remaining| remaining.clamp(0.0, 100.0))
}
