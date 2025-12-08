// Copyright (c) 2025 telephono
// Licensed under the MIT License. See LICENSE file in the project root for full license information.

use xplm::xplane_plugin;

mod handler;
mod plugin;

xplane_plugin!(plugin::TweaksPlugin);
