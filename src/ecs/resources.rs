use crate::content::BossKind;
use nightshade::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Title,
    Settings,
    LevelSelect,
    Shop,
    Briefing,
    Cinematic,
    Playing,
    Paused,
    SectorClear,
    GameOver,
    Victory,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ModeKind {
    Story,
    Arcade,
    Endless,
}

#[derive(Default, Clone, Copy)]
pub struct ShipMods {
    pub hull: u8,
    pub rapid: u8,
    pub magnet: u8,
    pub seeker: u8,
    pub lance: u8,
    pub nova_max: u8,
    pub aegis: u8,
}

pub struct Missile {
    pub entity: Entity,
    pub position: Vec3,
    pub velocity: Vec3,
    pub life: f32,
}

pub struct Fragment {
    pub entity: Entity,
    pub position: Vec3,
    pub velocity: Vec3,
    pub spin_axis: Vec3,
    pub angle: f32,
    pub spin_speed: f32,
    pub life: f32,
    pub scale: f32,
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
    pub radius: f32,
    pub closing_speed: f32,
    pub fires: bool,
    pub fire_interval: f32,
    pub lane_x: f32,
    pub lane_y: f32,
    pub sway_phase: f32,
    pub sway_amount: f32,
    pub fire_timer: f32,
}

pub struct Boss {
    pub entity: Entity,
    pub kind: BossKind,
    pub position: Vec3,
    pub health: i32,
    pub max_health: i32,
    pub fire_timer: f32,
    pub phase: f32,
    pub spin: f32,
    pub arrived: bool,
    pub beam_timer: f32,
    pub firing: f32,
    pub aim_x: f32,
    pub aim_y: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PickupKind {
    Overdrive,
    Barrier,
    Spread,
    Nitrous,
}

impl PickupKind {
    pub fn label(self) -> &'static str {
        match self {
            PickupKind::Overdrive => "OVERDRIVE",
            PickupKind::Barrier => "BARRIER",
            PickupKind::Spread => "SPREAD SHOT",
            PickupKind::Nitrous => "NITROUS",
        }
    }

    pub fn tag(self) -> &'static str {
        match self {
            PickupKind::Overdrive => ">>>",
            PickupKind::Barrier => "( )",
            PickupKind::Spread => "<|>",
            PickupKind::Nitrous => "N2O",
        }
    }

    pub fn duration(self) -> f32 {
        match self {
            PickupKind::Overdrive => 8.0,
            PickupKind::Barrier => 7.0,
            PickupKind::Spread => 9.0,
            PickupKind::Nitrous => 3.5,
        }
    }

    pub fn color(self) -> Vec4 {
        match self {
            PickupKind::Overdrive => vec4(1.0, 0.78, 0.2, 1.0),
            PickupKind::Barrier => vec4(0.4, 0.9, 1.0, 1.0),
            PickupKind::Spread => vec4(1.0, 0.4, 0.9, 1.0),
            PickupKind::Nitrous => vec4(0.3, 1.0, 0.4, 1.0),
        }
    }

    pub fn emissive(self) -> [f32; 3] {
        match self {
            PickupKind::Overdrive => [1.0, 0.6, 0.1],
            PickupKind::Barrier => [0.2, 0.7, 1.0],
            PickupKind::Spread => [0.9, 0.2, 0.8],
            PickupKind::Nitrous => [0.2, 1.4, 0.35],
        }
    }
}

pub struct Pickup {
    pub entity: Entity,
    pub kind: PickupKind,
    pub position: Vec3,
    pub spin: f32,
    pub resolved: bool,
    pub terminal: Option<Entity>,
}

pub struct AllyShip {
    pub entity: Entity,
    pub position: Vec3,
    pub velocity: Vec3,
    pub timer: f32,
    pub phase: u8,
    pub slot: f32,
}

