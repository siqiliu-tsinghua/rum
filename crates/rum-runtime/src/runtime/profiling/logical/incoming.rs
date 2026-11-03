use super::{Call, Policy};

impl Call {
    pub(in crate::runtime) fn finish_incoming(self) {
        let policy = self
            .inner
            .epoch
            .inspect(self.inner.record)
            .map(|(_, _, policy)| policy);
        match policy {
            Some(Policy::Incoming(frozen)) => self.finish(frozen),
            Some(_) => {
                self.inner.epoch.fault();
                drop(self);
            }
            None => drop(self),
        }
    }
}
