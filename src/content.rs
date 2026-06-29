use crate::systems::enemy_mesh;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    Drone,
    Fighter,
    Gunship,
    Weaver,
    Lancer,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Behavior {
    Rusher,
    Strafer,
    Turret,
    Weaver,
    Diver,
}

pub struct EnemyStats {
    pub mesh: &'static str,
    pub scale: f32,
    pub radius: f32,
    pub health: i32,
    pub closing_speed: f32,
    pub fires: bool,
    pub fire_interval: f32,
    pub sway: f32,
    pub behavior: Behavior,
    pub hold_z: f32,
    pub base_color: [f32; 4],
    pub emissive: [f32; 3],
}

impl EnemyKind {
    pub fn stats(self) -> EnemyStats {
        match self {
            EnemyKind::Drone => EnemyStats {
                mesh: enemy_mesh::DRONE_MESH,
                scale: 0.8,
                radius: 1.0,
                health: 1,
                closing_speed: 40.0,
                fires: false,
                fire_interval: 0.0,
                sway: 1.0,
                behavior: Behavior::Rusher,
                hold_z: 0.0,
                base_color: [0.4, 0.08, 0.08, 1.0],
                emissive: [1.5, 0.22, 0.16],
            },
            EnemyKind::Fighter => EnemyStats {
                mesh: enemy_mesh::FIGHTER_MESH,
                scale: 1.05,
                radius: 1.2,
                health: 2,
                closing_speed: 30.0,
                fires: true,
                fire_interval: 1.6,
                sway: 1.0,
                behavior: Behavior::Strafer,
                hold_z: -42.0,
                base_color: [0.42, 0.1, 0.1, 1.0],
                emissive: [1.6, 0.18, 0.12],
            },
            EnemyKind::Gunship => EnemyStats {
                mesh: enemy_mesh::GUNSHIP_MESH,
                scale: 1.9,
                radius: 2.0,
                health: 5,
                closing_speed: 16.0,
                fires: true,
                fire_interval: 2.0,
                sway: 0.7,
                behavior: Behavior::Turret,
                hold_z: -60.0,
                base_color: [0.45, 0.2, 0.06, 1.0],
                emissive: [1.8, 0.5, 0.06],
            },
            EnemyKind::Weaver => EnemyStats {
                mesh: enemy_mesh::FIGHTER_MESH,
                scale: 0.85,
                radius: 1.0,
                health: 1,
                closing_speed: 38.0,
                fires: true,
                fire_interval: 1.9,
                sway: 2.6,
                behavior: Behavior::Weaver,
                hold_z: -36.0,
                base_color: [0.42, 0.1, 0.34, 1.0],
                emissive: [1.6, 0.2, 0.85],
            },
            EnemyKind::Lancer => EnemyStats {
                mesh: enemy_mesh::FIGHTER_MESH,
                scale: 1.1,
                radius: 1.1,
                health: 2,
                closing_speed: 30.0,
                fires: false,
                fire_interval: 0.0,
                sway: 1.0,
                behavior: Behavior::Diver,
                hold_z: -52.0,
                base_color: [0.5, 0.34, 0.05, 1.0],
                emissive: [2.0, 1.1, 0.1],
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BossKind {
    Harvester,
    Warden,
    Sentinel,
    Monarch,
}

pub struct BossStats {
    pub name: &'static str,
    pub mesh: &'static str,
    pub scale: f32,
    pub radius: f32,
    pub health: i32,
    pub base_color: [f32; 4],
    pub emissive: [f32; 3],
    pub fire_interval: f32,
    pub volley: usize,
    pub spread: f32,
    pub escort_interval: f32,
    pub escort: EnemyKind,
    pub hold_z: f32,
    pub approach_speed: f32,
    pub score: u32,
    pub beam: bool,
}

impl BossKind {
    pub fn stats(self) -> BossStats {
        match self {
            BossKind::Harvester => BossStats {
                name: "HARVESTER",
                mesh: enemy_mesh::HARVESTER_MESH,
                scale: 2.6,
                radius: 2.9,
                health: 28,
                base_color: [0.2, 0.12, 0.16, 1.0],
                emissive: [0.55, 0.1, 0.5],
                fire_interval: 1.6,
                volley: 3,
                spread: 4.2,
                escort_interval: 0.0,
                escort: EnemyKind::Drone,
                hold_z: -26.0,
                approach_speed: 24.0,
                score: 80,
                beam: false,
            },
            BossKind::Warden => BossStats {
                name: "WARDEN",
                mesh: enemy_mesh::WARDEN_MESH,
                scale: 3.6,
                radius: 3.9,
                health: 58,
                base_color: [0.14, 0.16, 0.21, 1.0],
                emissive: [0.2, 0.42, 0.72],
                fire_interval: 1.45,
                volley: 4,
                spread: 5.2,
                escort_interval: 3.4,
                escort: EnemyKind::Fighter,
                hold_z: -30.0,
                approach_speed: 22.0,
                score: 170,
                beam: true,
            },
            BossKind::Sentinel => BossStats {
                name: "SENTINEL",
                mesh: enemy_mesh::SENTINEL_MESH,
                scale: 3.0,
                radius: 3.2,
                health: 46,
                base_color: [0.16, 0.12, 0.24, 1.0],
                emissive: [0.4, 0.2, 0.95],
                fire_interval: 1.25,
                volley: 4,
                spread: 5.6,
                escort_interval: 3.0,
                escort: EnemyKind::Weaver,
                hold_z: -28.0,
                approach_speed: 26.0,
                score: 140,
                beam: true,
            },
            BossKind::Monarch => BossStats {
                name: "MONARCH",
                mesh: enemy_mesh::MONARCH_MESH,
                scale: 5.0,
                radius: 5.0,
                health: 98,
                base_color: [0.1, 0.08, 0.13, 1.0],
                emissive: [0.65, 0.05, 0.08],
                fire_interval: 1.35,
                volley: 5,
                spread: 6.2,
                escort_interval: 2.8,
                escort: EnemyKind::Fighter,
                hold_z: -32.0,
                approach_speed: 20.0,
                score: 320,
                beam: true,
            },
        }
    }
}

pub enum Beat {
    Field {
        length: f32,
        count: usize,
    },
    Belt {
        length: f32,
        density: usize,
    },
    Derelicts {
        length: f32,
        count: usize,
    },
    Rings {
        count: usize,
    },
    Wave {
        groups: &'static [(EnemyKind, usize)],
    },
    MiniBoss(BossKind),
    Boss(BossKind),
    Breather {
        length: f32,
    },
}

pub struct Sector {
    pub name: &'static str,
    pub subtitle: &'static str,
    pub briefing: &'static str,
    pub debrief: &'static str,
    pub beats: &'static [Beat],
}

pub const TAGLINE: &str = "The Drift swarm grew from our own dead colonies.\nTen thousand souls wait behind it — six hours of air.\nFly the last corridor. Break the Monarch. Let the fleet through.";

pub const SECTORS: &[Sector] = &[
    Sector {
        name: "SECTOR I",
        subtitle: "THE VERGE",
        briefing: "WREN: Scout nodes swarm the Verge — the corridor's closing.\nTALON: On your wing, Ranger. Mind the rocks and the crossfire.\nWREN: Punch through to the Harvester and put it down.",
        debrief: "WREN: Harvester's scrap. The corridor's holding, barely.\nTALON: It was stripping hulls for mass. Our hulls, Wren.\nWREN: Re-arm at the cache. The Maw runs deeper and meaner.",
        beats: &[
            Beat::Breather { length: 120.0 },
            Beat::Derelicts {
                length: 280.0,
                count: 5,
            },
            Beat::Belt {
                length: 300.0,
                density: 52,
            },
            Beat::Wave {
                groups: &[(EnemyKind::Drone, 5), (EnemyKind::Fighter, 2)],
            },
            Beat::Rings { count: 5 },
            Beat::Field {
                length: 200.0,
                count: 32,
            },
            Beat::Wave {
                groups: &[(EnemyKind::Fighter, 3), (EnemyKind::Drone, 3)],
            },
            Beat::Breather { length: 80.0 },
            Beat::Wave {
                groups: &[
                    (EnemyKind::Drone, 4),
                    (EnemyKind::Fighter, 2),
                    (EnemyKind::Lancer, 1),
                ],
            },
            Beat::Rings { count: 4 },
            Beat::MiniBoss(BossKind::Harvester),
        ],
    },
    Sector {
        name: "SECTOR II",
        subtitle: "THE MAW",
        briefing: "WREN: The swarm packs the rock solid through the Maw.\nTALON: Ranger, this wreckage — these are Tesse colony hulls.\nWREN: Cut the chatter. The Warden anchors this stretch. Break it.",
        debrief: "TALON: That signal under the static is a Tesse distress loop. Still live.\nWREN: The Monarch is the old colony core. It never stopped calling.\nWREN: Order stands. End the loop — clean.",
        beats: &[
            Beat::Belt {
                length: 340.0,
                density: 60,
            },
            Beat::Derelicts {
                length: 340.0,
                count: 6,
            },
            Beat::Wave {
                groups: &[(EnemyKind::Fighter, 4), (EnemyKind::Gunship, 1)],
            },
            Beat::Rings { count: 6 },
            Beat::Field {
                length: 220.0,
                count: 40,
            },
            Beat::Wave {
                groups: &[
                    (EnemyKind::Drone, 4),
                    (EnemyKind::Weaver, 3),
                    (EnemyKind::Lancer, 2),
                ],
            },
            Beat::Derelicts {
                length: 240.0,
                count: 4,
            },
            Beat::MiniBoss(BossKind::Harvester),
            Beat::Breather { length: 100.0 },
            Beat::Wave {
                groups: &[
                    (EnemyKind::Fighter, 3),
                    (EnemyKind::Gunship, 2),
                    (EnemyKind::Lancer, 2),
                ],
            },
            Beat::Boss(BossKind::Warden),
        ],
    },
    Sector {
        name: "SECTOR III",
        subtitle: "THE CROWN",
        briefing: "WREN: The Crown. The Monarch runs the entire Drift from here.\nTALON: Whatever it was, it's the swarm's heart now. Burn the escort.\nWREN: Then break the core. For the fleet. For Tesse. Go, Ranger.",
        debrief: "WREN: Core's coming apart. Hold the line, Ranger.\nTALON: Almost through — stay on it!\nWREN: All wings, the gap is opening.",
        beats: &[
            Beat::Wave {
                groups: &[
                    (EnemyKind::Fighter, 3),
                    (EnemyKind::Weaver, 3),
                    (EnemyKind::Gunship, 2),
                ],
            },
            Beat::Belt {
                length: 260.0,
                density: 50,
            },
            Beat::Derelicts {
                length: 260.0,
                count: 5,
            },
            Beat::Wave {
                groups: &[
                    (EnemyKind::Drone, 5),
                    (EnemyKind::Weaver, 4),
                    (EnemyKind::Lancer, 3),
                ],
            },
            Beat::Rings { count: 6 },
            Beat::MiniBoss(BossKind::Sentinel),
            Beat::Breather { length: 90.0 },
            Beat::Wave {
                groups: &[
                    (EnemyKind::Weaver, 4),
                    (EnemyKind::Gunship, 3),
                    (EnemyKind::Lancer, 2),
                ],
            },
            Beat::Field {
                length: 180.0,
                count: 28,
            },
            Beat::Boss(BossKind::Monarch),
        ],
    },
];
