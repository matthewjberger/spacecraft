pub const COMBO_WINDOW: f32 = 2.6;

pub const SHIP_SCALE: f32 = 0.42;
pub const SHIP_BASE_YAW: f32 = std::f32::consts::PI;
pub const BOUND_X: f32 = 11.0;
pub const BOUND_Y: f32 = 6.0;
pub const BASE_HEIGHT: f32 = 0.0;

pub const MAX_BANK: f32 = 0.85;
pub const MAX_PITCH: f32 = 0.40;
pub const MAX_LEAD_YAW: f32 = 0.25;
pub const IDLE_BOB: f32 = 0.12;

pub const LATERAL_ACCEL: f32 = 13.0;
pub const MAX_LATERAL_SPEED: f32 = 18.0;
pub const BANK_IN_RESPONSE: f32 = 13.0;
pub const BANK_OUT_RESPONSE: f32 = 3.2;
pub const EDGE_SHAKE: f32 = 0.3;

pub const CAMERA_BOOST_DOLLY: f32 = 2.2;
pub const CAMERA_LEAD: f32 = 0.6;
pub const CAMERA_KICK_DECAY: f32 = 7.0;
pub const FOV_POP_DECAY: f32 = 5.0;

pub const FIRE_KICK: f32 = 0.04;
pub const DAMAGE_KICK: f32 = 0.5;
pub const LASER_KICK: f32 = 0.35;
pub const NOVA_KICK: f32 = 0.6;
pub const FOV_POP_LASER: f32 = 5.0;
pub const FOV_POP_DAMAGE: f32 = 7.0;

pub const RECOIL_IMPULSE: f32 = 0.09;
pub const RECOIL_DECAY: f32 = 14.0;

pub const HITSTOP_BIG: f32 = 0.05;

pub const SHIELD_RADIUS: f32 = 1.85;
pub const SHIELD_PULSE: f32 = 0.05;
pub const SHIELD_SPIN: f32 = 0.5;

pub const BARREL_DURATION: f32 = 0.55;

pub const BOOST_GAIN: f32 = 1.4;
pub const BRAKE_GAIN: f32 = 0.5;
pub const SPEED_RESPONSE: f32 = 4.0;

pub const STICK_DEADZONE: f32 = 0.15;

pub const CAMERA_DISTANCE: f32 = 11.5;
pub const CAMERA_HEIGHT: f32 = 1.7;
pub const CAMERA_FOLLOW_X: f32 = 0.18;
pub const CAMERA_FOLLOW_Y: f32 = 0.16;
pub const CAMERA_RESPONSE: f32 = 3.8;
pub const FOV_RESPONSE: f32 = 9.0;
pub const CAMERA_PITCH: f32 = -0.155;
pub const CAMERA_ROLL: f32 = 0.30;
pub const BASE_FOV_DEGREES: f32 = 66.0;
pub const BOOST_FOV_DEGREES: f32 = 18.0;

pub const RAIL_SPEED: f32 = 38.0;
pub const SCENERY_DESPAWN_Z: f32 = 16.0;
pub const RING_RADIUS: f32 = 3.6;
pub const RING_SPACING: f32 = 34.0;
pub const RING_PULSE_AMOUNT: f32 = 0.07;
pub const RING_PULSE_SPEED: f32 = 1.7;
pub const RING_COLLECT_TIME: f32 = 0.45;
pub const RING_GROW: f32 = 1.8;
pub const RING_BOOST_TIME: f32 = 1.5;
pub const RING_BOOST_GAIN: f32 = 0.8;
pub const PATTERN_GAP: f32 = 30.0;
pub const BELT_MAX_ROCKS: usize = 165;
pub const ASTEROID_FIELD_X: f32 = 46.0;
pub const ASTEROID_FIELD_Y: f32 = 28.0;

pub const STAR_SPEED: f32 = 68.0;
pub const STARFIELD_CENTER_Z: f32 = -70.0;
pub const STARFIELD_HALF_X: f32 = 38.0;
pub const STARFIELD_HALF_Y: f32 = 24.0;
pub const STARFIELD_HALF_Z: f32 = 95.0;
pub const STARFIELD_RATE: f32 = 720.0;
pub const STAR_SIZE: f32 = 0.05;

pub const BURST_LIFETIME: f32 = 2.5;

pub const FIRE_INTERVAL: f32 = 0.11;
pub const PROJECTILE_SPEED: f32 = 95.0;
pub const PROJECTILE_RANGE: f32 = 175.0;
pub const PROJECTILE_HIT_RADIUS: f32 = 0.7;

pub const PLAYER_HIT_RADIUS: f32 = 0.6;
pub const ENEMY_SPAWN_AHEAD: f32 = 120.0;
pub const ENEMY_DESPAWN_Z: f32 = 16.0;
pub const ENEMY_SHOT_SPEED: f32 = 52.0;
pub const ENEMY_SCORE: u32 = 5;

pub const BOSS_SPAWN_Z: f32 = -80.0;

pub const COURSE_AHEAD: f32 = 210.0;
pub const PICKUP_COLLECT_RADIUS: f32 = 2.3;
pub const OVERDRIVE_FIRE_SCALE: f32 = 0.45;

pub const RETICLE_NEAR_Z: f32 = 11.0;
pub const RETICLE_MID_Z: f32 = 27.0;
pub const RETICLE_FAR_Z: f32 = 44.0;
pub const AIM_FAR_LEAD_X: f32 = 3.6;
pub const AIM_FAR_LEAD_Y: f32 = 3.0;

pub const LASER_DURATION: f32 = 1.1;
pub const LASER_COOLDOWN: f32 = 0.7;
pub const LASER_LENGTH: f32 = 135.0;
pub const LASER_SLICE_STRENGTH: f32 = 0.3;
pub const LASER_SLICE_RADIUS: f32 = 2.2;
pub const FRAGMENT_LIFE: f32 = 1.7;

pub const NOVA_RADIUS: f32 = 24.0;
pub const NOVA_RANGE_Z: f32 = 165.0;
pub const NOVA_FLASH_TIME: f32 = 0.32;
pub const NOVA_BOSS_DAMAGE: i32 = 12;
pub const AEGIS_DURATION: f32 = 2.6;
pub const AEGIS_COOLDOWN: f32 = 7.5;

pub const SEEKER_INTERVAL: f32 = 1.7;
pub const MISSILE_SPEED: f32 = 72.0;
pub const MISSILE_TURN: f32 = 4.5;
pub const MISSILE_LIFE: f32 = 3.0;
pub const MISSILE_HIT_RADIUS: f32 = 1.4;
pub const MISSILE_DAMAGE: i32 = 3;

pub const MAGNET_BASE_RANGE: f32 = 4.5;
pub const MAGNET_RANGE_PER: f32 = 4.5;
pub const MAGNET_PULL_SPEED: f32 = 18.0;
pub const ASTEROID_DROP_CHANCE: f32 = 0.18;

pub const BOSS_BEAM_INTERVAL: f32 = 5.5;
pub const BOSS_BEAM_DURATION: f32 = 1.4;
pub const BOSS_BEAM_CHARGE: f32 = 0.75;
pub const BOSS_BEAM_RADIUS: f32 = 1.9;

pub const DAMAGE_INVULN: f32 = 1.1;
pub const DAMAGE_FLASH_TIME: f32 = 0.35;
pub const DAMAGE_SHAKE: f32 = 0.9;
