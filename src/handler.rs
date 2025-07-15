use xplm::command::Command;

#[cfg(any(feature = "fixtms", feature = "syncthrottle"))]
use xplm::data::ArrayReadWrite;

#[cfg(any(feature = "fixradio", feature = "fixtms", feature = "syncthrottle"))]
use xplm::data::ArrayRead;

#[cfg(any(feature = "fixgpu", feature = "fixnws", feature = "fixradio"))]
use xplm::data::{DataRead, DataReadWrite};

#[cfg(any(
    feature = "fixgpu",
    feature = "fixnws",
    feature = "fixradio",
    feature = "fixtms",
    feature = "syncthrottle"
))]
use xplm::data::{borrowed::DataRef, ReadWrite};

use xplm::flight_loop::FlightLoopCallback;

use crate::plugin::{PluginError, PLUGIN_NAME};

pub(crate) struct FlightLoopHandler {
    initialization_done: bool,
    onetime_actions_done: bool,

    default_view: Command,

    #[cfg(feature = "fixautothrottle")]
    /// sim/autopilot/autothrottle_hard_off
    autothrottle_hard_off: Command,

    #[cfg(feature = "fixgpu")]
    /// thranda/electrical/ExtPwrGPUAvailable
    thranda_gpu_available: Option<DataRef<i32>>,
    #[cfg(feature = "fixgpu")]
    /// sim/cockpit2/electrical/GPU_generator_volts
    gpu_generator_volts: DataRef<f32, ReadWrite>,
    #[cfg(feature = "fixgpu")]
    /// sim/operation/override/override_GPU_volts
    override_gpu_volts: DataRef<i32, ReadWrite>,

    #[cfg(feature = "fixnws")]
    /// sim/cockpit2/hydraulics/indicators/hydraulic_pressure_2
    hydraulic_pressure_green: DataRef<f32>,
    #[cfg(feature = "fixnws")]
    /// sim/operation/override/override_wheel_steer
    override_wheel_steer: DataRef<i32, ReadWrite>,

    #[cfg(feature = "fixradio")]
    /// sim/cockpit2/electrical/bus_volts
    bus_volts: DataRef<[f32]>,
    #[cfg(feature = "fixradio")]
    bus_volts_slice: [f32; 2],
    #[cfg(feature = "fixradio")]
    /// sim/cockpit2/radios/actuators/gps_power
    radio_gps1_power: DataRef<i32>,
    #[cfg(feature = "fixradio")]
    /// sim/cockpit2/radios/actuators/gps2_power
    radio_gps2_power: DataRef<i32>,
    #[cfg(feature = "fixradio")]
    /// sim/cockpit2/radios/actuators/com1_power
    radio_com1_power: DataRef<i32, ReadWrite>,
    #[cfg(feature = "fixradio")]
    /// sim/cockpit2/radios/actuators/com2_power
    radio_com2_power: DataRef<i32, ReadWrite>,
    #[cfg(feature = "fixradio")]
    /// thranda/generic/com1/genCom1Pwr
    thranda_radio_com1_power: Option<DataRef<i32>>,
    #[cfg(feature = "fixradio")]
    /// thranda/generic/com1/genCom2Pwr [sic!]
    thranda_radio_com2_power: Option<DataRef<i32>>,

    #[cfg(feature = "fixtms")]
    /// thranda/engine/TGT_C_Act
    thranda_tgt_c_act: Option<DataRef<[f32], ReadWrite>>,
    #[cfg(feature = "fixtms")]
    thranda_tgt_c_act_slice: [f32; 3],

    #[cfg(feature = "syncthrottle")]
    throttle_levers: [f32; 4],
    #[cfg(feature = "syncthrottle")]
    /// sim/cockpit2/engine/actuators/throttle_ratio
    throttle_ratio: DataRef<[f32], ReadWrite>,
}

impl FlightLoopHandler {
    pub(crate) fn new() -> Result<Self, PluginError> {
        let handler = Self {
            initialization_done: false,
            onetime_actions_done: false,

            default_view: Command::find("sim/view/default_view")?,

            #[cfg(feature = "fixautothrottle")]
            autothrottle_hard_off: Command::find("sim/autopilot/autothrottle_hard_off")?,

            #[cfg(feature = "fixgpu")]
            thranda_gpu_available: None,
            #[cfg(feature = "fixgpu")]
            gpu_generator_volts: DataRef::find("sim/cockpit2/electrical/GPU_generator_volts")?
                .writeable()?,
            #[cfg(feature = "fixgpu")]
            override_gpu_volts: DataRef::find("sim/operation/override/override_GPU_volts")?
                .writeable()?,

            #[cfg(feature = "fixnws")]
            hydraulic_pressure_green: DataRef::find(
                "sim/cockpit2/hydraulics/indicators/hydraulic_pressure_2",
            )?,
            #[cfg(feature = "fixnws")]
            override_wheel_steer: DataRef::find("sim/operation/override/override_wheel_steer")?
                .writeable()?,

            #[cfg(feature = "fixradio")]
            bus_volts: DataRef::find("sim/cockpit2/electrical/bus_volts")?,
            #[cfg(feature = "fixradio")]
            bus_volts_slice: [0.0; 2],
            #[cfg(feature = "fixradio")]
            radio_gps1_power: DataRef::find("sim/cockpit2/radios/actuators/gps_power")?,
            #[cfg(feature = "fixradio")]
            radio_gps2_power: DataRef::find("sim/cockpit2/radios/actuators/gps2_power")?,
            #[cfg(feature = "fixradio")]
            radio_com1_power: DataRef::find("sim/cockpit2/radios/actuators/com1_power")?
                .writeable()?,
            #[cfg(feature = "fixradio")]
            radio_com2_power: DataRef::find("sim/cockpit2/radios/actuators/com2_power")?
                .writeable()?,
            #[cfg(feature = "fixradio")]
            thranda_radio_com1_power: None,
            #[cfg(feature = "fixradio")]
            thranda_radio_com2_power: None,

            #[cfg(feature = "fixtms")]
            thranda_tgt_c_act: None,
            #[cfg(feature = "fixtms")]
            thranda_tgt_c_act_slice: [0.0; 3],

            #[cfg(feature = "syncthrottle")]
            throttle_levers: [0.0; 4],
            #[cfg(feature = "syncthrottle")]
            throttle_ratio: DataRef::find("sim/cockpit2/engine/actuators/throttle_ratio")?
                .writeable()?,
        };

        Ok(handler)
    }

