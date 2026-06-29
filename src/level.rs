use crate::content::{Beat, BossKind, EnemyKind, Sector};
use crate::systems::common::next_random;

/// One encounter along the rail. Owned counterpart to the authored [`Beat`],
/// so segments can be produced procedurally rather than only from `'static`
/// tables.
#[derive(Clone)]
pub enum Segment {
    Field { length: f32, count: usize },
    Belt { length: f32, density: usize },
    Derelicts { length: f32, count: usize },
    Rings { count: usize },
    Wave { groups: Vec<(EnemyKind, usize)> },
    MiniBoss(BossKind),
    Boss(BossKind),
    Breather { length: f32 },
}

impl Segment {
    fn from_beat(beat: &Beat) -> Segment {
        match beat {
            Beat::Field { length, count } => Segment::Field {
                length: *length,
                count: *count,
            },
            Beat::Belt { length, density } => Segment::Belt {
                length: *length,
                density: *density,
            },
            Beat::Derelicts { length, count } => Segment::Derelicts {
                length: *length,
                count: *count,
            },
            Beat::Rings { count } => Segment::Rings { count: *count },
            Beat::Wave { groups } => Segment::Wave {
                groups: groups.to_vec(),
            },
            Beat::MiniBoss(kind) => Segment::MiniBoss(*kind),
            Beat::Boss(kind) => Segment::Boss(*kind),
            Beat::Breather { length } => Segment::Breather { length: *length },
        }
    }
}

/// Condition gating whether an edge can be taken when leaving a node. Lets the
/// flow branch on player performance, not just chance.
#[derive(Clone, Copy)]
pub enum EdgeGate {
    Always,
    ComboAtLeast(u32),
    ShieldsAtLeast(i32),
    ShieldsBelow(i32),
}

impl EdgeGate {
    fn passes(self, combo: u32, shields: i32) -> bool {
        match self {
            EdgeGate::Always => true,
            EdgeGate::ComboAtLeast(value) => combo >= value,
            EdgeGate::ShieldsAtLeast(value) => shields >= value,
            EdgeGate::ShieldsBelow(value) => shields < value,
        }
    }
}

/// A directed transition to another node. Eligible edges (gate satisfied) are
/// chosen by weighted random, so the same graph can yield varied runs.
#[derive(Clone, Copy)]
pub struct Edge {
    pub target: usize,
    pub gate: EdgeGate,
    pub weight: f32,
}

impl Edge {
    fn always(target: usize) -> Edge {
        Edge {
            target,
            gate: EdgeGate::Always,
            weight: 1.0,
        }
    }
}

#[derive(Clone)]
pub struct LevelNode {
    pub segment: Segment,
    pub edges: Vec<Edge>,
}

/// The flow of a whole level as a directed graph. Authored sectors compile to
/// a linear chain; the generator emits branching graphs.
#[derive(Clone, Default)]
pub struct LevelGraph {
    pub nodes: Vec<LevelNode>,
    pub start: usize,
}

impl LevelGraph {
    /// Compile an authored sector into a linear graph so the runtime only ever
    /// has to traverse one representation.
    pub fn from_sector(sector: &Sector) -> LevelGraph {
        let count = sector.beats.len();
        let nodes = sector
            .beats
            .iter()
            .enumerate()
            .map(|(index, beat)| {
                let edges = if index + 1 < count {
                    vec![Edge::always(index + 1)]
                } else {
                    Vec::new()
                };
                LevelNode {
                    segment: Segment::from_beat(beat),
                    edges,
                }
            })
            .collect();
        LevelGraph { nodes, start: 0 }
    }
}

/// Choose the next node leaving `edges`: filter by gate, then weighted random.
/// Returns `None` at a terminal node (level complete).
pub fn select_next(edges: &[Edge], combo: u32, shields: i32, rng: &mut u64) -> Option<usize> {
    let total: f32 = edges
        .iter()
        .filter(|edge| edge.gate.passes(combo, shields))
        .map(|edge| edge.weight)
        .sum();
    if total <= 0.0 {
        return None;
    }
    let mut roll = next_random(rng) * total;
    for edge in edges.iter().filter(|edge| edge.gate.passes(combo, shields)) {
        roll -= edge.weight;
        if roll <= 0.0 {
            return Some(edge.target);
        }
    }
    edges
        .iter()
        .rev()
        .find(|edge| edge.gate.passes(combo, shields))
        .map(|edge| edge.target)
}

fn enemy_palette(tier: u32) -> Vec<EnemyKind> {
    let mut palette = vec![EnemyKind::Drone, EnemyKind::Fighter];
    if tier >= 2 {
        palette.push(EnemyKind::Weaver);
        palette.push(EnemyKind::Lancer);
    }
    if tier >= 4 {
        palette.push(EnemyKind::Gunship);
    }
    palette
}

