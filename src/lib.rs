//! Macaw's Fences and Walls for Sunrise: the fences, gates and walls are content in `data/` and
//! `assets/`; this code is the contact damage of the wired fences, the metal fences' joining and the curved
//! gates.
//!
//! | Moment | Where | What it does |
//! |---|---|---|
//! | `register` | [`declare`] | attaches [`Wired`] to the wired fences and the cheval de frise |
//! | `entity_inside` | [`Wired`] | hurts the entity inside with cactus damage |
//! | `state_for_placement`, `update_shape` | [`MetalFence`] | which sides join, and the top, middle or bottom |
//! | `state_for_placement`, `update_shape` | [`CurvedGate`] | the facing, hinge and top or bottom |
//! | `use_item_on`, `use_without_item` | [`CurvedGate`] | opens or closes every touching gate that faces the same way |

pub mod fence;

use fence::{GateCell, GateWorld, Neighbour};
use mod_api::{
    self, BlockFace, BlockPart, BlockState, BlockUpdateFlags, DamageTypeKey, Direction,
    EntityInsideEvent, EntityWrite, Hooks, InsideEffect, InteractionResult, ItemKey, LevelHost,
    LevelRandomHost, LevelRef, LevelSide, LevelTicksHost, Registrar, RegistrationContext,
    ServerHost, SoundSource, StateForPlacementEvent, SupportType, UpdateShapeEvent, UseItemOnEvent,
    UseWithoutItemEvent,
};

/// The wood fences the original's `WiredFence` class builds: one per wood, plus the wooden
/// cheval de frise (`init/BlockInit.java`).
const WIRED_WOODS: [&str; 14] = [
    "oak",
    "spruce",
    "birch",
    "jungle",
    "acacia",
    "dark_oak",
    "crimson",
    "warped",
    "mangrove",
    "cherry",
    "bamboo",
    "pale_oak",
    "poplar",
    "wooden_cheval_de_frise",
];

/// `WiredFence.entityInside`: 2 cactus damage.
const WIRED_DAMAGE: f32 = 2.0;
/// `StoneWiredFence.entityInside`: 4 cactus damage.
const IRON_DAMAGE: f32 = 4.0;

/// The ids of the blocks that hurt, each with the damage it deals.
fn hurting_blocks() -> Vec<(String, f32)> {
    let mut blocks: Vec<(String, f32)> = WIRED_WOODS
        .iter()
        .map(|wood| {
            let name = if wood.ends_with("de_frise") {
                (*wood).to_string()
            } else {
                format!("{wood}_wired_fence")
            };
            (format!("mcwfences:{name}"), WIRED_DAMAGE)
        })
        .collect();
    blocks.push(("mcwfences:iron_cheval_de_frise".to_string(), IRON_DAMAGE));
    blocks
}

/// A fence that hurts whatever stands in it.
pub struct Wired {
    cactus: DamageTypeKey,
    amount: f32,
}

#[mod_api::sunrise_part]
impl BlockPart for Wired {
    fn entity_inside(&self, _level: &mut dyn LevelRandomHost, event: &mut EntityInsideEvent<'_>) {
        event.run_before(
            InsideEffect::Freeze,
            EntityWrite::Hurt {
                damage_type: self.cactus,
                amount: self.amount,
            },
        );
    }
}

/// The metal fences of `MetalFence`: a top and a bottom.
const METAL_FENCES: [&str; 5] = [
    "acorn_metal_fence",
    "guardian_metal_fence",
    "mesh_metal_fence",
    "ornate_metal_fence",
    "panelled_metal_fence",
];

/// The metal fences of `MetalFenceMiddle`: a top, a middle and a bottom.
const METAL_FENCES_WITH_MIDDLE: [&str; 9] = [
    "bastion_metal_fence",
    "cathedral_metal_fence",
    "curved_metal_fence",
    "double_curved_metal_fence",
    "expanded_mesh_metal_fence",
    "fortress_metal_fence",
    "gothic_metal_fence",
    "twilight_metal_fence",
    "vintage_metal_fence",
];

/// The woods that have a curved gate.
const CURVED_WOODS: [&str; 13] = [
    "oak", "spruce", "birch", "jungle", "acacia", "dark_oak", "crimson", "warped", "mangrove",
    "cherry", "bamboo", "pale_oak", "poplar",
];

/// The metal curved gates.
const METAL_GATES: [&str; 5] = [
    "bastion_metal_fence_gate",
    "curved_metal_fence_gate",
    "majestic_metal_fence_gate",
    "mesh_metal_fence_gate",
    "panelled_metal_fence_gate",
];