    /// Fetch SASL datarefs if they are available, otherwise return an error
    fn initialize(&mut self) -> Result<(), PluginError> {
        #[cfg(feature = "fixgpu")]
        {
            if self.thranda_gpu_available.is_none() {
                self.thranda_gpu_available =
                    Some(DataRef::find("thranda/electrical/ExtPwrGPUAvailable")?);
            }
        }

        #[cfg(feature = "fixradio")]
        {
            if self.thranda_radio_com1_power.is_none() {
                self.thranda_radio_com1_power =
                    Some(DataRef::find("thranda/generic/com1/genCom1Pwr")?);
            }

            if self.thranda_radio_com2_power.is_none() {
                self.thranda_radio_com2_power =
                    Some(DataRef::find("thranda/generic/com1/genCom2Pwr")?);
            }
        }

        #[cfg(feature = "fixtms")]
        {
            if self.thranda_tgt_c_act.is_none() {
                self.thranda_tgt_c_act =
                    Some(DataRef::find("thranda/engine/TGT_C_Act")?.writeable()?);
            }
        }

        Ok(())
    }

    /// Run one-time actions
    fn onetime_actions(&mut self) {
        self.default_view.trigger();

        #[cfg(feature = "fixautothrottle")]
        self.autothrottle_hard_off.trigger();

        #[cfg(feature = "fixgpu")]
        self.override_gpu_volts.set(1);
    }

    /// The current GPU/external power isn't compatible with X-Plane's current GPU/external power
    /// implementation. This corrects the supplied generator voltage...
    #[cfg(feature = "fixgpu")]
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
    #[cfg(feature = "fixnws")]
    fn fix_nosewheel_steering(&mut self) {
        if self.hydraulic_pressure_green.get() > 100.0 {
            self.override_wheel_steer.set(1);
        }
    }

    /// Fix radio power based on bus voltage available
    #[cfg(feature = "fixradio")]
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

    #[cfg(feature = "fixtms")]
    fn fix_tms(&mut self) {
        if let Some(mut thranda_tgt_c_act) = self.thranda_tgt_c_act.take() {
            thranda_tgt_c_act.get(&mut self.thranda_tgt_c_act_slice);

            for i in 0..3 {
                if self.thranda_tgt_c_act_slice[i] < 0.0 {
                    self.thranda_tgt_c_act_slice[i] = 0.0;
                } else if self.thranda_tgt_c_act_slice[i] > 9.0 {
                    self.thranda_tgt_c_act_slice[i] = 9.0;
                } else {
                    self.thranda_tgt_c_act_slice[i] = self.thranda_tgt_c_act_slice[i].floor();
                }
            }

            thranda_tgt_c_act.set(&self.thranda_tgt_c_act_slice);
            self.thranda_tgt_c_act = Some(thranda_tgt_c_act);
        }
    }

    /// Align throttle lever 3 and 4 with throttle lever 2
    #[cfg(feature = "syncthrottle")]
    fn synchonize_throttle_levers(&mut self) {
        self.throttle_ratio.get(&mut self.throttle_levers);

        self.throttle_levers[2] = self.throttle_levers[1];
        self.throttle_levers[3] = self.throttle_levers[1];

        self.throttle_ratio.set(&self.throttle_levers);
    }
}

impl FlightLoopCallback for FlightLoopHandler {
    fn flight_loop(&mut self, state: &mut xplm::flight_loop::LoopState) {
        // We need to wait until all datarefs created by SASL are available...
        if !self.initialization_done {
            if self.initialize().is_ok() {
                self.initialization_done = true;
                debugln!("{PLUGIN_NAME} initialization complete");
            } else {
                // Exit flight loop early and try again after the next interval...
                debugln!("{PLUGIN_NAME} waiting for initialization...");
                return;
            }
        }

        if !self.onetime_actions_done {
            self.onetime_actions_done = true;
            self.onetime_actions();
            debugln!("{PLUGIN_NAME} one-time actions complete");
        }

        #[cfg(feature = "fixgpu")]
        self.fix_gpu_generator_volts();

        #[cfg(feature = "fixnws")]
        self.fix_nosewheel_steering();

        #[cfg(feature = "fixradio")]
        self.fix_radio();

        #[cfg(feature = "fixtms")]
        self.fix_tms();

        #[cfg(feature = "syncthrottle")]
        self.synchonize_throttle_levers();

        // Run flightloop callback on every flightloop from now on
        state.call_next_loop();
    }
}
