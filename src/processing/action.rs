use std::collections::HashMap;

use menu::action::Action;
use menu::libra_data::LibraData;

pub fn aggregate_by_action(data: &[LibraData], action: Action) -> usize {
    data.iter().filter(|l| l.data_action == action).count()
}

pub fn aggregate_actions(data: &[LibraData]) -> HashMap<Action, usize> {
    data.iter().fold(HashMap::new(), |mut map, data| {
        *map.entry(data.data_action.clone()).or_insert(0) += 1;
        map
    })
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
}
