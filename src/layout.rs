//! Shared geometry for the floating UI and its input masks.
use crate::{
    city::{Disaster, MapStyle},
    tools, State, Tool,
};
use vesper3d::two_d::{Point, Rect};

pub const TOP_BAR: Rect = Rect::new(12, 12, 776, 42);
pub const NAV: Rect = Rect::new(508, 64, 280, 28);
pub const DRAWER: Rect = Rect::new(548, 100, 240, 300);
pub const PALETTE: Rect = Rect::new(12, 348, 776, 52);
pub const DOCK: Rect = Rect::new(12, 408, 776, 30);
pub const INSPECT: Rect = Rect::new(500, 408, 68, 30);
pub const BULLDOZE: Rect = Rect::new(572, 408, 78, 30);
pub const ZOOM_MINUS: Rect = Rect::new(662, 408, 28, 30);
pub const ZOOM_PLUS: Rect = Rect::new(694, 408, 28, 30);
pub const CENTER: Rect = Rect::new(726, 408, 62, 30);
pub const CLOSE_PANEL: Rect = Rect::new(760, 106, 20, 20);
pub const CITY_OVERVIEW: Rect = Rect::new(682, 106, 72, 20);
pub const NEW_CITY: Rect = Rect::new(560, 362, 216, 26);
pub const CONFIRM_NEW: Rect = Rect::new(560, 346, 216, 26);
pub const CANCEL_NEW: Rect = Rect::new(560, 376, 216, 18);
pub const SEED_FIELD: Rect = Rect::new(560, 224, 216, 26);
pub const SEED_CLEAR: Rect = Rect::new(560, 310, 64, 22);
pub const SEED_BACKSPACE: Rect = Rect::new(630, 310, 64, 22);
pub const SEED_SHUFFLE: Rect = Rect::new(700, 310, 76, 22);
pub const BORROW: Rect = Rect::new(560, 310, 104, 22);
pub const REPAY: Rect = Rect::new(672, 310, 104, 22);

