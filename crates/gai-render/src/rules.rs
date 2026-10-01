//! Per-set additions to the canonical checks, as data.
//!
//! Sets 35 onwards (and every new set) run the canonical `verify.html` with no
//! additions. Sets 06, 12, 16 and 25–34 carry raster rules or loader overrides
//! that their own pages ran and that some of their icons need to pass.

use crate::player::Style;

/// A minimum count of enclosed openings for an icon in a style, if it has one.
type Minimum = fn(icon: &str, style: Style) -> Option<u32>;

/// Sets 25–34: enclosed openings that must survive at small sizes (all
/// non-duotone styles, every frame). Counts use `counters(buffer, size, 1)`.
#[derive(Clone, Copy, Debug)]
pub struct CounterRule {
    /// The smallest size the rule applies to (24 or 32); the largest is 48.
    pub from_size: u32,
    minimum: Minimum,
}

impl CounterRule {
    /// The required opening count for `icon` in `style` at `size`, if any.
    pub fn minimum(&self, icon: &str, style: Style, size: u32) -> Option<u32> {
        if size < self.from_size || size > 48 || style == Style::Duotone {
            return None;
        }
        (self.minimum)(icon, style)
    }
}

/// Continuous-loader messages and playback settings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoaderRule {
    /// Message when a loader's rest pose is empty.
    pub empty_rest: &'static str,
    /// Message when the loop holds a frame five or more times.
    pub pauses: &'static str,
    /// Whether the "Insufficient continuous motion" message carries the count.
    pub count_in_message: bool,
    /// Controller speed during the real-playback check.
    pub speed: f32,
    /// Real-playback timeout in milliseconds.
    pub timeout_ms: f32,
    /// Icons with a smaller distinct-frame minimum than 32 (rotational symmetry).
    pub distinct_overrides: &'static [(&'static str, usize)],
}

impl LoaderRule {
    /// The canonical loader settings (sets 17 and 35 onwards).
    pub const CANONICAL: LoaderRule = LoaderRule {
        empty_rest: "Continuous rest pose is empty",
        pauses: "Continuous media indicator pauses",
        count_in_message: false,
        speed: 1.0,
        timeout_ms: 9000.0,
        distinct_overrides: &[],
    };

    /// Required distinct frames over one continuous loop.
    pub fn distinct_minimum(&self, icon: &str) -> usize {
        self.distinct_overrides.iter().find(|(name, _)| *name == icon).map_or(32, |&(_, n)| n)
    }
}

/// The rules a set's visual verification runs on top of the canonical checks.
#[derive(Clone, Copy, Debug)]
pub struct SetRules {
    /// Enclosed-opening minima at small sizes (sets 25–34).
    pub native_counters: Option<CounterRule>,
    /// Set 06 `keyframes`: diamond openings stay open in Outline.
    pub keyframe_openings: bool,
    /// Set 06 `credit-card`/`wallet`: Solid keeps two enclosed details at 192 px.
    pub solid_counters: bool,
    /// Loader messages and playback settings.
    pub loader: LoaderRule,
}

impl Default for SetRules {
    fn default() -> SetRules {
        SetRules::CANONICAL
    }
}

fn table(entries: &'static [(&'static str, u32)], icon: &str) -> Option<u32> {
    entries.iter().find(|(name, _)| *name == icon).map(|&(_, n)| n)
}

fn counters(from_size: u32, minimum: Minimum) -> Option<CounterRule> {
    Some(CounterRule { from_size, minimum })
}

impl SetRules {
    /// No additions: sets 35 onwards and every new set.
    pub const CANONICAL: SetRules =
        SetRules { native_counters: None, keyframe_openings: false, solid_counters: false, loader: LoaderRule::CANONICAL };

