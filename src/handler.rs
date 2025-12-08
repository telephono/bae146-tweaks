use xplm::debugln;
use xplm::flight_loop::FlightLoopCallback;
use xplm::menu::{CheckHandler, CheckItem};

use crate::component::PluginComponent;
use crate::plugin::{PLUGIN_NAME, PluginError, SYNC_THROTTLES};

// Components
use crate::gpu::GeneratorVolts;
use crate::hsi::CopilotHSI;
use crate::nosewheel_steering::NosewheelSteering;
use crate::radio::Radio;
use crate::throttle_levers::ThrottleLevers;

pub(crate) struct FlightLoopHandler {
    initialization_done: bool,
    components: [Box<dyn PluginComponent>; 5],
}

impl FlightLoopHandler {
    pub(crate) fn new() -> Result<Self, PluginError> {
        let handler = Self {
            initialization_done: false,
            components: [
                Box::new(GeneratorVolts::new()?),
                Box::new(CopilotHSI::new()?),
                Box::new(NosewheelSteering::new()?),
                Box::new(Radio::new()?),
                Box::new(ThrottleLevers::new()?),
            ],
        };

        Ok(handler)
    }
}

impl FlightLoopCallback for FlightLoopHandler {
    fn flight_loop(&mut self, state: &mut xplm::flight_loop::LoopState<'_>) {
        for component in &mut self.components {
            component.update();
        }

        // We need to wait until all datarefs created by SASL are available...
        if !self.initialization_done {
            let initialization_done = self.components.iter().all(|comp| comp.is_initialized());
            if initialization_done {
                // Run flightloop callback on every flightloop from now on
                self.initialization_done = true;
                state.call_next_loop();
                debugln!("{PLUGIN_NAME} initialization complete");
            } else {
                debugln!("{PLUGIN_NAME} waiting for initialization...");
            }
        }
    }
}

pub(crate) struct SyncThrottlesMenuHandler;

impl CheckHandler for SyncThrottlesMenuHandler {
    fn item_checked(&mut self, _item: &CheckItem, checked: bool) {
        if let Ok(mut sync_throttles) = SYNC_THROTTLES.lock() {
            *sync_throttles = checked;
        }
    }
}
