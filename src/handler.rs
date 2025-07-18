use xplm::command::Command;
use xplm::data::borrowed::DataRef;
use xplm::data::{ArrayRead, ArrayReadWrite, DataRead, DataReadWrite, ReadWrite};
use xplm::flight_loop::FlightLoopCallback;
use xplm::menu::{CheckHandler, CheckItem};

use crate::plugin::{PluginError, PLUGIN_NAME, SYNC_THROTTLES};

pub(crate) struct FlightLoopHandler {
    initialization_done: bool,
    onetime_actions_done: bool,

    /// sim/view/default_view
    default_view: Command,

    /// thranda/electrical/ExtPwrGPUAvailable
    thranda_gpu_available: Option<DataRef<i32>>,

    /// sim/cockpit2/electrical/GPU_generator_volts
    gpu_generator_volts: DataRef<f32, ReadWrite>,

    /// sim/operation/override/override_GPU_volts
    override_gpu_volts: DataRef<i32, ReadWrite>,

    /// sim/cockpit2/hydraulics/indicators/hydraulic_pressure_2
    hydraulic_pressure_green: DataRef<f32>,

    /// sim/operation/override/override_wheel_steer
    override_wheel_steer: DataRef<i32, ReadWrite>,

    /// sim/cockpit2/electrical/bus_volts
    bus_volts: DataRef<[f32]>,
    bus_volts_slice: [f32; 2],

    /// sim/cockpit2/radios/actuators/gps_power
    radio_gps1_power: DataRef<i32>,

    /// sim/cockpit2/radios/actuators/gps2_power
    radio_gps2_power: DataRef<i32>,

    /// sim/cockpit2/radios/actuators/com1_power
    radio_com1_power: DataRef<i32, ReadWrite>,

    /// sim/cockpit2/radios/actuators/com2_power
    radio_com2_power: DataRef<i32, ReadWrite>,

    /// thranda/generic/com1/genCom1Pwr
    thranda_radio_com1_power: Option<DataRef<i32>>,

    /// thranda/generic/com1/genCom2Pwr [sic!]
    thranda_radio_com2_power: Option<DataRef<i32>>,

    /// sim/cockpit2/engine/actuators/throttle_ratio
    throttle_ratio: DataRef<[f32], ReadWrite>,
    throttle_ratio_slice: [f32; 4],
}

impl FlightLoopHandler {
    pub(crate) fn new() -> Result<Self, PluginError> {
        let handler = Self {
            initialization_done: false,
            onetime_actions_done: false,

            default_view: Command::find("sim/view/default_view")?,

            thranda_gpu_available: None,
            gpu_generator_volts: DataRef::find("sim/cockpit2/electrical/GPU_generator_volts")?
                .writeable()?,
            override_gpu_volts: DataRef::find("sim/operation/override/override_GPU_volts")?
                .writeable()?,

            hydraulic_pressure_green: DataRef::find(
                "sim/cockpit2/hydraulics/indicators/hydraulic_pressure_2",
            )?,
            override_wheel_steer: DataRef::find("sim/operation/override/override_wheel_steer")?
                .writeable()?,

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

            throttle_ratio: DataRef::find("sim/cockpit2/engine/actuators/throttle_ratio")?
                .writeable()?,
            throttle_ratio_slice: [0.0; 4],
        };

        Ok(handler)
    }

    /// Fetch SASL datarefs if they are available
    fn initialize(&mut self) -> Result<(), PluginError> {
        if self.thranda_gpu_available.is_none() {
            self.thranda_gpu_available =
                Some(DataRef::find("thranda/electrical/ExtPwrGPUAvailable")?);
        }

        if self.thranda_radio_com1_power.is_none() {
            self.thranda_radio_com1_power = Some(DataRef::find("thranda/generic/com1/genCom1Pwr")?);
        }

        if self.thranda_radio_com2_power.is_none() {
            self.thranda_radio_com2_power = Some(DataRef::find("thranda/generic/com1/genCom2Pwr")?);
        }

        Ok(())
    }

    /// Run one-time actions
    fn onetime_actions(&mut self) {
        self.default_view.trigger();

        self.override_gpu_volts.set(1);
    }

    /// The current GPU/external power isn't compatible with X-Plane's current GPU/external power
    /// implementation. This corrects the supplied generator voltage...
    fn fix_gpu_generator_volts(&mut self) {
        let gpu_available = self.thranda_gpu_available.as_ref().map_or(0, |d| d.get());
        let gpu_generator_volts = self.gpu_generator_volts.get();

        // Set override GPU volts if BAe 146 GPU is connected
        if gpu_available == 1 && gpu_generator_volts != 27.5 {
            self.gpu_generator_volts.set(27.5);
        } else if gpu_available == 0 && gpu_generator_volts != 0.0 {
            self.gpu_generator_volts.set(0.0);
        }
    }

    /// UFMC sometimes blocks nosewheel steering...
    /// This enables nosewheel steering as long as there is enough pressure
    /// in the green system.
    fn fix_nosewheel_steering(&mut self) {
        if self.hydraulic_pressure_green.get() > 100.0 {
            self.override_wheel_steer.set(1);
        }
    }

    /// Fix radio power based on bus voltage available
    fn fix_radio(&mut self) {
        self.bus_volts.get(&mut self.bus_volts_slice);

        let thranda_radio_com1_power = self
            .thranda_radio_com1_power
            .as_ref()
            .map_or(0, |d| d.get());
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
            .map_or(0, |d| d.get());
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

    /// Align throttle lever 3 and 4 with throttle lever 2
    fn synchonize_throttle_levers(&mut self) {
        let sync_throttles = SYNC_THROTTLES.try_lock().is_ok_and(|l| *l);
        if sync_throttles {
            self.throttle_ratio.get(&mut self.throttle_ratio_slice);

            self.throttle_ratio_slice[2] = self.throttle_ratio_slice[1];
            self.throttle_ratio_slice[3] = self.throttle_ratio_slice[1];

            self.throttle_ratio.set(&self.throttle_ratio_slice);
        }
    }
}

impl FlightLoopCallback for FlightLoopHandler {
    fn flight_loop(&mut self, state: &mut xplm::flight_loop::LoopState) {
        // We need to wait until all datarefs created by SASL are available...
        if !self.initialization_done {
            if self.initialize().is_ok() {
                self.initialization_done = true;
                debugln!("{PLUGIN_NAME} initialization complete");
                return;
            } else {
                // Try again after the next interval...
                debugln!("{PLUGIN_NAME} waiting for initialization...");
                return;
            }
        }

        if !self.onetime_actions_done {
            self.onetime_actions_done = true;
            self.onetime_actions();
            debugln!("{PLUGIN_NAME} one-time actions complete");
        }

        self.fix_gpu_generator_volts();

        self.fix_nosewheel_steering();

        self.fix_radio();

        self.synchonize_throttle_levers();

        // Run flightloop callback on every flightloop from now on
        state.call_next_loop();
    }
}

pub(crate) struct SyncThrottlesMenuHandler;

impl CheckHandler for SyncThrottlesMenuHandler {
    fn item_checked(&mut self, item: &CheckItem, checked: bool) {
        if let Ok(mut sync_throttles) = SYNC_THROTTLES.lock() {
            debugln!(
                "{PLUGIN_NAME} SyncThrottlesMenuHandler: checked = {:?}, item = {:?}, sync_throttles = {:?}",
                checked,
                item.checked(),
                *sync_throttles,
            );

            if *sync_throttles != checked {
                *sync_throttles = checked;
            }
        }
    }
}