pub struct Structure {
    pub parts: Vec<(Entity, Vec3, Vec3)>,
    pub position: Vec3,
    pub spin_axis: Vec3,
    pub spin_speed: f32,
    pub angle: f32,
    pub drift: Vec3,
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
    pub score: Option<Entity>,
    pub combo: Option<Entity>,
    pub shields_bar: Option<Entity>,
    pub thrust_bar: Option<Entity>,
    pub approach_bar: Option<Entity>,
    pub boss_panel: Option<Entity>,
    pub boss_label: Option<Entity>,
    pub boss_bar: Option<Entity>,
    pub pickup_panel: Option<Entity>,
    pub pickup_label: Option<Entity>,
    pub pickup_time: Option<Entity>,
    pub pickup_bar: Option<Entity>,
    pub overlay_panel: Option<Entity>,
    pub overlay_heading: Option<Entity>,
    pub overlay_body: Option<Entity>,
    pub overlay_prompt: Option<Entity>,
    pub damage_flash: Option<Entity>,
    pub shop_panel: Option<Entity>,
    pub shop_credits: Option<Entity>,
    pub shop_lines: [Option<Entity>; 8],
    pub shop_prompt: Option<Entity>,
    pub ability_panel: Option<Entity>,
    pub lance_label: Option<Entity>,
    pub lance_bar: Option<Entity>,
    pub nova_label: Option<Entity>,
    pub aegis_label: Option<Entity>,
    pub aegis_bar: Option<Entity>,
    pub nova_flash: Option<Entity>,
    pub comms_panel: Option<Entity>,
    pub comms_avatar: Option<Entity>,
    pub comms_initial: Option<Entity>,
    pub comms_name: Option<Entity>,
    pub comms_text: Option<Entity>,
}

