use super::{Context, Module, RootModuleConfig};
use crate::configs::battery::BatteryConfig;

/// Creates a module for the battery percentage and charging state
pub fn module<'a>(context: &'a Context) -> Option<Module<'a>> {
    // TODO: Update when v1.0 printing refactor is implemented to only
    // print escapes in a prompt context.
    let shell = std::env::var("STARSHIP_SHELL").unwrap_or_default();
    let percentage_char = match shell.as_str() {
        "zsh" => "%%", // % is an escape in zsh, see PROMPT in `man zshmisc`
        "powershell" => "`%",
        _ => "%",
    };

    let battery_status = get_battery_status()?;
    let BatteryStatus { state, percentage } = battery_status;

    let mut module = context.new_module("battery");
    let battery_config: BatteryConfig = BatteryConfig::try_load(module.config);

    // Parse config under `display`
    let display_styles = &battery_config.display;
    let display_style = display_styles
        .iter()
        .find(|display_style| percentage <= display_style.threshold as f32);

    if let Some(display_style) = display_style {
        // Set style based on percentage
        module.set_style(display_style.style);
        module.get_prefix().set_value("");

        match state {
            battery::State::Full => {
                module.create_segment("full_symbol", &battery_config.full_symbol);
            }
            battery::State::Charging => {
                module.create_segment("charging_symbol", &battery_config.charging_symbol);
            }
            battery::State::Discharging => {
                module.create_segment("discharging_symbol", &battery_config.discharging_symbol);
            }
            battery::State::Unknown => {
                log::debug!("Unknown detected");
                if let Some(unknown_symbol) = battery_config.unknown_symbol {
                    module.create_segment("unknown_symbol", &unknown_symbol);
                }
            }
            battery::State::Empty => {
                if let Some(empty_symbol) = battery_config.empty_symbol {
                    module.create_segment("empty_symbol", &empty_symbol);
                }
            }
            _ => {
                log::debug!("Unhandled battery state `{}`", state);
                return None;
            }
        }

        let mut percent_string = Vec::<String>::with_capacity(2);
        // Round the percentage to a whole number
        percent_string.push(percentage.round().to_string());
        percent_string.push(percentage_char.to_string());
        module.create_segment(
            "percentage",
            &battery_config
                .percentage
                .with_value(percent_string.join("").as_ref()),
        );

        Some(module)
    } else {
        None
    }
}

fn get_battery_status() -> Option<BatteryStatus> {
    let battery_manager = battery::Manager::new().ok()?;

    let batteries = match battery_manager.batteries() {
        Ok(batteries) => batteries,
        Err(err) => {
            log::debug!("Unable to access battery information:\n{}", &err);
            return None;
        }
    };

    match sum_batteries(batteries) {
        Some(total) => {
            // Doubles check for sane values
            if total.energy.value <= 0.0 {
                log::debug!("Unexpected current energy value: {:?}", total.energy);
                return None;
            }
            if total.energy_full.value <= 0.0 {
                log::debug!("Unexpected total energy value: {:?}", total.energy_full);
                return None;
            }

            // roughly equivalent to battery::Battery::state_of_charge
            // but does lose some accuracy as stated in `battery`'s docs
            let percentage = (total.energy.value / total.energy_full.value) * 100.0;
            log::debug!(
                "Total batteries energy, percentage: {} energy left: {:?} energy capacity: {:?} state: {:?}",
                percentage,
                total.energy,
                total.energy_full,
                total.state
            );

            Some(BatteryStatus {
                percentage,
                state: total.state,
            })
        }
        None => {
            log::debug!("No batteries found");
            None
        }
    }
}

fn sum_batteries(batteries: battery::Batteries) -> Option<BatterySum> {
    batteries.fold(None, |sum, battery| {
        match battery {
            Ok(battery) => {
                if let Some(sum) = sum {

                    let state = merge_battery_state(sum.state, battery.state());

                    Some(BatterySum {
                        energy: sum.energy + battery.energy(),
                        energy_full: sum.energy_full + battery.energy_full(),
                        state,
                    })
                } else {
                    Some(BatterySum {
                        energy: battery.energy(),
                        energy_full: battery.energy_full(),
                        state: battery.state(),
                    })
                }
            }
            Err(err) => {
                log::debug!("Unable to access battery information:\n{}", &err);
                sum
            }
        }
    })
}

fn merge_battery_state(left: battery::State, right: battery::State) -> battery::State {
    use battery::State::*;

    if left == right {
        return left;
    }

    match (left, right) {
        // If either of the batteries are discharging
        (_, Discharging) | (Discharging, _) => Discharging,
        // Then, if either of them are charging
        (Charging, _) | (_, Charging) => Charging,
        // All that is left is a combination of Full, Empty & Unknown
        // Not sure what is going on there
        _ => Unknown,
    }
}

struct BatterySum {
    energy: battery::units::Energy,
    energy_full: battery::units::Energy,
    state: battery::State,
}

struct BatteryStatus {
    percentage: f32,
    state: battery::State,
}

#[cfg(test)]
mod test {
    use super::*;
    use battery::State::*;

    #[test]
    fn battery_state_merge_always_discharge() {
        for state in &[Full, Empty, Discharging, Charging, Unknown] {
            assert_eq!(merge_battery_state(*state, Discharging), Discharging);
            assert_eq!(merge_battery_state(Discharging, *state), Discharging);
        }
    }

    #[test]
    fn battery_state_merge_charges() {
        // Note: discharging is missing since it wins over charging
        for state in &[Full, Empty, Charging, Unknown] {
            assert_eq!(merge_battery_state(*state, Charging), Charging);
            assert_eq!(merge_battery_state(Charging, *state), Charging);
        }
    }

    #[test]
    fn battery_state_merge_unknown() {
        for left in &[Full, Empty,  Unknown] {
            for right in &[Full, Empty,  Unknown] {
                if left == right {
                    assert_eq!(merge_battery_state(*left, *right), *left);
                } else {
                    assert_eq!(merge_battery_state(*left, *right), Unknown);
                }
            }
        }
    }

}
