use xplm::data::borrowed::DataRef;
use xplm::data::{DataRead, DataReadWrite, ReadWrite};

use crate::component::PluginComponent;
use crate::plugin::PluginError;

/// UFMC sometimes blocks nosewheel steering...
/// This enables nosewheel steering as long as there is enough pressure
/// in the green system.
pub(crate) struct NosewheelSteering {
    /// `sim/cockpit2/hydraulics/indicators/hydraulic_pressure_2`
    hydraulic_pressure_green: DataRef<f32>,

    /// `sim/operation/override/override_wheel_steer`
    override_wheel_steer: DataRef<i32, ReadWrite>,
}

impl NosewheelSteering {
    pub(crate) fn new() -> Result<Self, PluginError> {
        let component = Self {
            hydraulic_pressure_green: DataRef::find(
                "sim/cockpit2/hydraulics/indicators/hydraulic_pressure_2",
            )?,
            override_wheel_steer: DataRef::find("sim/operation/override/override_wheel_steer")?
                .writeable()?,
        };

        Ok(component)
    }
}

impl PluginComponent for NosewheelSteering {
    fn is_initialized(&self) -> bool {
        true
    }

    fn update(&mut self) {
        if self.hydraulic_pressure_green.get() > 100.0 {
            self.override_wheel_steer.set(1);
        } else {
            self.override_wheel_steer.set(0);
        }
    }
}
