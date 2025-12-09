use xplm::data::borrowed::DataRef;
use xplm::data::{DataRead, DataReadWrite, ReadWrite};

use crate::component::PluginComponent;
use crate::plugin::PluginError;

/// Fix copilot HSI when both HSI are in RNAV mode
pub(crate) struct CopilotHSI {
    is_initialized: bool,

    /// `sim/cockpit/switches/HSI_selector`
    hsi_selector: DataRef<i32>,

    /// `sim/cockpit/switches/HSI_selector2`
    hsi_selector2: DataRef<i32>,

    /// `sim/cockpit2/radios/actuators/hsi_obs_deg_mag_pilot`
    hsi_obs_deg_mag_pilot: DataRef<f32>,

    /// `sim/cockpit2/radios/actuators/hsi_obs_deg_mag_copilot`
    hsi_obs_deg_mag_copilot: DataRef<f32, ReadWrite>,

    /// `thranda/anim/hsiHdefDotsPilot`
    thranda_hsi_hdef_dots_pilot: Option<DataRef<f32>>,

    /// `thranda/anim/hsiHdefDotsCoPilot`
    thranda_hsi_hdef_dots_copilot: Option<DataRef<f32, ReadWrite>>,
}

impl CopilotHSI {
    pub(crate) fn new() -> Result<Self, PluginError> {
        let component = Self {
            is_initialized: false,

            hsi_selector: DataRef::find("sim/cockpit/switches/HSI_selector")?,
            hsi_selector2: DataRef::find("sim/cockpit/switches/HSI_selector2")?,
            hsi_obs_deg_mag_pilot: DataRef::find(
                "sim/cockpit2/radios/actuators/hsi_obs_deg_mag_pilot",
            )?,
            hsi_obs_deg_mag_copilot: DataRef::find(
                "sim/cockpit2/radios/actuators/hsi_obs_deg_mag_copilot",
            )?
            .writeable()?,
            thranda_hsi_hdef_dots_pilot: None,
            thranda_hsi_hdef_dots_copilot: None,
        };

        Ok(component)
    }

    /// Fetch SASL datarefs if they are available
    fn initialize(&mut self) -> Result<(), PluginError> {
        if self.thranda_hsi_hdef_dots_pilot.is_none() {
            self.thranda_hsi_hdef_dots_pilot =
                Some(DataRef::find("thranda/anim/hsiHdefDotsPilot")?);
        }

        if self.thranda_hsi_hdef_dots_copilot.is_none() {
            self.thranda_hsi_hdef_dots_copilot = Some(
                DataRef::find("thranda/anim/hsiHdefDotsCoPilot")?
                    .writeable()?,
            );
        }

        Ok(())
    }
}

impl PluginComponent for CopilotHSI {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    fn update(&mut self) {
        // We need to wait until all datarefs created by SASL are available...
        if !self.is_initialized {
            if self.initialize().is_ok() {
                self.is_initialized = true;
            } else {
                return;
            }
        }

        let hsi_selector = self.hsi_selector.get();
        let hsi_selector2 = self.hsi_selector2.get();
        let hsi_obs_deg_mag_pilot = self.hsi_obs_deg_mag_pilot.get();
        let hsi_obs_deg_mag_copilot = self.hsi_obs_deg_mag_copilot.get();
        let thranda_hsi_hdef_dots_pilot = self
            .thranda_hsi_hdef_dots_pilot
            .as_ref()
            .map_or(0.0, DataRead::get);
        let thranda_hsi_hdef_dots_copilot = self
            .thranda_hsi_hdef_dots_copilot
            .as_ref()
            .map_or(0.0, DataRead::get);

        // If both HSIs are in RNAV mode...
        if hsi_selector == 2 && hsi_selector2 == 2 {
            if !almost::equal(hsi_obs_deg_mag_pilot, hsi_obs_deg_mag_copilot) {
                self.hsi_obs_deg_mag_copilot.set(hsi_obs_deg_mag_pilot);
            }
            if !almost::equal(
                thranda_hsi_hdef_dots_pilot,
                thranda_hsi_hdef_dots_copilot,
            ) && let Some(thranda_hsi_hdef_dots_copilot) =
                self.thranda_hsi_hdef_dots_copilot.as_mut()
            {
                thranda_hsi_hdef_dots_copilot.set(thranda_hsi_hdef_dots_pilot);
            }
        }
    }
}
