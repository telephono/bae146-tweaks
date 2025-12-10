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
    hydraulic_pressure_green: Option<DataRef<f32>>,

    /// `sim/operation/override/override_wheel_steer`
    override_wheel_steer: Option<DataRef<i32, ReadWrite>>,
}

impl NosewheelSteering {
    pub fn new() -> Self {
        Self {
            is_initialized: false,

            hydraulic_pressure_green: None,
            override_wheel_steer: None,
        }
    }

    fn initialize(&mut self) -> Result<(), PluginError> {
        if self.hydraulic_pressure_green.is_none() {
            self.hydraulic_pressure_green = Some(DataRef::find(
                "sim/cockpit2/hydraulics/indicators/hydraulic_pressure_2",
            )?);
        }

        if self.override_wheel_steer.is_none() {
            self.override_wheel_steer = Some(
                DataRef::find("sim/operation/override/override_wheel_steer")?
                    .writeable()?,
            );
        }

        Ok(())
    }
}

impl PluginComponent for NosewheelSteering {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    fn update(&mut self) {
        if !self.is_initialized {
            if self.initialize().is_ok() {
                self.is_initialized = true;
                debugln!(
                    "{PLUGIN_NAME} FixNosewheelSteering component initialized"
                );
            } else {
                return;
            }
        }

        if let Some(override_wheel_steer) = self.override_wheel_steer.as_mut()
            && let Some(hydraulic_pressure_green) =
                self.hydraulic_pressure_green.as_ref()
        {
            if hydraulic_pressure_green.get() > 100.0 {
                override_wheel_steer.set(1);
            } else {
                override_wheel_steer.set(0);
            }
        }
    }
}
