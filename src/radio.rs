use xplm::data::borrowed::DataRef;
use xplm::data::{ArrayRead, DataRead, DataReadWrite, ReadWrite};

use crate::component::PluginComponent;
use crate::plugin::PluginError;

/// Fix radio power based on bus voltage available
#[allow(clippy::struct_field_names)]
pub(crate) struct Radio {
    is_initialized: bool,

    /// `sim/cockpit2/electrical/bus_volts`
    bus_volts: DataRef<[f32]>,
    bus_volts_slice: [f32; 2],

    /// `sim/cockpit2/radios/actuators/gps_power`
    radio_gps1_power: DataRef<i32>,

    /// `sim/cockpit2/radios/actuators/gps2_power`
    radio_gps2_power: DataRef<i32>,

    /// `sim/cockpit2/radios/actuators/com1_power`
    radio_com1_power: DataRef<i32, ReadWrite>,

    /// `sim/cockpit2/radios/actuators/com2_power`
    radio_com2_power: DataRef<i32, ReadWrite>,

    /// `thranda/generic/com1/genCom1Pwr`
    thranda_radio_com1_power: Option<DataRef<i32>>,

    /// `thranda/generic/com1/genCom2Pwr` [sic!]
    thranda_radio_com2_power: Option<DataRef<i32>>,
}

impl Radio {
    pub(crate) fn new() -> Result<Self, PluginError> {
        let component = Self {
            is_initialized: false,

            bus_volts: DataRef::find("sim/cockpit2/electrical/bus_volts")?,
            bus_volts_slice: [0.0; 2],
            radio_gps1_power: DataRef::find("sim/cockpit2/radios/actuators/gps_power")?,
            radio_gps2_power: DataRef::find("sim/cockpit2/radios/actuators/gps2_power")?,
            radio_com1_power: DataRef::find("sim/cockpit2/radios/actuators/com1_power")?
                .writeable()?,
            radio_com2_power: DataRef::find("sim/cockpit2/radios/actuators/com2_power")?
                .writeable()?,
            thranda_radio_com1_power: None,
            thranda_radio_com2_power: None,
        };

        Ok(component)
    }

    /// Fetch SASL datarefs if they are available
    fn initialize(&mut self) -> Result<(), PluginError> {
        if self.thranda_radio_com1_power.is_none() {
            self.thranda_radio_com1_power = Some(DataRef::find("thranda/generic/com1/genCom1Pwr")?);
        }

        if self.thranda_radio_com2_power.is_none() {
            self.thranda_radio_com2_power = Some(DataRef::find("thranda/generic/com1/genCom2Pwr")?);
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
            } else {
                return;
            }
        }

        self.bus_volts.get(&mut self.bus_volts_slice);

        let thranda_radio_com1_power = self
            .thranda_radio_com1_power
            .as_ref()
            .map_or(0, DataRead::get);
        let radio_com1_power = self.radio_com1_power.get();
        let radio_gps1_power = self.radio_gps1_power.get();

        if self.bus_volts_slice[0] > 21.0 && radio_gps1_power == 1 {
            if radio_com1_power != thranda_radio_com1_power {
                self.radio_com1_power.set(thranda_radio_com1_power);
            }
        } else if radio_com1_power == 1 {
            self.radio_com1_power.set(0);
        }

        let thranda_radio_com2_power = self
            .thranda_radio_com2_power
            .as_ref()
            .map_or(0, DataRead::get);
        let radio_com2_power = self.radio_com2_power.get();
        let radio_gps2_power = self.radio_gps2_power.get();

        if self.bus_volts_slice[1] > 21.0 && radio_gps2_power == 1 {
            if radio_com2_power != thranda_radio_com2_power {
                self.radio_com2_power.set(thranda_radio_com2_power);
            }
        } else if radio_com2_power == 1 {
            self.radio_com2_power.set(0);
        }
    }
}
