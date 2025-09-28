mod player;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use player::PlayerPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(PlayerPlugin)
        .add_systems(Startup, create_ground)
        .run();
}

fn create_ground(mut commands: Commands) {
    commands.spawn((
        Collider::cuboid(500.0, 50.0),
        Transform::from_xyz(0.0, -250.0, 0.0),
    ));
}
