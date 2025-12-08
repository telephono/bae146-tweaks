// Copyright (c) 2025 telephono
// Licensed under the MIT License. See LICENSE file in the project root for full license information.

use xplm::xplane_plugin;

mod component;
mod handler;
mod plugin;

// Components
mod gpu;
mod hsi;
mod nosewheel_steering;
mod radio;
mod throttle_levers;

xplane_plugin!(plugin::TweaksPlugin);
