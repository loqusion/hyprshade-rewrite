use std::cmp::Ordering;

use chrono::NaiveTime;

use crate::{
    config::{Config, Shader as ShaderConfig},
    resolver::{self, Resolver},
    shader::Shader,
};

pub struct Schedule<'a> {
    table: ScheduleTable<'a, ShaderConfig>,
    config: &'a Config,
}

impl<'a> Schedule<'a> {
    pub fn with_config(config: &'a Config) -> Self {
        Self {
            table: ScheduleTable::from_iter(config.all_shaders()),
            config,
        }
    }

    pub fn scheduled_shader(&self, time: &NaiveTime) -> Result<Option<Shader>, resolver::Error> {
        self.table
            .get(time)
            .or_else(|| self.config.default_shader())
            .map(|s| Resolver::with_name(&s.name).resolve())
            .transpose()
    }
}

struct ScheduleTable<'a, T> {
    events: Vec<(TimeRange, &'a T)>,
}

impl<'a, T> ScheduleTable<'a, T> {
    fn get(&self, time: &NaiveTime) -> Option<&'a T> {
        let index = self
            .events
            .binary_search_by(|(range, _)| {
                if range.contains(time) {
                    Ordering::Equal
                } else {
                    range.start.cmp(time)
                }
            })
            .ok()
            // We have to check the last one manually to account for wrapping time.
            // E.g. If time=01:00 and last event is 23:00-02:00, the last event will sometimes be
            // skipped by binary search unless we do this.
            .or_else(|| {
                self.events
                    .last()?
                    .0
                    .contains(time)
                    .then(|| self.events.len() - 1)
            })?;
        Some(self.events[index].1)
    }
}

impl<'a> FromIterator<&'a ShaderConfig> for ScheduleTable<'a, ShaderConfig> {
    fn from_iter<T: IntoIterator<Item = &'a ShaderConfig>>(iter: T) -> Self {
        let mut inputs: Vec<_> = iter
            .into_iter()
            .filter(|shader_config| shader_config.start_time.is_some())
            .collect();

        if inputs.is_empty() {
            return Self { events: Vec::new() };
        }

        inputs.sort_by(|a, b| a.start_time.cmp(&b.start_time));

        let events: Vec<(TimeRange, &ShaderConfig)> = inputs
            .windows(2)
            .map(|window| {
                let (event, next_event) = (window[0], window[1]);
                // SAFETY: All items with no `start_time` are filtered out above.
                let range = unsafe {
                    let start_time = event.start_time.unwrap_unchecked();
                    let end_time = event
                        .end_time
                        .unwrap_or(next_event.start_time.unwrap_unchecked());
                    TimeRange::new(start_time, end_time)
                };
                (range, event)
            })
            .chain(std::iter::once(
                // SAFETY: the function returns early above if `inputs` is empty.
                // All items with no `start_time` are filtered out above.
                unsafe {
                    let last = inputs.last().unwrap_unchecked();
                    let start_time = last.start_time.unwrap_unchecked();
                    let end_time = last.end_time.unwrap_or_else(|| {
                        inputs
                            .first()
                            .unwrap_unchecked()
                            .start_time
                            .unwrap_unchecked()
                    });
                    (TimeRange::new(start_time, end_time), *last)
                },
            ))
            .collect();
        Self { events }
    }
}

#[derive(Debug, Clone)]
struct TimeRange {
    pub start: NaiveTime,
    pub end: NaiveTime,
}

impl TimeRange {
    pub fn new(start: NaiveTime, end: NaiveTime) -> Self {
        Self { start, end }
    }

    pub fn contains(&self, item: &NaiveTime) -> bool {
        if self.start < self.end {
            self.start <= *item && *item < self.end
        } else {
            self.start <= *item || *item < self.end
        }
    }
}

impl From<(NaiveTime, NaiveTime)> for TimeRange {
    fn from(value: (NaiveTime, NaiveTime)) -> Self {
        Self {
            start: value.0,
            end: value.1,
        }
    }
}