/// The four sides a fence joins, with the property that says it.
const SIDES: [(Direction, &str); 4] = [
    (Direction::North, "north"),
    (Direction::East, "east"),
    (Direction::South, "south"),
    (Direction::West, "west"),
];

fn face(direction: Direction) -> BlockFace {
    match direction {
        Direction::Down => BlockFace::Down,
        Direction::Up => BlockFace::Up,
        Direction::North => BlockFace::North,
        Direction::South => BlockFace::South,
        Direction::West => BlockFace::West,
        Direction::East => BlockFace::East,
    }
}

fn offset(pos: [i32; 3], direction: Direction) -> [i32; 3] {
    let [x, y, z] = direction.normal();
    [pos[0] + x, pos[1] + y, pos[2] + z]
}

fn in_tag(level: &dyn LevelHost, state: BlockState, tag: &str) -> bool {
    level.block_in_tag(state.block(), tag).unwrap_or(false)
}

/// Whether the state is air, which no metal fence joins.
fn is_air(level: &dyn LevelHost, state: BlockState) -> bool {
    matches!(
        level.state_block(state),
        Some("minecraft:air" | "minecraft:cave_air" | "minecraft:void_air")
    )
}

/// `MetalFence.canConnect` for the block at `at`, `toward` the fence from the fence's side.
fn joins(level: &dyn LevelHost, at: [i32; 3], state: BlockState, toward: Direction) -> bool {
    if is_air(level, state) {
        return false;
    }
    let exception = matches!(
        level.state_block(state),
        Some(
            "minecraft:barrier"
                | "minecraft:carved_pumpkin"
                | "minecraft:jack_o_lantern"
                | "minecraft:melon"
                | "minecraft:pumpkin"
        )
    ) || in_tag(level, state, "minecraft:leaves")
        || in_tag(level, state, "minecraft:shulker_boxes");
    let sturdy = level
        .block_face_sturdy(at, face(toward.opposite()), SupportType::Full)
        .unwrap_or(false);
    fence::connects(Neighbour {
        exception,
        sturdy,
        fence: in_tag(level, state, "minecraft:fences"),
        wall: in_tag(level, state, "minecraft:walls"),
        gate: in_tag(level, state, "minecraft:fence_gates"),
        hitbox: in_tag(level, state, "mcwfences:hedges")
            || in_tag(level, state, "mcwfences:grass_topped_walls"),
        double_gate: in_tag(level, state, "mcwfences:curved_double_gates")
            || in_tag(level, state, "mcwfences:metal_double_gates"),
    })
}

/// Whether the block beside `pos` in `direction` is `own`.
fn same_block(level: &dyn LevelHost, pos: [i32; 3], direction: Direction, own: BlockState) -> bool {
    level
        .block_state(offset(pos, direction))
        .is_some_and(|state| state.block() == own.block())
}

/// A metal fence: joins its neighbours on four sides and is a top, a bottom or, for some, a
/// middle (`MetalFence`, `MetalFenceMiddle`).
pub struct MetalFence {
    middle: bool,
}

impl MetalFence {
    fn part(&self, level: &dyn LevelHost, pos: [i32; 3], state: BlockState) -> Option<BlockState> {
        let above = same_block(level, pos, Direction::Up, state);
        let below = same_block(level, pos, Direction::Down, state);
        level.state_with_property(
            state,
            "fencepart",
            fence::fence_part(self.middle, above, below),
        )
    }
}

#[mod_api::sunrise_part]
impl BlockPart for MetalFence {
    /// `getStateForPlacement`: each side joins what `canConnect` allows, then the part.
    fn state_for_placement(
        &self,
        level: LevelSide,
        event: &mut StateForPlacementEvent<'_>,
    ) -> Option<Option<BlockState>> {
        let (LevelSide::Server(level) | LevelSide::Client(level)) = level;
        let p = event.context.pos;
        let pos = [p.x, p.y, p.z];
        let mut state = event.state;
        for (direction, name) in SIDES {
            let at = offset(pos, direction);
            let joined = level
                .block_state(at)
                .is_some_and(|neighbour| joins(level, at, neighbour, direction));
            state =
                level.state_with_property(state, name, if joined { "true" } else { "false" })?;
        }
        Some(self.part(level, pos, state))
    }

