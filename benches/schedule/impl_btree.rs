use std::collections::BTreeMap;

use crate::util::Event;

#[derive(Debug, Clone)]
pub struct Schedule<'a, K, V> {
    events: BTreeMap<K, Option<&'a V>>,
}

impl<'a, K, V> Schedule<'a, K, V>
where
    K: Ord,
{
    pub fn get(&self, time: &K) -> Option<&'a V> {
        self.events
            .range(..=time)
            .next_back()
            .or_else(|| self.events.last_key_value())
            .and_then(|(_, v)| *v)
    }
}

impl<'a> FromIterator<&'a Event> for Schedule<'a, chrono::NaiveTime, Event<chrono::NaiveTime>> {
    fn from_iter<T: IntoIterator<Item = &'a Event>>(iter: T) -> Self {
        let mut events = BTreeMap::new();
        iter.into_iter().for_each(|event| {
            if let Some(end_time) = event.end_time {
                events.entry(end_time).or_insert(None);
            }
            events.insert(event.start_time, Some(event));
        });
        Self { events }
    }
}
