//! Gameboard controller.

/// **********************************************************************
/// The `InputState` is exactly what it sounds like, it just keeps track of
/// the user's input state so that we turn keyboard events into something
/// state-based and device-independent.
/// **********************************************************************
#[derive(Debug)]
pub struct InputState {
    // Turn right and left axis.
    pub xaxis: f32,
    // Thruster axis or forward and backward.
    pub yaxis: f32,
    // FOV of the combined Sensor axis.
    pub fov_axis: f32,
    // Freeze all motor functions ;)
    pub freeze: f32,
    // Unleash the fire of hell upon your ennemy.
    pub fire: bool,
}

impl Default for InputState {
    fn default() -> Self {
        InputState {
            xaxis: 0.0,
            yaxis: 0.0,
            fov_axis: 0.0,
            freeze: 0.0,
            fire: false,
        }
    }
}
