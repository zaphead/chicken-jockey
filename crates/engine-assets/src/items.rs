use engine_world::{BlockId, BlockState};

use crate::blocks::BlockRegistry;
use crate::tools::{ToolId, ToolRegistry};

pub const BLOCK_MAX_STACK: u16 = 100;
pub const TOOL_MAX_STACK: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemKind {
    Block {
        id: BlockId,
        state: BlockState,
    },
    Tool(ToolId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemStack {
    pub kind: ItemKind,
    pub count: u16,
}

impl ItemStack {
    pub fn block(id: BlockId, count: u16) -> Self {
        Self {
            kind: ItemKind::Block {
                id,
                state: BlockState::default(),
            },
            count,
        }
    }

    pub fn tool(id: ToolId) -> Self {
        Self {
            kind: ItemKind::Tool(id),
            count: 1,
        }
    }

    pub fn is_tool(self) -> bool {
        matches!(self.kind, ItemKind::Tool(_))
    }

    pub fn split(self, amount: u16) -> (Self, Option<Self>) {
        debug_assert!(amount > 0);
        if amount >= self.count {
            return (self, None);
        }
        let remainder = Self {
            kind: self.kind,
            count: self.count - amount,
        };
        (
            Self {
                kind: self.kind,
                count: amount,
            },
            Some(remainder),
        )
    }
}

pub fn max_stack(kind: ItemKind) -> u16 {
    match kind {
        ItemKind::Block { .. } => BLOCK_MAX_STACK,
        ItemKind::Tool(_) => TOOL_MAX_STACK,
    }
}

pub fn stacks_merge(a: ItemKind, b: ItemKind) -> bool {
    a == b
}

pub fn clamp_stack_count(kind: ItemKind, count: u16) -> u16 {
    count.min(max_stack(kind))
}

pub fn item_kind_registry_name(
    kind: ItemKind,
    blocks: &BlockRegistry,
    tools: &ToolRegistry,
) -> Option<String> {
    match kind {
        ItemKind::Tool(id) => tools.get(id).map(|tool| tool.name.clone()),
        ItemKind::Block { id, .. } => blocks.get(id).map(|block| block.name.clone()),
    }
}

pub fn item_name_short_label(name: &str) -> String {
    name.split('_')
        .filter_map(|part| part.chars().next())
        .collect::<String>()
        .to_uppercase()
}
