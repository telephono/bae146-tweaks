use xplm::data::borrowed::DataRef;
use xplm::data::{DataRead, DataReadWrite, ReadWrite};
use xplm::debugln;

use crate::component::PluginComponent;
use crate::plugin::PLUGIN_NAME;
use crate::plugin::PluginError;

/// The current GPU/external power isn't compatible with X-Plane's
/// current GPU/external power implementation.
/// This corrects the supplied generator voltage...
#[allow(clippy::struct_field_names)]
pub struct GeneratorVolts {
    is_initialized: bool,

    /// `thranda/electrical/ExtPwrGPUAvailable`
    thranda_gpu_available: Option<DataRef<i32>>,

    /// `sim/cockpit2/electrical/GPU_generator_volts`
    gpu_generator_volts: Option<DataRef<f32, ReadWrite>>,

    /// `sim/operation/override/override_GPU_volts`
    override_gpu_volts: Option<DataRef<i32, ReadWrite>>,
}

impl GeneratorVolts {
    pub fn new() -> Self {
        Self {
            is_initialized: false,

            gpu_generator_volts: None,
            override_gpu_volts: None,
            thranda_gpu_available: None,
        }
    }

    /// Fetch SASL datarefs if they are available
    fn initialize(&mut self) -> Result<(), PluginError> {
        if self.gpu_generator_volts.is_none() {
            self.gpu_generator_volts = Some(
                DataRef::find("sim/cockpit2/electrical/GPU_generator_volts")?
                    .writeable()?,
            );
        }
        if self.override_gpu_volts.is_none() {
            self.override_gpu_volts = Some(
                DataRef::find("sim/operation/override/override_GPU_volts")?
                    .writeable()?,
            );
        }
        if self.thranda_gpu_available.is_none() {
            self.thranda_gpu_available =
                Some(DataRef::find("thranda/electrical/ExtPwrGPUAvailable")?);
        }

        Ok(())
    }
}

impl PluginComponent for GeneratorVolts {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    #[allow(clippy::collapsible_if)]
    fn update(&mut self) {
        // We need to wait until all datarefs created by SASL are available...
        if !self.is_initialized {
            if self.initialize().is_ok() {
                if let Some(override_gpu_volts) =
                    self.override_gpu_volts.as_mut()
                {
                    override_gpu_volts.set(1);
                }
                self.is_initialized = true;
                debugln!(
                    "{PLUGIN_NAME} FixGPUGeneratorVolts component initialized"
                );
            } else {
                return;
            }
        }

        let gpu_available =
            self.thranda_gpu_available.as_ref().map_or(0, DataRead::get);
        let gpu_generator_volts =
            self.gpu_generator_volts.as_ref().map_or(0.0, DataRef::get);

        // Set override GPU volts if BAe 146 GPU is connected
        if gpu_available == 1 && !almost::equal(gpu_generator_volts, 27.5) {
            if let Some(gpu_generator_volts) =
                self.gpu_generator_volts.as_mut()
            {
                gpu_generator_volts.set(27.5);
            }
        } else if gpu_available == 0 && !almost::zero(gpu_generator_volts) {
            if let Some(gpu_generator_volts) =
                self.gpu_generator_volts.as_mut()
            {
                gpu_generator_volts.set(0.0);
            }
        }
    }
}
