use crate::domain::{Fraction, fraction};

/// Limits for the traffic light, configurable in the settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrafficThresholds {
    /// Averages up to and including this value are green.
    pub green_max: Fraction,
    /// Averages above `green_max` up to and including this value are
    /// yellow; everything above is red.
    pub yellow_max: Fraction,
}

impl Default for TrafficThresholds {
    fn default() -> Self {
        TrafficThresholds {
            green_max: fraction(3, 1),
            yellow_max: fraction(4, 1),
        }
    }
}

/// Quick visual indicator of how safe a subject is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrafficLight {
    Green,
    Yellow,
    Red,
}

impl TrafficLight {
    /// Classifies a subject average.
    pub fn for_average(average: Fraction, thresholds: &TrafficThresholds) -> TrafficLight {
        if average <= thresholds.green_max {
            TrafficLight::Green
        } else if average <= thresholds.yellow_max {
            TrafficLight::Yellow
        } else {
            TrafficLight::Red
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_boundaries_are_inclusive() {
        let t = TrafficThresholds::default();
        let light = |n, d| TrafficLight::for_average(fraction(n, d), &t);
        assert_eq!(light(1, 1), TrafficLight::Green);
        assert_eq!(light(3, 1), TrafficLight::Green);
        assert_eq!(light(301, 100), TrafficLight::Yellow);
        assert_eq!(light(4, 1), TrafficLight::Yellow);
        assert_eq!(light(401, 100), TrafficLight::Red);
        assert_eq!(light(5, 1), TrafficLight::Red);
    }

    #[test]
    fn thresholds_are_configurable() {
        let t = TrafficThresholds {
            green_max: fraction(5, 2),
            yellow_max: fraction(7, 2),
        };
        assert_eq!(TrafficLight::for_average(fraction(3, 1), &t), TrafficLight::Yellow);
        assert_eq!(TrafficLight::for_average(fraction(4, 1), &t), TrafficLight::Red);
    }
}
