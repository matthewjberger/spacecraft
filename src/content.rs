pub struct Sector {
    pub name: &'static str,
    pub subtitle: &'static str,
    pub briefing: &'static str,
    pub goal: f32,
    pub enemy_interval: f32,
    pub enemy_health: i32,
    pub enemy_speed: f32,
    pub boss: bool,
}

pub const TAGLINE: &str =
    "The Drift swarm has overrun the belt.\nFly the corridor. Break the Monarch.";

pub const SECTORS: &[Sector] = &[
    Sector {
        name: "SECTOR I",
        subtitle: "THE VERGE",
        briefing: "The Drift has swallowed the outer belt.\nThread the debris, splash their scouts,\nand cut a corridor for the fleet.",
        goal: 1500.0,
        enemy_interval: 3.2,
        enemy_health: 1,
        enemy_speed: 26.0,
        boss: false,
    },
    Sector {
        name: "SECTOR II",
        subtitle: "THE MAW",
        briefing: "Deeper in, the swarm packs the rock tight.\nThe old gates still hold a charge.\nPunch the line. The interceptors hit harder here.",
        goal: 1900.0,
        enemy_interval: 2.2,
        enemy_health: 2,
        enemy_speed: 30.0,
        boss: false,
    },
    Sector {
        name: "SECTOR III",
        subtitle: "MONARCH'S APPROACH",
        briefing: "The Monarch coordinates the whole swarm.\nBurn through its escort, then break the core.\nThis is where it ends.",
        goal: 620.0,
        enemy_interval: 1.6,
        enemy_health: 2,
        enemy_speed: 34.0,
        boss: true,
    },
];
