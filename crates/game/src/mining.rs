use engine_assets::{BlockRegistry, ToolRegistry};

use engine_world::BlockId;

pub const HAND_EFFICIENCY: f32 = 1.0;

pub fn mining_delta(sim_dt: f32, hardness: f32, tool_efficiency: f32, can_harvest: bool) -> f32 {
    if hardness < 0.0 {
        return 0.0;
    }
    if hardness == 0.0 {
        return 1.0;
    }
    let divisor = if can_harvest { 30.0 } else { 100.0 };
    (tool_efficiency / hardness.max(f32::EPSILON) / divisor) * (sim_dt * 20.0)
}

pub fn can_harvest_block(
    blocks: &BlockRegistry,
    tools: &ToolRegistry,
    block_id: BlockId,
    active_tool: Option<u16>,
) -> bool {
    if !blocks.requires_tool(block_id) {
        return true;
    }
    let Some(preferred) = blocks.preferred_tool(block_id) else {
        return true;
    };
    let Some(tool_id) = active_tool else {
        return false;
    };
    tools
        .get(tool_id)
        .is_some_and(|tool| tool.tool_class == preferred)
}

pub fn tool_efficiency(tools: &ToolRegistry, active_tool: Option<u16>) -> f32 {
    active_tool
        .and_then(|id| tools.get(id))
        .map(|tool| tool.efficiency)
        .unwrap_or(HAND_EFFICIENCY)
}

pub fn destroy_stage(progress: f32) -> u8 {
    ((progress * 10.0) as u8).min(9)
}

pub fn tool_label_for_inventory(
    inventory: &crate::components::PlayerInventory,
    tools: &ToolRegistry,
) -> String {
    inventory
        .active_tool()
        .and_then(|id| tools.get(id))
        .map(|tool| tool.name.clone())
        .unwrap_or_else(|| "hand".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_assets::{load_block_registry, load_tool_registry};
    use std::path::PathBuf;

    fn break_time_seconds(hardness: f32, tool_efficiency: f32, can_harvest: bool) -> f32 {
        if hardness < 0.0 {
            return f32::INFINITY;
        }
        if hardness == 0.0 {
            return 0.0;
        }
        let divisor = if can_harvest { 30.0 } else { 100.0 };
        hardness * divisor / tool_efficiency / 20.0
    }

    fn manifest_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    fn registries() -> (BlockRegistry, ToolRegistry) {
        let assets = manifest_dir().join("../../assets");
        let blocks = load_block_registry(&assets.join("blocks"));
        let tools = load_tool_registry(&assets.join("tools"));
        (blocks, tools)
    }

    #[test]
    fn dirt_by_hand_about_three_quarters_second() {
        let (blocks, tools) = registries();
        let dirt = blocks.id_by_name("dirt").unwrap();
        let hardness = blocks.hardness(dirt);
        let can_harvest = can_harvest_block(&blocks, &tools, dirt, None);
        let seconds = break_time_seconds(hardness, HAND_EFFICIENCY, can_harvest);
        assert!((seconds - 0.75).abs() < 0.05, "got {seconds}");
    }

    #[test]
    fn dirt_by_wooden_pick_about_point_four_seconds() {
        let (blocks, tools) = registries();
        let dirt = blocks.id_by_name("dirt").unwrap();
        let pick = tools.id_by_name("wooden_pickaxe").unwrap();
        let hardness = blocks.hardness(dirt);
        let eff = tool_efficiency(&tools, Some(pick));
        let can_harvest = can_harvest_block(&blocks, &tools, dirt, Some(pick));
        let seconds = break_time_seconds(hardness, eff, can_harvest);
        assert!((seconds - 0.375).abs() < 0.05, "got {seconds}");
    }

    #[test]
    fn stone_by_hand_about_seven_and_half_seconds() {
        let (blocks, tools) = registries();
        let stone = blocks.id_by_name("stone").unwrap();
        let hardness = blocks.hardness(stone);
        let can_harvest = can_harvest_block(&blocks, &tools, stone, None);
        assert!(!can_harvest);
        let seconds = break_time_seconds(hardness, HAND_EFFICIENCY, can_harvest);
        assert!((seconds - 7.5).abs() < 0.1, "got {seconds}");
    }

    #[test]
    fn stone_by_wooden_pick_about_one_point_one_seconds() {
        let (blocks, tools) = registries();
        let stone = blocks.id_by_name("stone").unwrap();
        let pick = tools.id_by_name("wooden_pickaxe").unwrap();
        let hardness = blocks.hardness(stone);
        let eff = tool_efficiency(&tools, Some(pick));
        let can_harvest = can_harvest_block(&blocks, &tools, stone, Some(pick));
        assert!(can_harvest);
        let seconds = break_time_seconds(hardness, eff, can_harvest);
        assert!((seconds - 1.125).abs() < 0.1, "got {seconds}");
    }
}
