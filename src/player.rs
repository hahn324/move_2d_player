use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use std::time::Duration;

const PLAYER_HITBOX: (Vec2, Vec2, f32) = (Vec2::new(0.0, -30.0), Vec2::new(0.0, -12.0), 11.0);

const PLAYER_SPRITE_GRID: (UVec2, u32, u32, Option<UVec2>, Option<UVec2>) =
    (UVec2::new(110, 80), 10, 2, Some(UVec2::new(10, 0)), None);

const IDLE_SPRITE_INDICES: (usize, usize) = (0, 9);
const IDLE_SPRITE_TIMER: f32 = 0.1;

const RUN_SPRITE_INDICES: (usize, usize) = (10, 19);
const RUN_SPRITE_TIMER: f32 = 0.05;

const PLAYER_SPEED: f32 = 500.0;
const GRAVITY: f32 = -250.0;

#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum PlayerDirection {
    #[default]
    Right,
    Left,
}

#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum PlayerState {
    #[default]
    Idle,
    Run,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<PlayerState>()
            .init_state::<PlayerDirection>()
            .add_systems(Startup, spawn_player)
            .add_systems(OnEnter(PlayerState::Idle), set_idle)
            .add_systems(OnEnter(PlayerState::Run), set_run)
            .add_systems(OnEnter(PlayerDirection::Right), set_sprite_right)
            .add_systems(OnEnter(PlayerDirection::Left), set_sprite_left)
            .add_systems(
                Update,
                (update_player_state, (animate_player, move_player)).chain(),
            );
    }
}

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_player(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut Sprite), With<Player>>,
) {
    if let Ok((indices, mut timer, mut sprite)) = query.single_mut() {
        timer.tick(time.delta());

        if timer.just_finished() {
            if let Some(atlas) = sprite.texture_atlas.as_mut() {
                atlas.index = if atlas.index == indices.last {
                    indices.first
                } else {
                    atlas.index + 1
                };
            }
        }
    }
}

#[derive(Component)]
pub struct Player;

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("player_sprite_sheet.png");
    let layout = TextureAtlasLayout::from_grid(
        PLAYER_SPRITE_GRID.0,
        PLAYER_SPRITE_GRID.1,
        PLAYER_SPRITE_GRID.2,
        PLAYER_SPRITE_GRID.3,
        PLAYER_SPRITE_GRID.4,
    );
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let animation_indices = AnimationIndices {
        first: IDLE_SPRITE_INDICES.0,
        last: IDLE_SPRITE_INDICES.1,
    };

    commands.spawn(Camera2d);

    commands.spawn((
        RigidBody::KinematicPositionBased,
        KinematicCharacterController {
            offset: CharacterLength::Absolute(0.02),
            ..default()
        },
        Sprite::from_atlas_image(
            texture,
            TextureAtlas {
                layout: texture_atlas_layout,
                index: animation_indices.first,
            },
        ),
        Transform::from_scale(Vec3::splat(3.0)),
        animation_indices,
        AnimationTimer(Timer::from_seconds(IDLE_SPRITE_TIMER, TimerMode::Repeating)),
        Player,
        Collider::capsule(PLAYER_HITBOX.0, PLAYER_HITBOX.1, PLAYER_HITBOX.2),
        GravityScale(1.0),
    ));
}

fn set_idle(
    mut query: Query<(&mut AnimationIndices, &mut AnimationTimer, &mut Sprite), With<Player>>,
) {
    if let Ok((mut animation_indices, mut animation_timer, mut sprite)) = query.single_mut() {
        animation_indices.first = IDLE_SPRITE_INDICES.0;
        animation_indices.last = IDLE_SPRITE_INDICES.1;

        animation_timer.set_duration(Duration::from_secs_f32(IDLE_SPRITE_TIMER));

        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            atlas.index = animation_indices.first;
        }
    }
}

fn set_run(
    mut query: Query<(&mut AnimationIndices, &mut AnimationTimer, &mut Sprite), With<Player>>,
) {
    if let Ok((mut animation_indices, mut animation_timer, mut sprite)) = query.single_mut() {
        animation_indices.first = RUN_SPRITE_INDICES.0;
        animation_indices.last = RUN_SPRITE_INDICES.1;

        animation_timer.set_duration(Duration::from_secs_f32(RUN_SPRITE_TIMER));

        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            atlas.index = animation_indices.first;
        }
    }
}

fn set_sprite_right(mut query: Query<&mut Sprite, With<Player>>) {
    if let Ok(mut sprite) = query.single_mut() {
        sprite.flip_x = false;
    }
}

fn set_sprite_left(mut query: Query<&mut Sprite, With<Player>>) {
    if let Ok(mut sprite) = query.single_mut() {
        sprite.flip_x = true;
    }
}

fn update_player_state(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_state: Res<State<PlayerState>>,
    mut next_player_state: ResMut<NextState<PlayerState>>,
    player_direction: Res<State<PlayerDirection>>,
    mut next_player_direction: ResMut<NextState<PlayerDirection>>,
) {
    let pressed_right = keyboard_input.just_pressed(KeyCode::ArrowRight);
    let released_right = keyboard_input.just_released(KeyCode::ArrowRight);

    let pressed_left = keyboard_input.just_pressed(KeyCode::ArrowLeft);
    let released_left = keyboard_input.just_released(KeyCode::ArrowLeft);

    match player_state.get() {
        PlayerState::Idle => {
            if (pressed_right && pressed_left) || (released_right && released_left) {
                return;
            } else if pressed_right || released_left {
                next_player_state.set(PlayerState::Run);
                match player_direction.get() {
                    PlayerDirection::Left => next_player_direction.set(PlayerDirection::Right),
                    PlayerDirection::Right => (),
                }
            } else if pressed_left || released_right {
                next_player_state.set(PlayerState::Run);
                match player_direction.get() {
                    PlayerDirection::Right => next_player_direction.set(PlayerDirection::Left),
                    PlayerDirection::Left => (),
                }
            }
        }
        PlayerState::Run => {
            if released_right && pressed_left {
                // Was Run Right, set to Run Left.
                next_player_direction.set(PlayerDirection::Left);
            } else if released_right || pressed_left {
                // Was Run Right, set to Idle Right.
                next_player_state.set(PlayerState::Idle);
            } else if released_left && pressed_right {
                // Was Run Left, set to Run Right.
                next_player_direction.set(PlayerDirection::Right);
            } else if released_left || pressed_right {
                // Was Run Left, set to Idle Left.
                next_player_state.set(PlayerState::Idle);
            }
        }
    }
}

fn move_player(
    mut query: Query<(&mut KinematicCharacterController, &GravityScale), With<Player>>,
    player_state: Res<State<PlayerState>>,
    player_direction: Res<State<PlayerDirection>>,
    time: Res<Time>,
) {
    if let Ok((mut controller, gravity_scale)) = query.single_mut() {
        let mut movement = Vec2::new(0.0, GRAVITY * gravity_scale.0);

        match player_state.get() {
            PlayerState::Run => match player_direction.get() {
                PlayerDirection::Right => movement.x += PLAYER_SPEED,
                PlayerDirection::Left => movement.x -= PLAYER_SPEED,
            },
            _ => (),
        }

        controller.translation = Some(movement * time.delta_secs());
    }
}
