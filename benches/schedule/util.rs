use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Event<T = chrono::NaiveTime> {
    pub start_time: T,
    pub end_time: Option<T>,
}

impl<S> From<(S, Option<S>)> for Event<chrono::NaiveTime>
where
    S: AsRef<str>,
{
    fn from(value: (S, Option<S>)) -> Self {
        Self {
            start_time: chrono::NaiveTime::from_str(value.0.as_ref()).unwrap(),
            end_time: value
                .1
                .map(|s| chrono::NaiveTime::from_str(s.as_ref()))
                .transpose()
                .unwrap(),
        }
    }
}

/// `TimeRange` is wrapping in the same way that clock time is wrapping.
/// This means that if `start` > `end`, `.contains(time)` should return `true` if `time` <= `start`
/// or `time` > `end`.
#[derive(Debug, Clone)]
pub struct TimeRange<T = chrono::NaiveTime> {
    pub start: T,
    pub end: T,
}

impl<T> TimeRange<T> {
    pub fn new(start: T, end: T) -> Self {
        Self { start, end }
    }
}

impl<T> TimeRange<T>
where
    T: Ord,
{
    #[inline]
    pub fn contains<U>(&self, item: &U) -> bool
    where
        T: PartialOrd<U>,
        U: ?Sized + PartialOrd<T>,
    {
        if self.start < self.end {
            self.start <= *item && *item < self.end
        } else {
            self.start <= *item || *item < self.end
        }
    }
}

impl From<(chrono::NaiveTime, chrono::NaiveTime)> for TimeRange {
    fn from(value: (chrono::NaiveTime, chrono::NaiveTime)) -> Self {
        Self {
            start: value.0,
            end: value.1,
        }
    }
}
