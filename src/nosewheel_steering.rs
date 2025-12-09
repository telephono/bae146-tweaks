use xplm::data::borrowed::DataRef;
use xplm::data::{DataRead, DataReadWrite, ReadWrite};
use xplm::debugln;

use crate::component::PluginComponent;
use crate::plugin::PLUGIN_NAME;
use crate::plugin::PluginError;

/// UFMC sometimes blocks nosewheel steering...
/// This enables nosewheel steering as long as there is enough pressure
/// in the green system.
pub struct NosewheelSteering {
    is_initialized: bool,

    /// `sim/cockpit2/hydraulics/indicators/hydraulic_pressure_2`
    hydraulic_pressure_green: DataRef<f32>,

    /// `sim/operation/override/override_wheel_steer`
    override_wheel_steer: DataRef<i32, ReadWrite>,
}

impl NosewheelSteering {
    pub fn new() -> Result<Self, PluginError> {
        let component = Self {
            is_initialized: false,

            hydraulic_pressure_green: DataRef::find(
                "sim/cockpit2/hydraulics/indicators/hydraulic_pressure_2",
            )?,
            override_wheel_steer: DataRef::find(
                "sim/operation/override/override_wheel_steer",
            )?
            .writeable()?,
        };

        Ok(component)
    }
}

impl PluginComponent for NosewheelSteering {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    fn update(&mut self) {
        if !self.is_initialized {
            self.is_initialized = true;
            debugln!(
                "{PLUGIN_NAME} FixNosewheelSteering component initialized"
            );
        }

        if self.hydraulic_pressure_green.get() > 100.0 {
            self.override_wheel_steer.set(1);
        } else {
            self.override_wheel_steer.set(0);
        }
    }
}