pub struct GameState {
    pub ship: Option<Entity>,
    pub ship_shined: bool,
    pub camera: Option<Entity>,
    pub cutscene_camera: Option<Entity>,
    pub cinematic_return: GameMode,
    pub exhaust: Option<Entity>,
    pub corner_thrusters: Vec<Entity>,
    pub blaster_ports: [Vec3; 4],
    pub backdrop: Vec<Backdrop>,
    pub moons: Vec<Moon>,
    pub ship_position: Vec3,
    pub roll: f32,
    pub pitch: f32,
    pub speed_scale: f32,
    pub ring_boost: f32,
    pub ship_vel: Vec3,
    pub cam_kick: f32,
    pub cam_fov_pop: f32,
    pub recoil: f32,
    pub hitstop: f32,
    pub elapsed: f32,
    pub barrel: BarrelRoll,
    pub scenery: Vec<Scenery>,
    pub bursts: Vec<(Entity, f32)>,
    pub projectiles: Vec<Projectile>,
    pub enemies: Vec<Enemy>,
    pub enemy_shots: Vec<Projectile>,
    pub missiles: Vec<Missile>,
    pub missile_timer: f32,
    pub pickups: Vec<Pickup>,
    pub allies: Vec<AllyShip>,
    pub structures: Vec<Structure>,
    pub effect: Option<PickupKind>,
    pub effect_timer: f32,
    pub effect_duration: f32,
    pub fragments: Vec<Fragment>,
    pub beam: Option<Entity>,
    pub laser_timer: f32,
    pub laser_cooldown: f32,
    pub nova_charges: u8,
    pub nova_flash: f32,
    pub aegis_timer: f32,
    pub aegis_cooldown: f32,
    pub boss: Option<Boss>,
    pub boss_beam: Option<Entity>,
    pub shield: Option<Entity>,
    pub mode: GameMode,
    pub mode_timer: f32,
    pub sector: usize,
    pub credits: u32,
    pub mods: ShipMods,
    pub run_mode: ModeKind,
    pub loop_count: u32,
    pub shop_cursor: usize,
    pub menu_cursor: usize,
    pub settings_cursor: usize,
    pub shake_enabled: bool,
    pub flash_enabled: bool,
    pub starfield_enabled: bool,
    pub crt_enabled: bool,
    pub hard_mode: bool,
    pub starfield: Option<Entity>,
    pub dais: Option<Entity>,
    pub reticle_near: [Option<Entity>; 8],
    pub reticle_far: Option<Entity>,
    pub menu_orbit: f32,
    pub beat_index: usize,
    pub beat_started: bool,
    pub beat_distance: f32,
    pub belt_accumulator: f32,
    pub escort_timer: f32,
    pub shields: i32,
    pub max_shields: i32,
    pub damage_flash: f32,
    pub shake: f32,
    pub invuln: f32,
    pub combo: u32,
    pub combo_timer: f32,
    pub best_combo: u32,
    pub best_score: u32,
    pub score_flash: f32,
    pub comms_line: String,
    pub comms_timer: f32,
    pub comms_low_warned: bool,
    pub hud: HudHandles,
    pub fire_cooldown: f32,
    pub next_turret: u8,
    pub solar_center: Vec3,
    pub score: u32,
    pub random_state: u64,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            ship: None,
            ship_shined: false,
            cutscene_camera: None,
            cinematic_return: GameMode::Playing,
            camera: None,
            exhaust: None,
            corner_thrusters: Vec::new(),
            blaster_ports: [Vec3::zeros(); 4],
            backdrop: Vec::new(),
            moons: Vec::new(),
            ship_position: Vec3::new(0.0, 0.0, 0.0),
            roll: 0.0,
            pitch: 0.0,
            speed_scale: 1.0,
            ring_boost: 0.0,
            ship_vel: Vec3::zeros(),
            cam_kick: 0.0,
            cam_fov_pop: 0.0,
            recoil: 0.0,
            hitstop: 0.0,
            elapsed: 0.0,
            barrel: BarrelRoll::default(),
            scenery: Vec::new(),
            bursts: Vec::new(),
            projectiles: Vec::new(),
            enemies: Vec::new(),
            enemy_shots: Vec::new(),
            missiles: Vec::new(),
            missile_timer: 0.0,
            pickups: Vec::new(),
            allies: Vec::new(),
            structures: Vec::new(),
            effect: None,
            effect_timer: 0.0,
            effect_duration: 0.0,
            fragments: Vec::new(),
            beam: None,
            laser_timer: 0.0,
            laser_cooldown: 0.0,
            nova_charges: 0,
            nova_flash: 0.0,
            aegis_timer: 0.0,
            aegis_cooldown: 0.0,
            boss: None,
            boss_beam: None,
            shield: None,
            mode: GameMode::Title,
            mode_timer: 0.0,
            sector: 0,
            credits: 0,
            mods: ShipMods::default(),
            run_mode: ModeKind::Story,
            loop_count: 0,
            shop_cursor: 0,
            menu_cursor: 0,
            settings_cursor: 0,
            shake_enabled: true,
            flash_enabled: true,
            starfield_enabled: true,
            crt_enabled: true,
            hard_mode: false,
            starfield: None,
            dais: None,
            reticle_near: [None; 8],
            reticle_far: None,
            menu_orbit: 0.0,
            beat_index: 0,
            beat_started: false,
            beat_distance: 0.0,
            belt_accumulator: 0.0,
            escort_timer: 0.0,
            shields: 4,
            max_shields: 4,
            damage_flash: 0.0,
            shake: 0.0,
            invuln: 0.0,
            combo: 0,
            combo_timer: 0.0,
            best_combo: 0,
            best_score: 0,
            score_flash: 0.0,
            comms_line: String::new(),
            comms_timer: 0.0,
            comms_low_warned: false,
            hud: HudHandles::default(),
            fire_cooldown: 0.0,
            next_turret: 0,
            solar_center: Vec3::new(0.0, 0.0, 0.0),
            score: 0,
            random_state: 0x9E37_79B9_7F4A_7C15,
        }
    }
}
