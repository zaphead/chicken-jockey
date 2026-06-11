use engine_net::InventoryAction;

#[derive(Debug, Default)]
pub struct PendingInventoryActions {
    pub actions: Vec<InventoryAction>,
}

impl PendingInventoryActions {
    pub fn push(&mut self, action: InventoryAction) {
        self.actions.push(action);
    }

    pub fn drain(&mut self) -> Vec<InventoryAction> {
        std::mem::take(&mut self.actions)
    }
}