    /// `updateShape`: a side follows its neighbour; the part follows the fences above and below.
    fn update_shape(
        &self,
        level: &mut dyn LevelTicksHost,
        event: &mut UpdateShapeEvent,
    ) -> Option<BlockState> {
        let p = event.pos;
        let pos = [p.x, p.y, p.z];
        let level: &dyn LevelHost = level;
        let Some((_, name)) = SIDES.iter().find(|(d, _)| *d == event.direction) else {
            return self.part(level, pos, event.state);
        };
        let n = event.neighbour_pos;
        let joined = joins(level, [n.x, n.y, n.z], event.neighbour, event.direction);
        level.state_with_property(event.state, name, if joined { "true" } else { "false" })
    }
}

/// A curved gate (`DoubleGate`): faces the placer, hinges to the click's side, is a top or a
/// bottom, and opens with every touching gate that faces the same way.
pub struct CurvedGate {
    item: ItemKey,
}

/// The clicked gate's level, for the flood of `DoubleGate.fenceGate`.
struct Gates<'a> {
    host: &'a mut dyn ServerHost,
    level: LevelRef,
    block: &'a str,
}

impl GateWorld for Gates<'_> {
    fn gate(&self, pos: [i32; 3]) -> Option<GateCell> {
        let state = self.level.block_state(&*self.host, pos)?;
        let text = self.host.block_state_string(state)?;
        (fence::block(&text) == self.block).then(|| GateCell {
            open: fence::prop(&text, "open") == Some("true"),
            facing: fence::prop(&text, "facing").unwrap_or_default().to_owned(),
        })
    }

    fn set_open(&mut self, pos: [i32; 3], open: bool) {
        let Some(state) = self.level.block_state(&*self.host, pos) else {
            return;
        };
        let Some(text) = self.host.block_state_string(state) else {
            return;
        };
        let Some(next) = fence::with_prop(&text, "open", if open { "true" } else { "false" })
            .and_then(|text| self.host.block_state_id(&text))
        else {
            return;
        };
        let _ = self.level.set_block(
            self.host,
            pos,
            next,
            BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS,
        );
    }
}

fn play(
    host: &mut dyn ServerHost,
    level: LevelRef,
    sound: &str,
    pos: [i32; 3],
    volume: f32,
    pitch: f32,
) {
    let [x, y, z] = pos;
    let _ = level.play_sound(
        host,
        sound,
        [f64::from(x) + 0.5, f64::from(y) + 0.5, f64::from(z) + 0.5],
        SoundSource::Blocks,
        volume,
        pitch,
        None,
    );
}

impl CurvedGate {
    /// `DoubleGate.useItemOn` for a hand without the gate's own item.
    fn click(
        host: &mut dyn ServerHost,
        click: &mod_api::BlockClick<'_>,
    ) -> Option<InteractionResult> {
        let level = host.level(click.dimension.key().id())?;
        let state = level.block_state(&*host, click.pos)?;
        let text = host.block_state_string(state)?;
        let open = fence::prop(&text, "open")? != "true";
        let facing = fence::prop(&text, "facing")?.to_owned();
        let mut gates = Gates {
            host,
            level,
            block: fence::block(&text),
        };
        fence::toggle(&mut gates, click.pos, open, &facing);
        let sound = if open {
            "minecraft:block.wooden_door.open"
        } else {
            "minecraft:block.wooden_door.close"
        };
        play(host, level, sound, click.pos, 1.0, 1.0);
        Some(InteractionResult::Success)
    }
}

#[mod_api::sunrise_part]
impl BlockPart for CurvedGate {
    fn use_item_on(
        &self,
        host: &mut dyn ServerHost,
        event: &mut UseItemOnEvent<'_>,
    ) -> Option<InteractionResult> {
        let click = event.click;
        if click.stack.is_some_and(|s| s.item() == self.item) {
            let level = host.level(click.dimension.key().id())?;
            let unit = level
                .with_random(host, |random| random.next_float())
                .unwrap_or(0.5);
            play(
                host,
                level,
                "minecraft:block.wood.place",
                click.pos,
                0.5,
                unit * 0.1 + 0.8,
            );
            return Some(InteractionResult::Pass);
        }
        // `FenceGateBlock`'s shape: any other item tries the empty hand, which runs for the main hand only,
        // so a click that reaches the server for both hands toggles the gates once.
        Some(InteractionResult::TryWithEmptyHand)
    }

    fn use_without_item(
        &self,
        host: &mut dyn ServerHost,
        event: &mut UseWithoutItemEvent<'_>,
    ) -> Option<InteractionResult> {
        Self::click(host, event.click)
    }

