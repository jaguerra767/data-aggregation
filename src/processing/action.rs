use chrono::{DateTime, Utc};
use menu::action::Action;
use menu::libra_data::LibraData;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ActionAggregates {
    pub served: usize,
    pub ran_out: usize,
    pub heartbeat: usize,
    pub starting: usize,
    pub refilled: usize,
    pub offline: usize,
    pub timestamp: DateTime<Utc>,
}

impl ActionAggregates {
    pub fn new() -> Self {
        ActionAggregates {
            served: 0,
            ran_out: 0,
            heartbeat: 0,
            starting: 0,
            refilled: 0,
            offline: 0,
            timestamp: Utc::now(),
        }
    }

    pub fn from_existing(&self) -> Self {
        ActionAggregates {
            timestamp: Utc::now(),
            ..*self
        }
    }
}

pub fn aggregate_by_action(data: &[LibraData], action: Action) -> usize {
    data.iter().filter(|l| l.data_action == action).count()
}

pub fn aggregate_actions(
    data: &[LibraData],
    past_aggregate: &ActionAggregates,
) -> ActionAggregates {
    data.iter().fold(
        ActionAggregates::from_existing(past_aggregate),
        |mut agg, data| {
            match &data.data_action {
                Action::Served => agg.served += 1,
                Action::RanOut => agg.ran_out += 1,
                Action::Heartbeat => agg.heartbeat += 1,
                Action::Starting => agg.starting += 1,
                Action::Refilled => agg.refilled += 1,
                Action::Offline => agg.offline += 1,
            }
            agg
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use menu::action::Action;
    use menu::device::{Device, Model};
    use menu::libra_data::LibraData;
    use time::OffsetDateTime;

    #[test]
    fn test_aggregate_by_action() {
        let data = vec![
            LibraData {
                device: Device {
                    model: Model::LibraV0,
                    serial_number: "test-1".to_string(),
                },
                location: "kitchen".to_string(),
                ingredient: "apple".to_string(),
                data_action: Action::Served,
                amount: 1.0,
                timestamp: OffsetDateTime::now_utc(),
            },
            LibraData {
                device: Device {
                    model: Model::LibraV0,
                    serial_number: "test-2".to_string(),
                },
                location: "kitchen".to_string(),
                ingredient: "banana".to_string(),
                data_action: Action::RanOut,
                amount: 1.0,
                timestamp: OffsetDateTime::now_utc(),
            },
            LibraData {
                device: Device {
                    model: Model::LibraV0,
                    serial_number: "test-3".to_string(),
                },
                location: "kitchen".to_string(),
                ingredient: "apple".to_string(),
                data_action: Action::Served,
                amount: 1.0,
                timestamp: OffsetDateTime::now_utc(),
            },
        ];

        assert_eq!(aggregate_by_action(&data, Action::Served), 2);
        assert_eq!(aggregate_by_action(&data, Action::RanOut), 1);
    }

    #[test]
    fn test_aggregate_actions() {
        let data = vec![
            LibraData {
                device: Device {
                    model: Model::LibraV0,
                    serial_number: "test-1".to_string(),
                },
                location: "kitchen".to_string(),
                ingredient: "apple".to_string(),
                data_action: Action::Served,
                amount: 1.0,
                timestamp: OffsetDateTime::now_utc(),
            },
            LibraData {
                device: Device {
                    model: Model::LibraV0,
                    serial_number: "test-2".to_string(),
                },
                location: "kitchen".to_string(),
                ingredient: "banana".to_string(),
                data_action: Action::RanOut,
                amount: 1.0,
                timestamp: OffsetDateTime::now_utc(),
            },
            LibraData {
                device: Device {
                    model: Model::LibraV0,
                    serial_number: "test-3".to_string(),
                },
                location: "kitchen".to_string(),
                ingredient: "apple".to_string(),
                data_action: Action::Served,
                amount: 1.0,
                timestamp: OffsetDateTime::now_utc(),
            },
            LibraData {
                device: Device {
                    model: Model::LibraV0,
                    serial_number: "test-4".to_string(),
                },
                location: "kitchen".to_string(),
                ingredient: "banana".to_string(),
                data_action: Action::Refilled,
                amount: 1.0,
                timestamp: OffsetDateTime::now_utc(),
            },
            LibraData {
                device: Device {
                    model: Model::LibraV0,
                    serial_number: "test-5".to_string(),
                },
                location: "kitchen".to_string(),
                ingredient: "orange".to_string(),
                data_action: Action::Heartbeat,
                amount: 1.0,
                timestamp: OffsetDateTime::now_utc(),
            },
            LibraData {
                device: Device {
                    model: Model::LibraV0,
                    serial_number: "test-6".to_string(),
                },
                location: "kitchen".to_string(),
                ingredient: "orange".to_string(),
                data_action: Action::Starting,
                amount: 1.0,
                timestamp: OffsetDateTime::now_utc(),
            },
        ];

        let past_aggregate = ActionAggregates {
            served: 35,
            ran_out: 3,
            refilled: 2,
            heartbeat: 666,
            starting: 123,
            offline: 0,
            timestamp: Utc::now(),
        };

        let aggregate = aggregate_actions(&data, &past_aggregate);
        assert_eq!(aggregate.served, 37);
        assert_eq!(aggregate.ran_out, 4);
        assert_eq!(aggregate.heartbeat, 667);
        assert_eq!(aggregate.starting.clone(), 124);
        assert_eq!(aggregate.refilled, 3);
    }
}
