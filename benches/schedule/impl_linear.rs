use crate::util::{Event, TimeRange};

#[derive(Debug, Clone)]
pub struct Schedule<'a, V> {
    events: Vec<(TimeRange, &'a V)>,
}

impl<'a, V> Schedule<'a, V> {
    pub fn get(&self, time: &chrono::NaiveTime) -> Option<&'a V> {
        self.events.iter().find_map(|(range, value)| {
            if range.contains(time) {
                Some(*value)
            } else {
                None
            }
        })
    }
}

impl<'a> FromIterator<&'a Event> for Schedule<'a, Event<chrono::NaiveTime>> {
    fn from_iter<T: IntoIterator<Item = &'a Event>>(iter: T) -> Self {
        let mut inputs: Vec<_> = iter.into_iter().collect();

        if inputs.is_empty() {
            return Self { events: Vec::new() };
        }

        inputs.sort_by(|a, b| a.start_time.cmp(&b.start_time));

        // SAFETY: The function returns early above if `inputs` is empty.
        let last = unsafe { *inputs.last().unwrap_unchecked() };
        let events: Vec<(TimeRange, &Event)> = inputs
            .windows(2)
            .map(|window| {
                let (event, next_event) = (window[0], window[1]);
                let range = TimeRange::new(
                    event.start_time,
                    event.end_time.unwrap_or(next_event.start_time),
                );
                (range, event)
            })
            .chain(std::iter::once((
                TimeRange::new(
                    last.start_time,
                    last.end_time
                        // SAFETY: The function returns early above if `inputs` is empty.
                        .unwrap_or_else(|| unsafe { inputs.first().unwrap_unchecked().start_time }),
                ),
                last,
            )))
            .collect();
        Self { events }
    }
}
