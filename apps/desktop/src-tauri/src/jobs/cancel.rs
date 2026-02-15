use std::collections::BTreeSet;
use uuid::Uuid;

#[derive(Default)]
pub struct CancellationSet {
    ids: BTreeSet<Uuid>,
}

impl CancellationSet {
    pub fn cancel(&mut self, id: Uuid) {
        self.ids.insert(id);
    }

    pub fn is_cancelled(&self, id: Uuid) -> bool {
        self.ids.contains(&id)
    }
}
