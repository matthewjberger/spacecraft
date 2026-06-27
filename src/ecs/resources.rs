use nightshade::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SceneryKind {
    Asteroid,
    Ring,
}

pub struct Scenery {
    pub entity: Entity,
    pub kind: SceneryKind,
    pub position: Vec3,
    pub spin_axis: Vec3,
    pub spin_speed: f32,
    pub angle: f32,
    pub radius: f32,
    pub resolved: bool,
    pub collected: bool,
    pub collect_timer: f32,
    pub pulse_phase: f32,
    pub material_name: String,
}

#[derive(Default)]
pub struct BarrelRoll {
    pub timer: f32,
    pub direction: f32,
}

pub struct Projectile {
    pub entity: Entity,
    pub position: Vec3,
    pub velocity: Vec3,
    pub age: f32,
}

pub struct Backdrop {
    pub entity: Entity,
    pub position: Vec3,
    pub radius: f32,
    pub atmosphere: Option<[f32; 3]>,
    pub orbit_radius: f32,
    pub orbit_angle: f32,
    pub orbit_speed: f32,
}

pub struct Moon {
    pub entity: Entity,
    pub parent: usize,
    pub radius: f32,
    pub orbit_radius: f32,
    pub orbit_angle: f32,
    pub orbit_speed: f32,
    pub tilt: f32,
}

pub struct GameState {
    pub ship: Option<Entity>,
    pub camera: Option<Entity>,
    pub exhaust: Option<Entity>,
    pub hud_score: Option<Entity>,
    pub hud_speed: Option<Entity>,
    pub corner_thrusters: Vec<Entity>,
    pub backdrop: Vec<Backdrop>,
    pub ship_position: Vec3,
    pub roll: f32,
    pub pitch: f32,
    pub speed_scale: f32,
    pub elapsed: f32,
    pub barrel: BarrelRoll,
    pub scenery: Vec<Scenery>,
    pub moons: Vec<Moon>,
    pub bursts: Vec<(Entity, f32)>,
    pub projectiles: Vec<Projectile>,
    pub ring_counter: u32,
    pub fire_cooldown: f32,
    pub next_turret: u8,
    pub frontier_z: f32,
    pub solar_center: Vec3,
    pub score: u32,
    pub random_state: u64,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            ship: None,
            camera: None,
            exhaust: None,
            hud_score: None,
            hud_speed: None,
            corner_thrusters: Vec::new(),
            backdrop: Vec::new(),
            ship_position: Vec3::new(0.0, 0.0, 0.0),
            roll: 0.0,
            pitch: 0.0,
            speed_scale: 1.0,
            elapsed: 0.0,
            barrel: BarrelRoll::default(),
            scenery: Vec::new(),
            moons: Vec::new(),
            bursts: Vec::new(),
            projectiles: Vec::new(),
            ring_counter: 0,
            fire_cooldown: 0.0,
            next_turret: 0,
            frontier_z: 0.0,
            solar_center: Vec3::new(0.0, 0.0, 0.0),
            score: 0,
            random_state: 0x9E37_79B9_7F4A_7C15,
        }
    }
}
