//! The rules of the metal fences and the curved gates, as plain functions over small values.

/// What a block beside a metal fence is, as far as the fence's `canConnect` asks.
#[derive(Clone, Copy, Debug, Default)]
pub struct Neighbour {
    /// `Block.isExceptionForConnection`: leaves, barriers, pumpkins, melons, shulker boxes.
    pub exception: bool,
    /// `BlockState.isFaceSturdy` on the face toward the fence.
    pub sturdy: bool,
    /// A fence (`FenceBlock`, or the same fence kind).
    pub fence: bool,
    /// A wall.
    pub wall: bool,
    /// A fence gate.
    pub gate: bool,
    /// A hedge or grass-topped wall (`FenceHitbox`).
    pub hitbox: bool,
    /// A curved gate (`DoubleGate`).
    pub double_gate: bool,
}

/// `MetalFence.canConnect`.
pub fn connects(n: Neighbour) -> bool {
    (!n.exception && n.sturdy) || n.fence || n.wall || n.gate || n.hitbox || n.double_gate
}

/// `getConnectionStatus`: the part of a fence by the same fence above and below it. A fence with
/// nothing above is the top; a three-part fence (`middle`) with the same fence above and below is
/// its middle; the rest are the bottom.
pub fn fence_part(middle: bool, above: bool, below: bool) -> &'static str {
    if !above {
        "top"
    } else if middle && below {
        "middle"
    } else {
        "bottom"
    }
}

/// `DoubleGate.updateFenceGateState`: a gate with another above it is the bottom.
pub fn gate_part(above: bool) -> &'static str {
    if above { "bottom" } else { "top" }
}

/// `Direction.fromYRot` for a placer's yaw in degrees.
pub fn facing(yaw: f32) -> &'static str {
    match ((f64::from(yaw) / 90.0 + 0.5).floor() as i64).rem_euclid(4) {
        0 => "south",
        1 => "west",
        2 => "north",
        _ => "east",
    }
}

/// `DoubleGate.getHinge`: the side of the gate's cell a click lands on, given the placer's facing
/// and the click's offset inside the block.
pub fn hinge(facing: &str, x: f64, z: f64) -> &'static str {
    let (step_x, step_z) = match facing {
        "west" => (-1, 0),
        "east" => (1, 0),
        "north" => (0, -1),
        _ => (0, 1),
    };
    let right = (step_x < 0 && z < 0.5)
        || (step_x > 0 && z > 0.5)
        || (step_z < 0 && x > 0.5)
        || (step_z > 0 && x < 0.5);
    if right { "right" } else { "left" }
}

/// The value of `key` in a block state's text (`mod:block[a=b,c=d]`).
pub fn prop<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    let props = text.strip_suffix(']')?.split_once('[')?.1;
    props.split(',').find_map(|pair| {
        pair.split_once('=')
            .filter(|(k, _)| *k == key)
            .map(|(_, v)| v)
    })
}

/// The block's name in a state's text.
pub fn block(text: &str) -> &str {
    text.split_once('[').map_or(text, |(block, _)| block)
}

/// A state's text with `key` set to `value`.
pub fn with_prop(text: &str, key: &str, value: &str) -> Option<String> {
    let (block, props) = text.strip_suffix(']')?.split_once('[')?;
    let mut found = false;
    let props: Vec<String> = props
        .split(',')
        .map(|pair| match pair.split_once('=') {
            Some((k, _)) if k == key => {
                found = true;
                format!("{key}={value}")
            }
            _ => pair.to_owned(),
        })
        .collect();
    found.then(|| format!("{block}[{}]", props.join(",")))
}

/// A gate of the clicked kind.
pub struct GateCell {
    /// Whether it is open.
    pub open: bool,
    /// Which way it faces.
    pub facing: String,
}

/// The level, as the gate flood sees it.
pub trait GateWorld {
    /// The gate of the clicked block at `pos`, or `None` for any other block.
    fn gate(&self, pos: [i32; 3]) -> Option<GateCell>;
    /// Open or close the gate at `pos`.
    fn set_open(&mut self, pos: [i32; 3], open: bool);
}

