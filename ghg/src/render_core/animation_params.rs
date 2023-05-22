use std::time::Duration;

use crate::render_core::viewport::Viewport;

#[derive(Clone)]
pub struct AnimationParams {
    pub viewport: Viewport,
    pub delta_time: Duration,
}
