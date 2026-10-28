//! Session output order. A stamp is handed out once; a mark is a clock value,
//! so it names the same items before and after a snapshot restore.

/// A position in the session's output order. Every push to a session output
/// list takes the next stamp; nothing ever hands one out again.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::runtime) struct OutputStamp(u64);

/// Everything stamped at or before this point. A mark is a value of the
/// session's output clock, not a position in any list, so it means the same
/// thing before and after any snapshot restore or rollback.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::runtime) struct OutputMark(u64);

impl OutputMark {
    #[cfg(test)]
    pub(in crate::runtime) const START: Self = Self(0);

    pub(in crate::runtime) fn from_clock(clock: u64) -> Self {
        Self(clock)
    }
}

impl OutputStamp {
    /// Only the session's clock mints stamps (`Session::next_output_stamp`).
    pub(super) fn from_clock(clock: u64) -> Self {
        Self(clock)
    }
}

/// One session output list. Items are sorted by strictly increasing stamp:
/// a restore only brings back items stamped before every later push.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) struct OutputLog<T> {
    items: Vec<(OutputStamp, T)>,
}

impl<T> Default for OutputLog<T> {
    fn default() -> Self {
        Self { items: Vec::new() }
    }
}

impl<T> OutputLog<T> {
    pub(in crate::runtime) fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    pub(in crate::runtime) fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
        }
    }

    pub(in crate::runtime) fn push(&mut self, stamp: OutputStamp, value: T) {
        if let Some((last, _)) = self.items.last() {
            debug_assert!(stamp > *last, "output stamps must strictly increase");
        }
        self.items.push((stamp, value));
    }

    fn first_after(&self, mark: OutputMark) -> usize {
        self.items.partition_point(|(stamp, _)| stamp.0 <= mark.0)
    }

    /// The items stamped after `mark`, oldest first.
    pub(in crate::runtime) fn after(&self, mark: OutputMark) -> impl ExactSizeIterator<Item = &T> {
        let start = self.first_after(mark);
        self.items[start..].iter().map(|(_, value)| value)
    }

    /// The items stamped after `from` and at or before `through`.
    pub(in crate::runtime) fn between(
        &self,
        from: OutputMark,
        through: OutputMark,
    ) -> impl ExactSizeIterator<Item = &T> {
        let start = self.first_after(from);
        let end = self.first_after(through).max(start);
        self.items[start..end].iter().map(|(_, value)| value)
    }

    /// Whether any item is stamped after `mark`. Reads only the last stamp.
    pub(in crate::runtime) fn has_after(&self, mark: OutputMark) -> bool {
        self.items.last().is_some_and(|(stamp, _)| stamp.0 > mark.0)
    }

    pub(in crate::runtime) fn split_after(&mut self, mark: OutputMark) -> Vec<T> {
        let start = self.first_after(mark);
        self.items
            .split_off(start)
            .into_iter()
            .map(|(_, value)| value)
            .collect()
    }

    /// Drops the items stamped after `mark`. A cut with nothing after it reads
    /// only the last stamp, inline, as a positional truncate did.
    #[inline]
    pub(in crate::runtime) fn truncate_after(&mut self, mark: OutputMark) {
        if self.has_after(mark) {
            self.truncate_found_after(mark);
        }
    }

    #[inline(never)]
    fn truncate_found_after(&mut self, mark: OutputMark) {
        let start = self.first_after(mark);
        self.items.truncate(start);
    }

    pub(in crate::runtime) fn iter(&self) -> impl ExactSizeIterator<Item = &T> {
        self.items.iter().map(|(_, value)| value)
    }

    pub(in crate::runtime) fn len(&self) -> usize {
        self.items.len()
    }

    #[cfg(test)]
    pub(in crate::runtime) fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub(in crate::runtime) fn capacity(&self) -> usize {
        self.items.capacity()
    }

    pub(in crate::runtime) fn last(&self) -> Option<&T> {
        self.items.last().map(|(_, value)| value)
    }

    pub(in crate::runtime) fn view(&self) -> OutputView<'_, T> {
        OutputView { items: &self.items }
    }

    pub(in crate::runtime) fn into_values(self) -> Vec<T> {
        self.items.into_iter().map(|(_, value)| value).collect()
    }
}

