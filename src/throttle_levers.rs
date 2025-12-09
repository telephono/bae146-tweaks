use xplm::data::borrowed::DataRef;
use xplm::data::{ArrayRead, ArrayReadWrite, ReadWrite};
use xplm::debugln;

use crate::component::PluginComponent;
use crate::plugin::PluginError;
use crate::plugin::{PLUGIN_NAME, SYNC_THROTTLES};

/// Align throttle lever 3 and 4 with throttle lever 2
pub(crate) struct ThrottleLevers {
    is_initialized: bool,

    /// `sim/cockpit2/engine/actuators/throttle_ratio`
    throttle_ratio: DataRef<[f32], ReadWrite>,
    throttle_ratio_slice: [f32; 4],
}

impl ThrottleLevers {
    pub(crate) fn new() -> Result<Self, PluginError> {
        let component = Self {
            is_initialized: false,

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
        self.is_initialized
    }

    fn update(&mut self) {
        if !self.is_initialized {
            self.is_initialized = true;
            debugln!("{PLUGIN_NAME} SyncThrottleLevers component initialized");
        }

        let sync_throttles = SYNC_THROTTLES.try_lock().is_ok_and(|lock| *lock);
        if sync_throttles {
            self.throttle_ratio.get(&mut self.throttle_ratio_slice);

            self.throttle_ratio_slice[2] = self.throttle_ratio_slice[1];
            self.throttle_ratio_slice[3] = self.throttle_ratio_slice[1];

            self.throttle_ratio.set(&self.throttle_ratio_slice);
        }
    }
}
