use xplm::data::borrowed::DataRef;
use xplm::data::{ArrayRead, ArrayReadWrite, ReadWrite};

use crate::component::PluginComponent;
use crate::plugin::{PluginError, SYNC_THROTTLES};

/// Align throttle lever 3 and 4 with throttle lever 2
pub(crate) struct ThrottleLevers {
    /// `sim/cockpit2/engine/actuators/throttle_ratio`
    throttle_ratio: DataRef<[f32], ReadWrite>,
    throttle_ratio_slice: [f32; 4],
}

impl ThrottleLevers {
    pub(crate) fn new() -> Result<Self, PluginError> {
        let component = Self {
            throttle_ratio: DataRef::find(
                "sim/cockpit2/engine/actuators/throttle_ratio",
            )?
            .writeable()?,
            throttle_ratio_slice: [0.0; 4],
        };

        Ok(component)
    }
}

impl PluginComponent for ThrottleLevers {
    fn is_initialized(&self) -> bool {
        true
    }

    fn update(&mut self) {
        let sync_throttles = SYNC_THROTTLES.try_lock().is_ok_and(|lock| *lock);
        if sync_throttles {
            self.throttle_ratio.get(&mut self.throttle_ratio_slice);

            self.throttle_ratio_slice[2] = self.throttle_ratio_slice[1];
            self.throttle_ratio_slice[3] = self.throttle_ratio_slice[1];

            self.throttle_ratio.set(&self.throttle_ratio_slice);
        }
    }
}