impl<T: Clone> OutputLog<T> {
    pub(in crate::runtime) fn after_vec(&self, mark: OutputMark) -> Vec<T> {
        self.after(mark).cloned().collect()
    }

    pub(in crate::runtime) fn between_vec(&self, from: OutputMark, through: OutputMark) -> Vec<T> {
        self.between(from, through).cloned().collect()
    }
}

impl<T, U> PartialEq<[U]> for OutputLog<T>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &[U]) -> bool {
        self.len() == other.len()
            && self
                .iter()
                .zip(other.iter())
                .all(|(left, right)| left == right)
    }
}

impl<T, U> PartialEq<&[U]> for OutputLog<T>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &&[U]) -> bool {
        self.eq(*other)
    }
}

impl<T, U> PartialEq<Vec<U>> for OutputLog<T>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &Vec<U>) -> bool {
        self.eq(other.as_slice())
    }
}

impl<T, U, const N: usize> PartialEq<[U; N]> for OutputLog<T>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &[U; N]) -> bool {
        self.eq(other.as_slice())
    }
}

impl<T, U, const N: usize> PartialEq<&[U; N]> for OutputLog<T>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &&[U; N]) -> bool {
        self.eq(other.as_slice())
    }
}

/// Borrowed view of one output log's values, in stamp order.
pub struct OutputView<'a, T> {
    items: &'a [(OutputStamp, T)],
}

impl<'a, T> Copy for OutputView<'a, T> {}

impl<'a, T> Clone for OutputView<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T: std::fmt::Debug> std::fmt::Debug for OutputView<'a, T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_list().entries(self.iter()).finish()
    }
}

impl<'a, T> OutputView<'a, T> {
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = &'a T> + 'a {
        self.items.iter().map(|(_, value)| value)
    }

    pub fn last(&self) -> Option<&'a T> {
        self.items.last().map(|(_, value)| value)
    }
}

impl<'a, T, U> PartialEq<[U]> for OutputView<'a, T>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &[U]) -> bool {
        self.len() == other.len()
            && self
                .iter()
                .zip(other.iter())
                .all(|(left, right)| left == right)
    }
}

impl<'a, T, U> PartialEq<&[U]> for OutputView<'a, T>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &&[U]) -> bool {
        self.eq(*other)
    }
}

impl<'a, T, U> PartialEq<Vec<U>> for OutputView<'a, T>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &Vec<U>) -> bool {
        self.eq(other.as_slice())
    }
}

impl<'a, T, U, const N: usize> PartialEq<[U; N]> for OutputView<'a, T>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &[U; N]) -> bool {
        self.eq(other.as_slice())
    }
}

impl<'a, T, U, const N: usize> PartialEq<&[U; N]> for OutputView<'a, T>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &&[U; N]) -> bool {
        self.eq(other.as_slice())
    }
}

/// Borrowed session trace events, in stamp order. Depths stay paired in the log.
pub struct SessionTraceView<'a> {
    items: OutputView<'a, (super::TraceEvent, usize)>,
}

impl<'a> Copy for SessionTraceView<'a> {}

impl<'a> Clone for SessionTraceView<'a> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a> SessionTraceView<'a> {
    pub(in crate::runtime) fn new(items: OutputView<'a, (super::TraceEvent, usize)>) -> Self {
        Self { items }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = &'a super::TraceEvent> + 'a {
        self.items.iter().map(|(event, _depth)| event)
    }

    pub fn last(&self) -> Option<&'a super::TraceEvent> {
        self.items.last().map(|(event, _depth)| event)
    }
}