pub fn speed(i: u8) -> Rect {
    Rect::new(636 + i as i32 * 36, 22, 32, 22)
}
pub fn tab(i: u8) -> Rect {
    Rect::new(508 + i as i32 * 71, 64, 67, 28)
}
pub fn category(i: u8) -> Rect {
    Rect::new(12 + i as i32 * 96, 408, 92, 30)
}
pub fn palette_tool(category: u8, i: usize) -> Rect {
    let width = 764 / tools(category).len() as i32;
    Rect::new(18 + i as i32 * width, 354, width - 4, 40)
}
pub fn overlay(i: u8) -> Rect {
    Rect::new(560, 139 + i as i32 * 22, 216, 20)
}
pub fn tax_minus(i: usize) -> Rect {
    Rect::new(716, 158 + i as i32 * 20, 26, 18)
}
pub fn tax_plus(i: usize) -> Rect {
    Rect::new(750, 158 + i as i32 * 20, 26, 18)
}
pub fn funding_minus(i: usize) -> Rect {
    Rect::new(716, 234 + i as i32 * 18, 26, 16)
}
pub fn funding_plus(i: usize) -> Rect {
    Rect::new(750, 234 + i as i32 * 18, 26, 16)
}
pub fn policy(i: usize) -> Rect {
    Rect::new(
        560 + (i % 2) as i32 * 112,
        350 + (i / 2) as i32 * 20,
        104,
        18,
    )
}
pub fn disaster(i: usize) -> Rect {
    Rect::new(560 + i as i32 * 44, 277, 40, 30)
}
pub fn map_style(i: usize) -> Rect {
    Rect::new(
        560 + (i % 2) as i32 * 112,
        146 + (i / 2) as i32 * 30,
        104,
        26,
    )
}
pub fn seed_digit(digit: u8) -> Rect {
    Rect::new(
        560 + (digit % 5) as i32 * 44,
        260 + (digit / 5) as i32 * 24,
        40,
        20,
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Speed(u8),
    Tab(u8),
    Category(u8),
    SelectTool(Tool),
    Zoom(i32),
    Center,
    ClosePanel,
    CityOverview,
    Overlay(u8),
    Tax { row: usize, delta: i32 },
    Funding { row: usize, delta: i32 },
    Borrow,
    Repay,
    Policy(usize),
    Disaster(Disaster),
    NewCity,
    ConfirmNew,
    CancelNew,
    MapStyle(MapStyle),
    SelectSeed,
    SeedDigit(u8),
    ClearSeed,
    BackspaceSeed,
    ShuffleSeed,
}

pub fn controls(state: &State) -> Vec<(Rect, Action)> {
    let mut result = vec![
        (INSPECT, Action::SelectTool(Tool::Inspect)),
        (BULLDOZE, Action::SelectTool(Tool::Bulldoze)),
        (ZOOM_MINUS, Action::Zoom(-1)),
        (ZOOM_PLUS, Action::Zoom(1)),
        (CENTER, Action::Center),
    ];
    for i in 0..4 {
        result.push((speed(i), Action::Speed(i)));
        result.push((tab(i), Action::Tab(i)));
    }
    for i in 0..5 {
        result.push((category(i), Action::Category(i)));
    }
    if state.palette_open {
        for (i, t) in tools(state.category).into_iter().enumerate() {
            result.push((palette_tool(state.category, i), Action::SelectTool(t)));
        }
    }
    if state.panel_open || state.confirm_new {
        result.push((CLOSE_PANEL, Action::ClosePanel));
        if state.confirm_new {
            for (i, style) in MapStyle::ALL.into_iter().enumerate() {
                result.push((map_style(i), Action::MapStyle(style)));
            }
            result.push((SEED_FIELD, Action::SelectSeed));
            for digit in 0..10 {
                result.push((seed_digit(digit), Action::SeedDigit(digit)));
            }
            result.push((SEED_CLEAR, Action::ClearSeed));
            result.push((SEED_BACKSPACE, Action::BackspaceSeed));
            result.push((SEED_SHUFFLE, Action::ShuffleSeed));
            result.push((CONFIRM_NEW, Action::ConfirmNew));
            result.push((CANCEL_NEW, Action::CancelNew));
        } else {
            match state.tab {
                0 => {
                    result.push((NEW_CITY, Action::NewCity));
                    if state.selected.is_some() {
                        result.push((CITY_OVERVIEW, Action::CityOverview));
                    }
                }
                1 => {
                    for i in 0..9 {
                        result.push((overlay(i), Action::Overlay(i)));
                    }
                }
                2 => {
                    for row in 0..3 {
                        result.push((tax_minus(row), Action::Tax { row, delta: -1 }));
                        result.push((tax_plus(row), Action::Tax { row, delta: 1 }));
                    }
                    for row in 0..4 {
                        result.push((funding_minus(row), Action::Funding { row, delta: -10 }));
                        result.push((funding_plus(row), Action::Funding { row, delta: 10 }));
                        result.push((policy(row), Action::Policy(row)));
                    }
                    result.push((BORROW, Action::Borrow));
                    result.push((REPAY, Action::Repay));
                }
                _ => {
                    for (i, d) in [
                        Disaster::Fire,
                        Disaster::Flood,
                        Disaster::Earthquake,
                        Disaster::Tornado,
                        Disaster::Meltdown,
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        result.push((disaster(i), Action::Disaster(d)));
                    }
                    result.push((NEW_CITY, Action::NewCity));
                }
            }
        }
    }
    result
}
pub fn hit(state: &State, point: Point) -> Option<Action> {
    controls(state)
        .into_iter()
        .find(|(r, _)| r.contains(point))
        .map(|(_, a)| a)
}
pub fn blocked_map(state: &State, point: Point) -> bool {
    TOP_BAR.contains(point)
        || NAV.contains(point)
        || DOCK.contains(point)
        || (state.palette_open && PALETTE.contains(point))
        || ((state.panel_open || state.confirm_new) && DRAWER.contains(point))
}
