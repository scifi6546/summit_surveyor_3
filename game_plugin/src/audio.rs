use crate::actions::Actions;
use crate::loading::AudioAssets;
use crate::GameState;
use bevy::prelude::*;

pub struct InternalAudioPlugin;

// This plugin is responsible to controll the game audio
impl Plugin for InternalAudioPlugin {
    fn build(&self, app: &mut App) {}
}