pub fn boss_for(tier: u32) -> BossKind {
    match tier % 4 {
        0 => BossKind::Harvester,
        1 => BossKind::Warden,
        2 => BossKind::Sentinel,
        _ => BossKind::Monarch,
    }
}

fn wave_segment(rng: &mut u64, tier: u32, intensity: usize) -> Segment {
    let palette = enemy_palette(tier);
    let kinds = 2 + (next_random(rng) * 2.0) as usize;
    let mut budget = 4 + intensity + tier as usize;
    let mut groups: Vec<(EnemyKind, usize)> = Vec::new();
    for slot in 0..kinds {
        let remaining_slots = kinds - slot;
        let amount = if remaining_slots == 1 {
            budget
        } else {
            (1 + (next_random(rng) * (budget / remaining_slots) as f32) as usize).max(1)
        };
        let amount = amount
            .min(budget.saturating_sub(remaining_slots - 1))
            .max(1);
        budget -= amount;
        let kind = palette[(next_random(rng) * palette.len() as f32) as usize % palette.len()];
        groups.push((kind, amount));
        if budget == 0 {
            break;
        }
    }
    Segment::Wave { groups }
}

fn traversal_segment(rng: &mut u64, tier: u32) -> Segment {
    let roll = next_random(rng);
    if roll < 0.34 {
        Segment::Field {
            length: 180.0 + next_random(rng) * 120.0,
            count: 24 + tier as usize * 4 + (next_random(rng) * 16.0) as usize,
        }
    } else if roll < 0.64 {
        Segment::Belt {
            length: 260.0 + next_random(rng) * 120.0,
            density: 46 + tier as usize * 4 + (next_random(rng) * 18.0) as usize,
        }
    } else if roll < 0.85 {
        Segment::Derelicts {
            length: 220.0 + next_random(rng) * 120.0,
            count: 4 + (next_random(rng) * 3.0) as usize,
        }
    } else {
        Segment::Rings {
            count: 5 + (next_random(rng) * 3.0) as usize,
        }
    }
}

/// Procedurally generate a level graph: an eased intro, alternating
/// traversal/combat phases that escalate with `difficulty`, a mid-level
/// mini-boss, a performance-gated harder detour, and a terminal boss.
pub fn generate(seed: u64, difficulty: u32, boss: BossKind) -> LevelGraph {
    let mut rng = seed | 1;
    let mut spine: Vec<Segment> = Vec::new();
    spine.push(Segment::Breather { length: 90.0 });

    let phases = 4 + difficulty.min(5) as usize;
    let mid = phases / 2;
    for phase in 0..phases {
        let tier = difficulty + phase as u32 / 2;
        spine.push(traversal_segment(&mut rng, tier));
        spine.push(wave_segment(&mut rng, tier, phase));
        if phase % 2 == 1 {
            spine.push(Segment::Rings {
                count: 4 + (next_random(&mut rng) * 3.0) as usize,
            });
        }
        if phase == mid {
            spine.push(Segment::MiniBoss(boss_for(difficulty + 1)));
            spine.push(Segment::Breather { length: 80.0 });
        }
    }
    spine.push(Segment::Boss(boss));

    build_graph(spine, &mut rng, difficulty)
}

/// Wire a spine of segments into a graph and splice in one combo-gated detour:
/// a tougher bonus wave that skilled players are routed through before
/// rejoining the main flow.
fn build_graph(spine: Vec<Segment>, rng: &mut u64, difficulty: u32) -> LevelGraph {
    let count = spine.len();
    let mut nodes: Vec<LevelNode> = spine
        .into_iter()
        .enumerate()
        .map(|(index, segment)| {
            let edges = if index + 1 < count {
                vec![Edge::always(index + 1)]
            } else {
                Vec::new()
            };
            LevelNode { segment, edges }
        })
        .collect();

    let branchable: Vec<usize> = (1..nodes.len().saturating_sub(1))
        .filter(|&index| matches!(nodes[index].segment, Segment::Wave { .. }))
        .collect();
    if let Some(&fork) = branchable.get(branchable.len() / 2) {
        let detour = LevelNode {
            segment: wave_segment(rng, difficulty + 3, 6),
            edges: vec![Edge::always(fork)],
        };
        let detour_index = nodes.len();
        nodes.push(detour);
        nodes[fork - 1].edges = vec![
            Edge {
                target: detour_index,
                gate: EdgeGate::ComboAtLeast(12),
                weight: 2.0,
            },
            Edge::always(fork),
        ];
    }

    LevelGraph { nodes, start: 0 }
}
