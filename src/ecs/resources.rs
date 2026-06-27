use nightshade::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Title,
    Briefing,
    Playing,
    SectorClear,
    GameOver,
    Victory,
}

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

pub struct Enemy {
    pub entity: Entity,
    pub position: Vec3,
    pub health: i32,
    pub closing_speed: f32,
    pub lane_x: f32,
    pub lane_y: f32,
    pub sway_phase: f32,
    pub sway_amount: f32,
    pub fire_timer: f32,
    pub spin: f32,
}

pub struct Boss {
    pub entity: Entity,
    pub core: Option<Entity>,
    pub position: Vec3,
    pub health: i32,
    pub max_health: i32,
    pub fire_timer: f32,
    pub phase: f32,
    pub spin: f32,
    pub arrived: bool,
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

#[derive(Default, Clone, Copy)]
pub struct HudHandles {
    pub gameplay_panel: Option<Entity>,
    pub sector: Option<Entity>,
    pub shields: Option<Entity>,
    pub score: Option<Entity>,
    pub thrust: Option<Entity>,
    pub progress: Option<Entity>,
    pub boss: Option<Entity>,
    pub overlay_panel: Option<Entity>,
    pub overlay_heading: Option<Entity>,
    pub overlay_body: Option<Entity>,
    pub overlay_prompt: Option<Entity>,
    pub damage_flash: Option<Entity>,
}

pub struct GameState {
    pub ship: Option<Entity>,
    pub camera: Option<Entity>,
    pub exhaust: Option<Entity>,
    pub corner_thrusters: Vec<Entity>,
    pub backdrop: Vec<Backdrop>,
    pub moons: Vec<Moon>,
    pub ship_position: Vec3,
    pub roll: f32,
    pub pitch: f32,
    pub speed_scale: f32,
    pub elapsed: f32,
    pub barrel: BarrelRoll,
    pub scenery: Vec<Scenery>,
    pub bursts: Vec<(Entity, f32)>,
    pub projectiles: Vec<Projectile>,
    pub enemies: Vec<Enemy>,
    pub enemy_shots: Vec<Projectile>,
    pub boss: Option<Boss>,
    pub mode: GameMode,
    pub mode_timer: f32,
    pub sector: usize,
    pub distance: f32,
    pub sector_goal: f32,
    pub enemy_timer: f32,
    pub escort_timer: f32,
    pub boss_defeated: bool,
    pub shields: i32,
    pub max_shields: i32,
    pub damage_flash: f32,
    pub shake: f32,
    pub invuln: f32,
    pub hud: HudHandles,
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
            corner_thrusters: Vec::new(),
            backdrop: Vec::new(),
            moons: Vec::new(),
            ship_position: Vec3::new(0.0, 0.0, 0.0),
            roll: 0.0,
            pitch: 0.0,
            speed_scale: 1.0,
            elapsed: 0.0,
            barrel: BarrelRoll::default(),
            scenery: Vec::new(),
            bursts: Vec::new(),
            projectiles: Vec::new(),
            enemies: Vec::new(),
            enemy_shots: Vec::new(),
            boss: None,
            mode: GameMode::Title,
            mode_timer: 0.0,
            sector: 0,
            distance: 0.0,
            sector_goal: 0.0,
            enemy_timer: 0.0,
            escort_timer: 0.0,
            boss_defeated: false,
            shields: 4,
            max_shields: 4,
            damage_flash: 0.0,
            shake: 0.0,
            invuln: 0.0,
            hud: HudHandles::default(),
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