/// `DoubleGate.fenceGate`: set `pos` to `open` if it is a gate facing `facing` in the other state,
/// then do the same for every cell around it, diagonals included.
pub fn toggle(world: &mut dyn GateWorld, pos: [i32; 3], open: bool, facing: &str) {
    let mut stack = vec![pos];
    while let Some([x, y, z]) = stack.pop() {
        let Some(cell) = world.gate([x, y, z]) else {
            continue;
        };
        if cell.open == open || cell.facing != facing {
            continue;
        }
        world.set_open([x, y, z], open);
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    stack.push([x + dx, y + dy, z + dz]);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn a_sturdy_face_connects_unless_the_block_is_an_exception() {
        let sturdy = Neighbour {
            sturdy: true,
            ..Neighbour::default()
        };
        assert!(connects(sturdy));
        assert!(!connects(Neighbour {
            exception: true,
            ..sturdy
        }));
        assert!(!connects(Neighbour::default()));
    }

    #[test]
    fn fences_walls_gates_and_hedges_connect_without_a_sturdy_face() {
        for n in [
            Neighbour {
                fence: true,
                ..Neighbour::default()
            },
            Neighbour {
                wall: true,
                ..Neighbour::default()
            },
            Neighbour {
                gate: true,
                ..Neighbour::default()
            },
            Neighbour {
                hitbox: true,
                exception: true,
                ..Neighbour::default()
            },
            Neighbour {
                double_gate: true,
                ..Neighbour::default()
            },
        ] {
            assert!(connects(n));
        }
    }

    #[test]
    fn a_fence_with_nothing_above_is_the_top() {
        assert_eq!(fence_part(false, false, true), "top");
        assert_eq!(fence_part(false, true, false), "bottom");
        assert_eq!(fence_part(false, true, true), "bottom");
        assert_eq!(fence_part(true, true, true), "middle");
        assert_eq!(fence_part(true, true, false), "bottom");
        assert_eq!(fence_part(true, false, true), "top");
    }

    #[test]
    fn a_gate_with_another_above_is_the_bottom() {
        assert_eq!(gate_part(true), "bottom");
        assert_eq!(gate_part(false), "top");
    }

    #[test]
    fn the_placer_faces_the_way_the_yaw_points() {
        assert_eq!(facing(0.0), "south");
        assert_eq!(facing(90.0), "west");
        assert_eq!(facing(180.0), "north");
        assert_eq!(facing(-90.0), "east");
        assert_eq!(facing(359.0), "south");
        assert_eq!(facing(44.0), "south");
        assert_eq!(facing(46.0), "west");
    }

    #[test]
    fn the_hinge_is_the_side_the_click_lands_on() {
        assert_eq!(hinge("north", 0.7, 0.2), "right");
        assert_eq!(hinge("north", 0.3, 0.2), "left");
        assert_eq!(hinge("south", 0.3, 0.8), "right");
        assert_eq!(hinge("east", 0.2, 0.9), "right");
        assert_eq!(hinge("west", 0.2, 0.9), "left");
    }

    #[test]
    fn state_text_reads_and_sets_a_property() {
        let text = "mcwfences:oak_curved_gate[facing=north,fencepart=top,hinge=left,open=false]";
        assert_eq!(prop(text, "open"), Some("false"));
        assert_eq!(prop(text, "missing"), None);
        assert_eq!(block(text), "mcwfences:oak_curved_gate");
        assert_eq!(
            with_prop(text, "open", "true").as_deref(),
            Some("mcwfences:oak_curved_gate[facing=north,fencepart=top,hinge=left,open=true]")
        );
        assert_eq!(with_prop("minecraft:stone", "open", "true"), None);
    }

    struct Grid(BTreeMap<[i32; 3], (bool, &'static str)>);

    impl GateWorld for Grid {
        fn gate(&self, pos: [i32; 3]) -> Option<GateCell> {
            self.0.get(&pos).map(|(open, facing)| GateCell {
                open: *open,
                facing: (*facing).to_owned(),
            })
        }
        fn set_open(&mut self, pos: [i32; 3], open: bool) {
            if let Some(cell) = self.0.get_mut(&pos) {
                cell.0 = open;
            }
        }
    }

    #[test]
    fn opening_a_gate_opens_the_connected_gates_that_face_the_same_way() {
        let mut grid = Grid(BTreeMap::from([
            ([0, 0, 0], (false, "north")),
            ([0, 1, 0], (false, "north")),
            ([1, 1, 1], (false, "north")),
            ([5, 0, 0], (false, "north")),
            ([0, 0, 1], (false, "east")),
        ]));
        toggle(&mut grid, [0, 0, 0], true, "north");
        assert!(grid.0[&[0, 0, 0]].0);
        assert!(grid.0[&[0, 1, 0]].0);
        assert!(grid.0[&[1, 1, 1]].0, "diagonals follow");
        assert!(!grid.0[&[5, 0, 0]].0, "too far");
        assert!(!grid.0[&[0, 0, 1]].0, "another facing");
        toggle(&mut grid, [0, 1, 0], false, "north");
        assert!(!grid.0[&[0, 0, 0]].0);
    }
}
