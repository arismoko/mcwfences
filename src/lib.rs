//! Macaw's Fences and Walls for Sunrise: the fences, gates and walls are content in `data/` and
//! `assets/`; this code is the contact damage of the wired fences.
//!
//! | Moment | Where | What it does |
//! |---|---|---|
//! | `register` | [`declare`] | attaches [`Wired`] to the wired fences and the cheval de frise |
//! | `entity_inside` | [`Wired`] | hurts the entity inside with cactus damage |

use mod_api::{
    self, BlockPart, DamageTypeKey, EntityInsideEvent, EntityWrite, Hooks, InsideEffect,
    LevelRandomHost, Registrar, RegistrationContext,
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

/// Attach [`Wired`] to every hurting block.
pub fn declare(r: &mut dyn Registrar) {
    let cactus = r.damage_type_key("minecraft:cactus");
    for (id, amount) in hurting_blocks() {
        let block = r.block_key(&id);
        r.block_part(block, Wired { cactus, amount });
    }
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
    fn iron_hurts_twice_as_much() {
        let blocks = hurting_blocks();
        let damage = |id: &str| blocks.iter().find(|(b, _)| b == id).map(|(_, d)| *d);
        assert_eq!(damage("mcwfences:oak_wired_fence"), Some(2.0));
        assert_eq!(damage("mcwfences:wooden_cheval_de_frise"), Some(2.0));
        assert_eq!(damage("mcwfences:iron_cheval_de_frise"), Some(4.0));
    }
}
