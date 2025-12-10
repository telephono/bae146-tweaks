use xplm::data::borrowed::DataRef;
use xplm::data::{ArrayRead, DataRead, DataReadWrite, ReadWrite};
use xplm::debugln;

use crate::component::PluginComponent;
use crate::plugin::PLUGIN_NAME;
use crate::plugin::PluginError;

/// Fix radio power based on bus voltage available
#[allow(clippy::struct_field_names)]
pub struct Radio {
    is_initialized: bool,

    /// `sim/cockpit2/electrical/bus_volts`
    bus_volts: Option<DataRef<[f32]>>,
    bus_volts_slice: [f32; 2],

    /// `sim/cockpit2/radios/actuators/gps_power`
    radio_gps1_power: Option<DataRef<i32>>,

    /// `sim/cockpit2/radios/actuators/gps2_power`
    radio_gps2_power: Option<DataRef<i32>>,

    /// `sim/cockpit2/radios/actuators/com1_power`
    radio_com1_power: Option<DataRef<i32, ReadWrite>>,

    /// `sim/cockpit2/radios/actuators/com2_power`
    radio_com2_power: Option<DataRef<i32, ReadWrite>>,

    /// `thranda/generic/com1/genCom1Pwr`
    thranda_radio_com1_power: Option<DataRef<i32>>,

    /// `thranda/generic/com1/genCom2Pwr` [sic!]
    thranda_radio_com2_power: Option<DataRef<i32>>,
}

impl Radio {
    pub fn new() -> Self {
        Self {
            is_initialized: false,

            bus_volts: None,
            bus_volts_slice: [0.0; 2],
            radio_gps1_power: None,
            radio_gps2_power: None,
            radio_com1_power: None,
            radio_com2_power: None,
            thranda_radio_com1_power: None,
            thranda_radio_com2_power: None,
        }
    }

    /// Fetch SASL datarefs if they are available
    fn initialize(&mut self) -> Result<(), PluginError> {
        if self.bus_volts.is_none() {
            self.bus_volts =
                Some(DataRef::find("sim/cockpit2/electrical/bus_volts")?);
        }

        if self.radio_gps1_power.is_none() {
            self.radio_gps1_power = Some(DataRef::find(
                "sim/cockpit2/radios/actuators/gps_power",
            )?);
        }

        if self.radio_gps2_power.is_none() {
            self.radio_gps2_power = Some(DataRef::find(
                "sim/cockpit2/radios/actuators/gps2_power",
            )?);
        }

        if self.radio_com1_power.is_none() {
            self.radio_com1_power = Some(
                DataRef::find("sim/cockpit2/radios/actuators/com1_power")?
                    .writeable()?,
            );
        }

        if self.radio_com2_power.is_none() {
            self.radio_com2_power = Some(
                DataRef::find("sim/cockpit2/radios/actuators/com2_power")?
                    .writeable()?,
            );
        }

        if self.thranda_radio_com1_power.is_none() {
            self.thranda_radio_com1_power =
                Some(DataRef::find("thranda/generic/com1/genCom1Pwr")?);
        }

        if self.thranda_radio_com2_power.is_none() {
            self.thranda_radio_com2_power =
                Some(DataRef::find("thranda/generic/com1/genCom2Pwr")?);
        }

        Ok(())
    }
}

impl PluginComponent for Radio {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    fn update(&mut self) {
        // We need to wait until all datarefs created by SASL are available...
        if !self.is_initialized {
            if self.initialize().is_ok() {
                self.is_initialized = true;
                debugln!("{PLUGIN_NAME} FixRadioPower component initialized");
            } else {
                return;
            }
        }

        if let Some(bus_volts) = self.bus_volts.as_ref() {
            bus_volts.get(&mut self.bus_volts_slice);
        }

        let radio_com1_power =
            self.radio_com1_power.as_ref().map_or(0, DataRead::get);
        let radio_com2_power =
            self.radio_com2_power.as_ref().map_or(0, DataRead::get);
        let radio_gps1_power =
            self.radio_gps1_power.as_ref().map_or(0, DataRead::get);
        let radio_gps2_power =
            self.radio_gps2_power.as_ref().map_or(0, DataRead::get);
        let thranda_radio_com1_power = self
            .thranda_radio_com1_power
            .as_ref()
            .map_or(0, DataRead::get);
        let thranda_radio_com2_power = self
            .thranda_radio_com2_power
            .as_ref()
            .map_or(0, DataRead::get);

        if self.bus_volts_slice[0] > 21.0 && radio_gps1_power == 1 {
            if radio_com1_power != thranda_radio_com1_power
                && let Some(radio_com1_power) = self.radio_com1_power.as_mut()
            {
                radio_com1_power.set(thranda_radio_com1_power);
            }
        } else if radio_com1_power == 1
            && let Some(radio_com1_power) = self.radio_com1_power.as_mut()
        {
            radio_com1_power.set(0);
        }

        if self.bus_volts_slice[1] > 21.0 && radio_gps2_power == 1 {
            if radio_com2_power != thranda_radio_com2_power
                && let Some(radio_com2_power) = self.radio_com2_power.as_mut()
            {
                radio_com2_power.set(thranda_radio_com2_power);
            }
        } else if radio_com2_power == 1
            && let Some(radio_com2_power) = self.radio_com2_power.as_mut()
        {
            radio_com2_power.set(0);
        }
    }
}
