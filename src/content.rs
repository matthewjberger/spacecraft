use crate::systems::enemy_mesh;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    Drone,
    Fighter,
    Gunship,
}

pub struct EnemyStats {
    pub mesh: &'static str,
    pub scale: f32,
    pub radius: f32,
    pub health: i32,
    pub closing_speed: f32,
    pub fires: bool,
    pub fire_interval: f32,
    pub base_color: [f32; 4],
    pub emissive: [f32; 3],
}

impl EnemyKind {
    pub fn stats(self) -> EnemyStats {
        match self {
            EnemyKind::Drone => EnemyStats {
                mesh: enemy_mesh::DRONE_MESH,
                scale: 0.62,
                radius: 0.85,
                health: 1,
                closing_speed: 34.0,
                fires: false,
                fire_interval: 0.0,
                base_color: [0.26, 0.1, 0.3, 1.0],
                emissive: [0.5, 0.1, 0.65],
            },
            EnemyKind::Fighter => EnemyStats {
                mesh: enemy_mesh::FIGHTER_MESH,
                scale: 1.05,
                radius: 1.2,
                health: 2,
                closing_speed: 28.0,
                fires: true,
                fire_interval: 1.7,
                base_color: [0.22, 0.13, 0.16, 1.0],
                emissive: [0.55, 0.05, 0.07],
            },
            EnemyKind::Gunship => EnemyStats {
                mesh: enemy_mesh::GUNSHIP_MESH,
                scale: 1.9,
                radius: 2.0,
                health: 5,
                closing_speed: 17.0,
                fires: true,
                fire_interval: 1.25,
                base_color: [0.2, 0.16, 0.1, 1.0],
                emissive: [0.7, 0.35, 0.05],
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BossKind {
    Harvester,
    Warden,
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
            },
        }
    }
}

pub enum Beat {
    Field {
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
    pub beats: &'static [Beat],
}

pub const TAGLINE: &str =
    "The Drift swarm has overrun the belt.\nFly the corridor. Break the Monarch.";

pub const SECTORS: &[Sector] = &[
    Sector {
        name: "SECTOR I",
        subtitle: "THE VERGE",
        briefing: "The Drift has swallowed the outer belt.\nThread the debris, splash their scouts,\nand cut a corridor for the fleet.",
        beats: &[
            Beat::Breather { length: 130.0 },
            Beat::Field {
                length: 220.0,
                count: 15,
            },
            Beat::Rings { count: 5 },
            Beat::Wave {
                groups: &[(EnemyKind::Drone, 4), (EnemyKind::Fighter, 2)],
            },
            Beat::Field {
                length: 200.0,
                count: 13,
            },
            Beat::Wave {
                groups: &[(EnemyKind::Fighter, 3), (EnemyKind::Drone, 3)],
            },
            Beat::Rings { count: 4 },
            Beat::MiniBoss(BossKind::Harvester),
        ],
    },
    Sector {
        name: "SECTOR II",
        subtitle: "THE MAW",
        briefing: "Deeper in, the swarm packs the rock tight.\nThe old gates still hold a charge.\nPunch the line. The interceptors hit harder here.",
        beats: &[
            Beat::Field {
                length: 240.0,
                count: 22,
            },
            Beat::Wave {
                groups: &[(EnemyKind::Fighter, 4), (EnemyKind::Gunship, 1)],
            },
            Beat::Rings { count: 6 },
            Beat::Field {
                length: 220.0,
                count: 24,
            },
            Beat::Wave {
                groups: &[(EnemyKind::Drone, 6), (EnemyKind::Fighter, 3)],
            },
            Beat::MiniBoss(BossKind::Harvester),
            Beat::Breather { length: 120.0 },
            Beat::Boss(BossKind::Warden),
        ],
    },
    Sector {
        name: "SECTOR III",
        subtitle: "MONARCH'S APPROACH",
        briefing: "The Monarch coordinates the whole swarm.\nBurn through its escort, then break the core.\nThis is where it ends.",
        beats: &[
            Beat::Wave {
                groups: &[(EnemyKind::Fighter, 4), (EnemyKind::Gunship, 2)],
            },
            Beat::Field {
                length: 180.0,
                count: 16,
            },
            Beat::Wave {
                groups: &[(EnemyKind::Drone, 8), (EnemyKind::Fighter, 4)],
            },
            Beat::MiniBoss(BossKind::Warden),
            Beat::Breather { length: 110.0 },
            Beat::Boss(BossKind::Monarch),
        ],
    },
];
