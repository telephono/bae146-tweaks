use xplm::data::borrowed::DataRef;
use xplm::data::StringRead;
use xplm::flight_loop::FlightLoop;
use xplm::plugin::management::plugin_with_signature;
use xplm::plugin::{Plugin, PluginInfo};

use crate::handler::FlightLoopHandler;

pub(crate) static PLUGIN_NAME: &str = concat!("BAe 146 Tweaks", " v", env!("CARGO_PKG_VERSION"));
static PLUGIN_SIGNATURE: &str = concat!("io.github.telephono.", env!("CARGO_PKG_NAME"));
static PLUGIN_DESCRIPTION: &str = "BAe 146 fixes and tweaks";

pub(crate) struct TweaksPlugin {
    flight_loop: FlightLoop,
}

impl Plugin for TweaksPlugin {
    type Error = PluginError;

    fn start() -> Result<Self, Self::Error> {
        if plugin_with_signature(PLUGIN_SIGNATURE).is_some() {
            return Err(PluginError::AlreadyRunning);
        }

        let acf_icao: DataRef<[u8]> = DataRef::find("sim/aircraft/view/acf_ICAO")?;
        let acf_icao = acf_icao.get_as_string()?;
        match acf_icao.as_str() {
            "B461" | "B462" | "B463" => debugln!("{PLUGIN_NAME} starting up..."),
            _ => return Err(PluginError::AircraftNotSupported(acf_icao)),
        }

        let handler = FlightLoopHandler::new()?;
        let plugin = Self {
            flight_loop: FlightLoop::new(handler),
        };

        debugln!("{PLUGIN_NAME} startup complete");
        Ok(plugin)
    }

    fn enable(&mut self) -> Result<(), Self::Error> {
        self.flight_loop.schedule_after_loops(300);

        debugln!("{PLUGIN_NAME} enabled");
        Ok(())
    }

    fn disable(&mut self) {
        self.flight_loop.deactivate();
        debugln!("{PLUGIN_NAME} disabled");
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: PLUGIN_NAME.to_string(),
            signature: PLUGIN_SIGNATURE.to_string(),
            description: PLUGIN_DESCRIPTION.to_string(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum PluginError {
    #[error("{PLUGIN_NAME} is already running")]
    AlreadyRunning,

    #[error("{PLUGIN_NAME} aircraft with ICAO code {0:?} is not supported")]
    AircraftNotSupported(String),

    #[error(transparent)]
    CommandFindError(#[from] xplm::command::CommandFindError),

    #[error(transparent)]
    DataRefFindError(#[from] xplm::data::borrowed::FindError),

    #[error(transparent)]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
}
