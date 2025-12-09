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
pub(crate) struct GeneratorVolts {
    is_initialized: bool,

    /// `thranda/electrical/ExtPwrGPUAvailable`
    thranda_gpu_available: Option<DataRef<i32>>,

    /// `sim/cockpit2/electrical/GPU_generator_volts`
    gpu_generator_volts: DataRef<f32, ReadWrite>,

    /// `sim/operation/override/override_GPU_volts`
    override_gpu_volts: DataRef<i32, ReadWrite>,
}

impl GeneratorVolts {
    pub(crate) fn new() -> Result<Self, PluginError> {
        let component = Self {
            is_initialized: false,

            gpu_generator_volts: DataRef::find(
                "sim/cockpit2/electrical/GPU_generator_volts",
            )?
            .writeable()?,
            override_gpu_volts: DataRef::find(
                "sim/operation/override/override_GPU_volts",
            )?
            .writeable()?,
            thranda_gpu_available: None,
        };

        Ok(component)
    }

    /// Fetch SASL datarefs if they are available
    fn initialize(&mut self) -> Result<(), PluginError> {
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

    fn update(&mut self) {
        // We need to wait until all datarefs created by SASL are available...
        if !self.is_initialized {
            if self.initialize().is_ok() {
                self.override_gpu_volts.set(1);
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
        let gpu_generator_volts = self.gpu_generator_volts.get();

        // Set override GPU volts if BAe 146 GPU is connected
        if gpu_available == 1 && !almost::equal(gpu_generator_volts, 27.5) {
            self.gpu_generator_volts.set(27.5);
        } else if gpu_available == 0 && !almost::zero(gpu_generator_volts) {
            self.gpu_generator_volts.set(0.0);
        }
    }
}
