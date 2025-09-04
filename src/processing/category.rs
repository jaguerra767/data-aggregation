use menu::libra_data::LibraData;
use std::collections::HashMap;

pub fn aggregate_by_category(
    data: &[LibraData],
    past_aggregate: &HashMap<String, usize>,
) -> HashMap<String, usize> {
    data.iter().fold(HashMap::new(), |mut map, data| {
        *map.entry(data.ingredient.clone())
            .or_insert(*past_aggregate.get(&data.ingredient).unwrap_or(&0)) += 1;
        map
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use menu::action::Action;
    use menu::device::{Device, Model};
    use menu::libra_data::LibraData;
    use std::collections::HashMap;
    use time::OffsetDateTime;

    #[test]
    fn test_aggregate_by_category_with_items() {
        let data = vec![
            LibraData {
                device: Device {
                    model: Model::LibraV0,
                    serial_number: String::from("123"),
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
                    serial_number: String::from("456"),
                },
                location: "kitchen".to_string(),
                ingredient: "banana".to_string(),
                data_action: Action::Served,
                amount: 1.0,
                timestamp: OffsetDateTime::now_utc(),
            },
            LibraData {
                device: Device {
                    model: Model::LibraV0,
                    serial_number: String::from("789"),
                },
                location: "kitchen".to_string(),
                ingredient: "apple".to_string(),
                data_action: Action::RanOut,
                amount: 1.0,
                timestamp: OffsetDateTime::now_utc(),
            },
        ];

        let mut past_aggregate: HashMap<String, usize> = HashMap::new();
        past_aggregate.insert("apple".to_string(), 77);
        past_aggregate.insert("banana".to_string(), 66);

        let result = aggregate_by_category(&data, &past_aggregate);
        let mut expected = HashMap::new();
        expected.insert("apple".to_string(), 79);
        expected.insert("banana".to_string(), 67);

        assert_eq!(result, expected);
    }
}