    /// `getStateForPlacement` and `setPlacedBy`: facing the placer, the hinge by the click, or by
    /// the gate below; a top unless another gate is above.
    fn state_for_placement(
        &self,
        level: LevelSide,
        event: &mut StateForPlacementEvent<'_>,
    ) -> Option<Option<BlockState>> {
        let (LevelSide::Server(level) | LevelSide::Client(level)) = level;
        let context = event.context;
        let pos = [context.pos.x, context.pos.y, context.pos.z];
        let facing = fence::facing(context.yaw);
        let mut hinge = fence::hinge(
            facing,
            context.hit[0] - f64::from(pos[0]),
            context.hit[2] - f64::from(pos[2]),
        );
        let below = level
            .block_state(offset(pos, Direction::Down))
            .filter(|below| below.block() == event.state.block())
            .and_then(|below| level.state_string(below));
        if let Some(below_hinge) = below.as_deref().and_then(|text| fence::prop(text, "hinge")) {
            hinge = if below_hinge == "right" {
                "right"
            } else {
                "left"
            };
        }
        let above = same_block(level, pos, Direction::Up, event.state);
        let state = level.state_with_property(event.state, "facing", facing)?;
        let state = level.state_with_property(state, "hinge", hinge)?;
        Some(level.state_with_property(state, "fencepart", fence::gate_part(above)))
    }

    /// `updateShape`: the part follows the gate above.
    fn update_shape(
        &self,
        level: &mut dyn LevelTicksHost,
        event: &mut UpdateShapeEvent,
    ) -> Option<BlockState> {
        let level: &dyn LevelHost = level;
        if event.direction != Direction::Up {
            return Some(event.state);
        }
        let above = event.neighbour.block() == event.state.block();
        level.state_with_property(event.state, "fencepart", fence::gate_part(above))
    }
}

/// Attach [`Wired`] to every hurting block.
pub fn declare(r: &mut dyn Registrar) {
    let cactus = r.damage_type_key("minecraft:cactus");
    for (id, amount) in hurting_blocks() {
        let block = r.block_key(&id);
        r.block_part(block, Wired { cactus, amount });
    }
    for name in METAL_FENCES {
        let block = r.block_key(&format!("mcwfences:{name}"));
        r.block_part(block, MetalFence { middle: false });
    }
    for name in METAL_FENCES_WITH_MIDDLE {
        let block = r.block_key(&format!("mcwfences:{name}"));
        r.block_part(block, MetalFence { middle: true });
    }
    for name in curved_gates() {
        let block = r.block_key(&format!("mcwfences:{name}"));
        let item = r.item_key(&format!("mcwfences:{name}"));
        r.block_part(block, CurvedGate { item });
    }
}

/// The curved gates: one per wood and the metal ones.
fn curved_gates() -> Vec<String> {
    CURVED_WOODS
        .iter()
        .map(|wood| format!("{wood}_curved_gate"))
        .chain(METAL_GATES.iter().map(|name| (*name).to_owned()))
        .collect()
}

#[derive(Default)]
struct Mod;

#[mod_api::sunrise_mod]
impl Hooks for Mod {
    fn register(&self, r: &mut dyn Registrar, _: &RegistrationContext) {
        declare(r);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn every_hurting_block_is_declared_in_data() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/mcwfences/sunrise/block");
        let blocks = hurting_blocks();
        assert_eq!(blocks.len(), 15);
        for (id, _) in &blocks {
            let file = dir.join(format!("{}.json", id.trim_start_matches("mcwfences:")));
            assert!(file.exists(), "{id} has no declaration");
        }
    }

    #[test]
    fn every_fence_and_gate_with_a_part_is_declared_with_a_part_property() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/mcwfences/sunrise/block");
        let fences = METAL_FENCES
            .iter()
            .chain(METAL_FENCES_WITH_MIDDLE.iter())
            .map(|name| (*name).to_owned())
            .chain(curved_gates());
        let mut count = 0;
        for name in fences {
            let text = std::fs::read_to_string(dir.join(format!("{name}.json")))
                .unwrap_or_else(|_| panic!("{name} has no declaration"));
            assert!(text.contains("\"fencepart\""), "{name}");
            count += 1;
        }
        assert_eq!(count, 14 + 18);
    }

    #[test]
    fn iron_hurts_twice_as_much() {
        let blocks = hurting_blocks();
        let damage = |id: &str| blocks.iter().find(|(b, _)| b == id).map(|(_, d)| *d);
        assert_eq!(damage("mcwfences:oak_wired_fence"), Some(2.0));
        assert_eq!(damage("mcwfences:wooden_cheval_de_frise"), Some(2.0));
        assert_eq!(damage("mcwfences:iron_cheval_de_frise"), Some(4.0));
    }
}