    /// The rules of a set by its two-digit id (`"06"`, `"34"`, …).
    pub fn for_set(set_id: &str) -> SetRules {
        let mut rules = SetRules::CANONICAL;
        match set_id {
            "06" => {
                rules.keyframe_openings = true;
                rules.solid_counters = true;
            }
            "12" => {
                rules.loader = LoaderRule {
                    empty_rest: "Loader rest pose is empty",
                    pauses: "Loader pauses inside its repeat cycle",
                    count_in_message: true,
                    speed: 2.0,
                    timeout_ms: 6000.0,
                    // Six identical paddles repeat every 60 degrees: 144/6 = 24 phase frames.
                    distinct_overrides: &[("pinwheel", 18)],
                }
            }
            "16" => rules.loader.pauses = "Continuous voice indicator pauses",
            "25" => rules.native_counters = counters(24, |icon, _| table(&[("command-menu", 5), ("cut", 2)], icon)),
            "26" => {
                rules.native_counters =
                    counters(32, |icon, _| table(&[("passkey", 1), ("private-browsing", 2), ("two-factor-code", 2)], icon))
            }
            "27" => {
                rules.native_counters = counters(32, |icon, _| {
                    table(
                        &[("checkbox", 1), ("radio-button", 1), ("toggle-switch", 1), ("multiple-choice", 2), ("qr-code", 3)],
                        icon,
                    )
                })
            }
            "28" => {
                rules.native_counters =
                    counters(32, |icon, _| table(&[("percentage", 2), ("heatmap", 5), ("radar-chart", 1)], icon))
            }
            "29" => {
                rules.native_counters = counters(32, |icon, _| {
                    table(&[("bold", 2), ("paragraph", 1), ("text-color", 1), ("letter-case", 1)], icon)
                })
            }
            "30" => {
                rules.native_counters = counters(32, |icon, style| {
                    const ICONS: [&str; 7] = [
                        "accessibility",
                        "wheelchair-access",
                        "high-contrast",
                        "text-size",
                        "keyboard",
                        "keyboard-shortcuts",
                        "reading-mode",
                    ];
                    ICONS.contains(&icon).then(|| match icon {
                        "keyboard-shortcuts" => 2,
                        "keyboard" | "reading-mode" if style == Style::Solid => 4,
                        _ => 1,
                    })
                })
            }
            "31" => {
                rules.native_counters = counters(32, |icon, _| {
                    table(
                        &[
                            ("repository", 1),
                            ("git-branch", 2),
                            ("git-commit", 1),
                            ("git-merge", 3),
                            ("pull-request", 3),
                            ("repository-fork", 3),
                            ("debug", 1),
                        ],
                        icon,
                    )
                })
            }
            "32" => {
                rules.native_counters = counters(32, |icon, style| {
                    if style == Style::Solid {
                        table(
                            &[
                                ("server", 4),
                                ("database", 1),
                                ("processor", 1),
                                ("memory-chip", 3),
                                ("hard-drive", 3),
                                ("ethernet", 2),
                                ("router", 1),
                                ("load-balancer", 3),
                                ("usb-drive", 3),
                            ],
                            icon,
                        )
                    } else {
                        table(
                            &[
                                ("server", 2),
                                ("database", 3),
                                ("processor", 1),
                                ("memory-chip", 1),
                                ("hard-drive", 1),
                                ("ethernet", 1),
                                ("router", 1),
                                ("load-balancer", 3),
                                ("firewall", 4),
                                ("usb-drive", 1),
                            ],
                            icon,
                        )
                    }
                })
            }
            "33" => {
                rules.native_counters = counters(32, |icon, style| {
                    if style == Style::Solid {
                        table(&[("emoji-picker", 3), ("address-book", 1), ("voicemail", 2)], icon)
                    } else {
                        table(
                            &[
                                ("inbox", 1),
                                ("address-book", 1),
                                ("voicemail", 2),
                                ("emoji-picker", 1),
                                ("thumbs-up-down", 1),
                                ("announcement", 1),
                            ],
                            icon,
                        )
                    }
                })
            }
            "34" => {
                rules.native_counters = counters(32, |icon, _| {
                    table(
                        &[
                            ("date-range", 2),
                            ("calendar-view", 2),
                            ("reschedule", 2),
                            ("alarm-clock", 1),
                            ("stopwatch", 1),
                            ("countdown-timer", 1),
                            ("world-clock", 4),
                            ("availability", 1),
                        ],
                        icon,
                    )
                })
            }
            _ => {}
        }
        rules
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_rules_follow_the_pages() {
        let r25 = SetRules::for_set("25").native_counters.unwrap();
        assert_eq!(r25.minimum("command-menu", Style::Outline, 24), Some(5));
        assert_eq!(r25.minimum("cut", Style::Solid, 48), Some(2));
        assert_eq!(r25.minimum("cut", Style::Duotone, 32), None);
        assert_eq!(r25.minimum("cut", Style::Outline, 192), None);
        let r30 = SetRules::for_set("30").native_counters.unwrap();
        assert_eq!(r30.minimum("keyboard", Style::Solid, 32), Some(4));
        assert_eq!(r30.minimum("keyboard", Style::Outline, 32), Some(1));
        assert_eq!(r30.minimum("keyboard", Style::Outline, 24), None);
        assert_eq!(r30.minimum("keyboard-shortcuts", Style::Solid, 48), Some(2));
        let r32 = SetRules::for_set("32").native_counters.unwrap();
        assert_eq!(r32.minimum("firewall", Style::Solid, 32), None);
        assert_eq!(r32.minimum("firewall", Style::Outline, 32), Some(4));
    }

    #[test]
    fn loader_overrides() {
        let set12 = SetRules::for_set("12").loader;
        assert_eq!(set12.distinct_minimum("pinwheel"), 18);
        assert_eq!(set12.distinct_minimum("orbit-comet"), 32);
        assert_eq!(set12.speed, 2.0);
        assert_eq!(SetRules::for_set("16").loader.pauses, "Continuous voice indicator pauses");
        assert_eq!(SetRules::for_set("63").loader, LoaderRule::CANONICAL);
        assert!(SetRules::for_set("06").keyframe_openings && SetRules::for_set("06").solid_counters);
        assert!(SetRules::for_set("40").native_counters.is_none());
    }
}
