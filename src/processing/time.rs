use menu::action::Action;
use menu::libra_data::LibraData;
use std::collections::HashMap;
use time::Date;

pub fn aggregate_hourly(
    data: &[LibraData],
    action: Action,
    past_aggregate: &HashMap<u8, usize>,
) -> HashMap<u8, usize> {
    data.iter()
        .filter(|data| data.data_action == action)
        .fold(HashMap::new(), |mut map, data| {
            *map.entry(data.timestamp.hour()).or_insert(
                *past_aggregate
                    .get(&data.timestamp.hour())
                    .unwrap_or(&0)
            ) += 1;
            map
        })
}

pub fn aggregate_daily(
    data: &[LibraData],
    action: Action,
    past_aggregate: &HashMap<Date, usize>,
) -> HashMap<Date, usize> {
    data.iter()
        .filter(|data| data.data_action == action)
        .fold(HashMap::new(), |mut map, data| {
            *map.entry(data.timestamp.date()).or_insert(
                *past_aggregate
                    .get(&data.timestamp.date())
                    .unwrap_or(&0)
            ) += 1;
            map
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use menu::device::{Device, Model};
    use time::{Duration, OffsetDateTime};

    fn create_libra_data(timestamp: OffsetDateTime, device: &Device, action: Action) -> LibraData {
        LibraData {
            device: device.clone(),
            location: "kitchen".to_string(),
            ingredient: "test_ingredient".to_string(),
            data_action: action,
            amount: 1.0,
            timestamp,
        }
    }

    #[test]
    fn it_aggregates_hourly() {
        let now = OffsetDateTime::now_utc();
        let one_hour_ago = now - Duration::hours(1);
        let two_hours_ago = now - Duration::hours(2);

        let device = Device {
            model: Model::LibraV0,
            serial_number: "test".to_string(),
        };

        let data = vec![
            create_libra_data(one_hour_ago, &device, Action::Served),
            create_libra_data(one_hour_ago + Duration::minutes(5), &device, Action::Served),
            create_libra_data(two_hours_ago, &device, Action::RanOut),
        ];

        let mut past_aggregate: HashMap<u8, usize> = HashMap::new();
        past_aggregate.insert(one_hour_ago.hour(), 10);
        past_aggregate.insert(two_hours_ago.hour(), 5);

        let result_served = aggregate_hourly(&data, Action::Served, &past_aggregate);
        assert_eq!(result_served.get(&one_hour_ago.hour()), Some(&12));
        assert_eq!(result_served.get(&two_hours_ago.hour()), None);

        let result_ran_out = aggregate_hourly(&data, Action::RanOut, &past_aggregate);
        assert_eq!(result_ran_out.get(&two_hours_ago.hour()), Some(&6));
        assert_eq!(result_ran_out.get(&one_hour_ago.hour()), None);
    }

    #[test]
    fn it_aggregates_daily() {
        let now = OffsetDateTime::now_utc();
        let yesterday = now - Duration::days(1);
        let day_before_yesterday = now - Duration::days(2);

        let device = Device {
            model: Model::LibraV0,
            serial_number: "test".to_string(),
        };

        let data = vec![
            create_libra_data(yesterday, &device, Action::Served),
            create_libra_data(yesterday + Duration::hours(1), &device, Action::Served),
            create_libra_data(day_before_yesterday, &device, Action::RanOut),
        ];

        let mut past_aggregate: HashMap<Date, usize> = HashMap::new();
        past_aggregate.insert(yesterday.date(), 20);
        past_aggregate.insert(day_before_yesterday.date(), 15);

        let result_served = aggregate_daily(&data, Action::Served, &past_aggregate);
        assert_eq!(result_served.get(&yesterday.date()), Some(&22));
        assert_eq!(result_served.get(&day_before_yesterday.date()), None);

        let result_ran_out = aggregate_daily(&data, Action::RanOut, &past_aggregate);
        assert_eq!(result_ran_out.get(&day_before_yesterday.date()), Some(&16));
        assert_eq!(result_ran_out.get(&yesterday.date()), None);
    }
}
