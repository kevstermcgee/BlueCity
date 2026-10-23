//! Authored civic content. Every entry has a typed, simulation-readable trigger.
use crate::city::{Kind, MapStyle};
use crate::world::Climate;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Metric {
    Population,
    Treasury,
    NetIncome,
    Happiness,
    Health,
    Education,
    Unemployment,
    Garbage,
    Transit,
    Traffic,
    Pollution,
    Crime,
    LandValue,
    PowerCoverage,
    WaterCoverage,
    Debt,
    Months,
    ResidentialTiles,
    CommercialJobs,
    IndustrialJobs,
    Parks,
    ForestTiles,
    HeritageSites,
    CompletedProjects,
    Approval,
    ElectionsWon,
    DisastersSurvived,
    RenewableShare,
    BondsOutstanding,
    SuccessfulScenarios,
    OperationalAirport,
    OperationalPort,
    ActiveWarnings,
    UnrepairedDamage,
    ReliefAvailable,
}
impl Metric {
    pub const COUNT: usize = 35;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum CivicEvent {
    BuiltTown,
    BalancedMonth,
    RepaidDebt,
    SurvivedDisaster,
    WonElection,
    SavedHeritage,
    CompletedProject,
    WonScenario,
    HeldDrill,
    RepairedDamage,
    LostElection,
    DeclaredBankruptcy,
}
impl CivicEvent {
    pub const COUNT: usize = 12;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum OrdinanceId {
    CleanAir,
    FreeTransit,
    Recycling,
    SmallBusinessGrants,
    Heritage,
    NoiseControl,
    RentSupport,
    Childcare,
    PublicHealth,
    WaterConservation,
    EnergyEfficiency,
    IndustrialSafety,
    TruckRoutes,
    TourismPromotion,
    SchoolMeals,
    PoliceAccountability,
    DisasterPreparedness,
}
impl OrdinanceId {
    pub const ALL: [Self; 17] = [
        Self::CleanAir,
        Self::FreeTransit,
        Self::Recycling,
        Self::SmallBusinessGrants,
        Self::Heritage,
        Self::NoiseControl,
        Self::RentSupport,
        Self::Childcare,
        Self::PublicHealth,
        Self::WaterConservation,
        Self::EnergyEfficiency,
        Self::IndustrialSafety,
        Self::TruckRoutes,
        Self::TourismPromotion,
        Self::SchoolMeals,
        Self::PoliceAccountability,
        Self::DisasterPreparedness,
    ];
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Perk {
    MayorsHouse,
    CityHall,
    Courthouse,
    CentralLibrary,
    CivicMuseum,
    BotanicalGarden,
    ConventionCentre,
    MedicalResearch,
    TransitAuthority,
    BroadcastingHouse,
    TradeCommission,
    ResilienceOffice,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Unlock {
    Services,
    Transit,
    HigherEducation,
    Seaport,
    Airport,
    Nuclear,
    Megaprojects,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Requirement {
    Min(Metric, i64),
    Max(Metric, i64),
    Buildings(Kind, u16),
    Operational(Kind, u16),
    Event(CivicEvent, u32),
    Policy(OrdinanceId),
}
#[derive(Clone, Copy, Debug)]
pub struct Goal {
    pub requirements: &'static [Requirement],
    pub months: u16,
}
#[derive(Clone, Debug)]
pub struct Facts {
    pub values: [i64; Metric::COUNT],
    pub counts: Vec<(Kind, u16, u16)>,
    pub events: [u32; CivicEvent::COUNT],
    pub ordinances: u32,
}
impl Default for Facts {
    fn default() -> Self {
        Self {
            values: [0; Metric::COUNT],
            counts: vec![],
            events: [0; CivicEvent::COUNT],
            ordinances: 0,
        }
    }
}
impl Facts {
    pub fn get(&self, m: Metric) -> i64 {
        self.values[m as usize]
    }
    pub fn set(&mut self, m: Metric, v: i64) {
        self.values[m as usize] = v;
    }
    pub fn count(&self, k: Kind, operational: bool) -> u16 {
        self.counts
            .iter()
            .find(|(kind, _, _)| *kind == k)
            .map(|(_, all, online)| if operational { *online } else { *all })
            .unwrap_or(0)
    }
    pub fn meets(&self, g: Goal) -> bool {
        g.requirements.iter().all(|r| match *r {
            Requirement::Min(m, n) => self.get(m) >= n,
            Requirement::Max(m, n) => self.get(m) <= n,
            Requirement::Buildings(k, n) => self.count(k, false) >= n,
            Requirement::Operational(k, n) => self.count(k, true) >= n,
            Requirement::Event(e, n) => self.events[e as usize] >= n,
            Requirement::Policy(p) => self.ordinances & (1 << p as u8) != 0,
        })
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Reward {
    pub cash: i64,
    pub perk: Option<Perk>,
    pub unlock: Option<Unlock>,
}
#[derive(Clone, Copy, Debug)]
pub struct MilestoneDef {
    pub title: &'static str,
    pub population: u32,
    pub goal: Goal,
    pub reward: Reward,
}
#[derive(Clone, Copy, Debug)]
pub struct RequestDef {
    pub title: &'static str,
    pub description: &'static str,
    pub goal: Goal,
    pub reward: Reward,
}
#[derive(Clone, Copy, Debug)]
pub struct AchievementDef {
    pub title: &'static str,
    pub description: &'static str,
    pub goal: Goal,
}
#[derive(Clone, Copy, Debug)]
pub struct ScenarioDef {
    pub title: &'static str,
    pub briefing: &'static str,
    pub deadline_months: u16,
    pub seed: u64,
    pub style: MapStyle,
    pub climate: Climate,
    pub start_population: u32,
    pub start_money: i64,
    pub goal: Goal,
    pub prize: i64,
}
#[derive(Clone, Copy, Debug)]
pub struct MessageDef {
    pub text: &'static str,
    pub goal: Goal,
    pub topic: u8,
}
#[derive(Clone, Copy, Debug)]
pub struct AdvisorLineDef {
    pub name: &'static str,
    pub text: &'static str,
}
#[derive(Clone, Copy, Debug)]
pub struct AdvisorTrigger {
    pub line: u8,
    pub priority: u8,
    pub goal: Goal,
}
#[derive(Clone, Copy, Debug)]
pub struct OrdinanceDef {
    pub id: OrdinanceId,
    pub title: &'static str,
    pub description: &'static str,
    pub monthly_base: i64,
    pub per_thousand: i64,
}
#[derive(Clone, Copy, Debug)]
pub struct MegaprojectDef {
    pub title: &'static str,
    pub description: &'static str,
    pub months: u16,
    pub cost: i64,
    pub goal: Goal,
}
macro_rules! goal {($months:expr; $($req:expr),* $(,)?)=>{Goal{requirements:&[$($req),*],months:$months}};}
use Metric::*;
use Requirement::{Buildings, Event, Max, Min, Operational, Policy};

pub const MILESTONES: [MilestoneDef; 25] = [
    MilestoneDef {
        title: "First Neighbors",
        population: 100,
        goal: goal!(1; Min(Population,100)),
        reward: Reward {
            cash: 520,
            perk: None,
            unlock: None,
        },
    },
    MilestoneDef {
        title: "Crossroads",
        population: 250,
        goal: goal!(1; Min(Population,250)),
        reward: Reward {
            cash: 550,
            perk: None,
            unlock: None,
        },
    },
    MilestoneDef {
        title: "Village Charter",
        population: 500,
        goal: goal!(1; Min(Population,500)),
        reward: Reward {
            cash: 600,
            perk: None,
            unlock: Some(Unlock::Services),
        },
    },
    MilestoneDef {
        title: "Market Day",
        population: 750,
        goal: goal!(1; Min(Population,750),Min(PowerCoverage,85),Min(WaterCoverage,85)),
        reward: Reward {
            cash: 650,
            perk: None,
            unlock: None,
        },
    },
    MilestoneDef {
        title: "A Town of Our Own",
        population: 1000,
        goal: goal!(1; Min(Population,1000),Min(PowerCoverage,85),Min(WaterCoverage,85)),
        reward: Reward {
            cash: 700,
            perk: None,
            unlock: None,
        },
    },
    MilestoneDef {
        title: "Growing Together",
        population: 1500,
        goal: goal!(1; Min(Population,1500),Min(PowerCoverage,85),Min(WaterCoverage,85)),
        reward: Reward {
            cash: 800,
            perk: None,
            unlock: Some(Unlock::Transit),
        },
    },
    MilestoneDef {
        title: "Service Centre",
        population: 2000,
        goal: goal!(1; Min(Population,2000),Min(PowerCoverage,85),Min(WaterCoverage,85)),
        reward: Reward {
            cash: 900,
            perk: None,
            unlock: None,
        },
    },
    MilestoneDef {
        title: "Busy Borough",
        population: 3000,
        goal: goal!(1; Min(Population,3000),Min(PowerCoverage,85),Min(WaterCoverage,85)),
        reward: Reward {
            cash: 1100,
            perk: None,
            unlock: None,
        },
    },
    MilestoneDef {
        title: "Learning City",
        population: 4500,
        goal: goal!(1; Min(Population,4500),Min(PowerCoverage,85),Min(WaterCoverage,85)),
        reward: Reward {
            cash: 1400,
            perk: None,
            unlock: Some(Unlock::HigherEducation),
        },
    },
    MilestoneDef {
        title: "Waterside Commerce",
        population: 6000,
        goal: goal!(1; Min(Population,6000),Min(PowerCoverage,85),Min(WaterCoverage,85),Min(Happiness,50)),
        reward: Reward {
            cash: 1700,
            perk: None,
            unlock: Some(Unlock::Seaport),
        },
    },
    MilestoneDef {
        title: "Regional Connection",
        population: 8000,
        goal: goal!(1; Min(Population,8000),Min(PowerCoverage,85),Min(WaterCoverage,85),Min(Happiness,50)),
        reward: Reward {
            cash: 2100,
            perk: None,
            unlock: Some(Unlock::Airport),
        },
    },
    MilestoneDef {
        title: "City Lights",
        population: 10000,
        goal: goal!(1; Min(Population,10000),Min(PowerCoverage,85),Min(WaterCoverage,85),Min(Happiness,50)),
        reward: Reward {
            cash: 2500,
            perk: None,
            unlock: None,
        },
    },
    MilestoneDef {
        title: "University Town",
        population: 12500,
        goal: goal!(1; Min(Population,12500),Min(PowerCoverage,85),Min(WaterCoverage,85),Min(Happiness,50)),
        reward: Reward {
            cash: 3000,
            perk: None,
            unlock: Some(Unlock::Nuclear),
        },
    },
    MilestoneDef {
        title: "Civic Confidence",
        population: 15000,
        goal: goal!(1; Min(Population,15000),Min(PowerCoverage,85),Min(WaterCoverage,85),Min(Happiness,50)),
        reward: Reward {
            cash: 3500,
            perk: None,
            unlock: None,
        },
    },
    MilestoneDef {
        title: "Green Ambition",
        population: 18000,
        goal: goal!(1; Min(Population,18000),Min(PowerCoverage,85),Min(WaterCoverage,85),Min(Happiness,50)),
        reward: Reward {
            cash: 4100,
            perk: None,
            unlock: None,
        },
    },
    MilestoneDef {
        title: "Metropolitan Charter",
        population: 22000,
        goal: goal!(1; Min(Population,22000),Min(PowerCoverage,85),Min(WaterCoverage,85),Min(Happiness,50)),
        reward: Reward {
            cash: 4900,
            perk: None,
            unlock: Some(Unlock::Megaprojects),
        },
    },
    MilestoneDef {
        title: "Connected Capital",
        population: 27000,
        goal: goal!(1; Min(Population,27000),Min(PowerCoverage,85),Min(WaterCoverage,85),Min(Happiness,50),Min(Education,45)),
        reward: Reward {
            cash: 5900,
            perk: None,
            unlock: None,
        },
    },
    MilestoneDef {
        title: "Research Corridor",
        population: 33000,
        goal: goal!(1; Min(Population,33000),Min(PowerCoverage,85),Min(WaterCoverage,85),Min(Happiness,50),Min(Education,45)),
        reward: Reward {
            cash: 7100,
            perk: None,
            unlock: None,
        },
    },
    MilestoneDef {
        title: "A Place for Everyone",
        population: 40000,
        goal: goal!(1; Min(Population,40000),Min(PowerCoverage,85),Min(WaterCoverage,85),Min(Happiness,50),Min(Education,45)),
        reward: Reward {
            cash: 8500,
            perk: None,
            unlock: None,
        },
    },
    MilestoneDef {
        title: "Regional Powerhouse",
        population: 50000,
        goal: goal!(1; Min(Population,50000),Min(PowerCoverage,85),Min(WaterCoverage,85),Min(Happiness,50),Min(Education,45)),
        reward: Reward {
            cash: 10500,
            perk: None,
            unlock: None,
        },
    },
    MilestoneDef {
        title: "Grand Metropolis",
        population: 65000,
        goal: goal!(1; Min(Population,65000),Min(PowerCoverage,85),Min(WaterCoverage,85),Min(Happiness,50),Min(Education,45)),
        reward: Reward {
            cash: 13500,
            perk: None,
            unlock: None,
        },
    },
    MilestoneDef {
        title: "World City",
        population: 85000,
        goal: goal!(1; Min(Population,85000),Min(PowerCoverage,85),Min(WaterCoverage,85),Min(Happiness,50),Min(Education,45)),
        reward: Reward {
            cash: 17500,
            perk: None,
            unlock: None,
        },
    },
    MilestoneDef {
        title: "Cultural Beacon",
        population: 110000,
        goal: goal!(1; Min(Population,110000),Min(PowerCoverage,85),Min(WaterCoverage,85),Min(Happiness,50),Min(Education,45)),
        reward: Reward {
            cash: 22500,
            perk: None,
            unlock: None,
        },
    },
    MilestoneDef {
        title: "Continental Capital",
        population: 150000,
        goal: goal!(1; Min(Population,150000),Min(PowerCoverage,85),Min(WaterCoverage,85),Min(Happiness,50),Min(Education,45)),
        reward: Reward {
            cash: 25000,
            perk: None,
            unlock: None,
        },
    },
    MilestoneDef {
        title: "Blue Horizon",
        population: 200000,
        goal: goal!(1; Min(Population,200000),Min(PowerCoverage,85),Min(WaterCoverage,85),Min(Happiness,50),Min(Education,45)),
        reward: Reward {
            cash: 25000,
            perk: None,
            unlock: None,
        },
    },
];
pub const REQUESTS: [RequestDef; 12] = [
    RequestDef {
        title: "A Home for the Mayor",
        description: "Residents offer a modest mayoral residence after a stable first year.",
        goal: goal!(3; Min(Population,500),Min(Happiness,60),Min(Months,12)),
        reward: Reward {
            cash: 1500,
            perk: Some(Perk::MayorsHouse),
            unlock: None,
        },
    },
    RequestDef {
        title: "One City, One Hall",
        description: "The town asks for a permanent, accountable civic administration.",
        goal: goal!(6; Min(Population,1500),Min(NetIncome,0),Min(PowerCoverage,95),Min(WaterCoverage,95)),
        reward: Reward {
            cash: 2500,
            perk: Some(Perk::CityHall),
            unlock: None,
        },
    },
    RequestDef {
        title: "Justice Close to Home",
        description: "Keep neighborhoods safe without cutting essential services.",
        goal: goal!(6; Min(Population,3000),Max(Crime,25),Operational(Kind::Police,1)),
        reward: Reward {
            cash: 3000,
            perk: Some(Perk::Courthouse),
            unlock: None,
        },
    },
    RequestDef {
        title: "Open the Central Library",
        description: "Students need a shared learning place beyond the classroom.",
        goal: goal!(3; Min(Population,4500),Min(Education,55),Operational(Kind::School,2)),
        reward: Reward {
            cash: 3000,
            perk: Some(Perk::CentralLibrary),
            unlock: None,
        },
    },
    RequestDef {
        title: "Our Story, Preserved",
        description: "Citizens request a museum after the first heritage district is protected.",
        goal: goal!(6; Min(Population,5000),Min(HeritageSites,1),Policy(OrdinanceId::Heritage)),
        reward: Reward {
            cash: 3500,
            perk: Some(Perk::CivicMuseum),
            unlock: None,
        },
    },
    RequestDef {
        title: "Room to Breathe",
        description: "A botanical collection rewards a healthy network of public green space.",
        goal: goal!(6; Min(Population,6000),Min(Parks,8),Max(Pollution,25)),
        reward: Reward {
            cash: 4000,
            perk: Some(Perk::BotanicalGarden),
            unlock: None,
        },
    },
    RequestDef {
        title: "A Meeting Place for the Region",
        description: "Local firms want a convention centre with reliable regional connections.",
        goal: goal!(6; Min(Population,10000),Min(CommercialJobs,2500),Operational(Kind::Airport,1)),
        reward: Reward {
            cash: 5000,
            perk: Some(Perk::ConventionCentre),
            unlock: None,
        },
    },
    RequestDef {
        title: "Care Beyond the Clinic",
        description:
            "Doctors propose a research partnership after sustained public health improvements.",
        goal: goal!(6; Min(Population,12500),Min(Health,75),Operational(Kind::Hospital,2),Operational(Kind::University,1)),
        reward: Reward {
            cash: 6000,
            perk: Some(Perk::MedicalResearch),
            unlock: None,
        },
    },
    RequestDef {
        title: "A Network That Works",
        description: "Bus and rail riders ask for one coordinated transit authority.",
        goal: goal!(6; Min(Population,15000),Min(Transit,35),Operational(Kind::BusDepot,1),Operational(Kind::Station,2)),
        reward: Reward {
            cash: 6500,
            perk: Some(Perk::TransitAuthority),
            unlock: None,
        },
    },
    RequestDef {
        title: "A Voice for the City",
        description: "A trusted broadcaster will celebrate a city with a healthy public mandate.",
        goal: goal!(6; Min(Population,18000),Min(Approval,70),Event(CivicEvent::WonElection,1)),
        reward: Reward {
            cash: 7500,
            perk: Some(Perk::BroadcastingHouse),
            unlock: None,
        },
    },
    RequestDef {
        title: "Open for Fair Trade",
        description: "Workers request a trade commission for a prosperous, clean harbor.",
        goal: goal!(6; Min(Population,22000),Min(IndustrialJobs,6000),Operational(Kind::Seaport,1),Max(Pollution,40)),
        reward: Reward {
            cash: 8500,
            perk: Some(Perk::TradeCommission),
            unlock: None,
        },
    },
    RequestDef {
        title: "Ready for the Next Storm",
        description: "Emergency workers seek a permanent resilience office after recovery.",
        goal: goal!(6; Event(CivicEvent::SurvivedDisaster,1),Event(CivicEvent::HeldDrill,1),Event(CivicEvent::RepairedDamage,5),Min(PowerCoverage,95),Min(WaterCoverage,95)),
        reward: Reward {
            cash: 10000,
            perk: Some(Perk::ResilienceOffice),
            unlock: None,
        },
    },
];
pub const ACHIEVEMENTS: [AchievementDef; 8] = [
    AchievementDef {
        title: "First Foundations",
        description: "Welcome the first hundred residents.",
        goal: goal!(1; Min(Population,100)),
    },
    AchievementDef {
        title: "Books Balance",
        description: "Keep a surplus and no municipal debt for a full year.",
        goal: goal!(12; Min(NetIncome,0),Max(Debt,0),Min(Population,1000)),
    },
    AchievementDef {
        title: "Every Tap, Every Light",
        description: "Provide reliable power and water to a city of ten thousand.",
        goal: goal!(6; Min(Population,10000),Min(PowerCoverage,100),Min(WaterCoverage,100)),
    },
    AchievementDef {
        title: "Beyond the Car",
        description: "Make public transport a habit for the region.",
        goal: goal!(6; Min(Population,15000),Min(Transit,50)),
    },
    AchievementDef {
        title: "City of Learning",
        description: "Sustain strong education and low unemployment.",
        goal: goal!(12; Min(Education,80),Max(Unemployment,5),Min(Population,12500)),
    },
    AchievementDef {
        title: "The Living Past",
        description: "Preserve twelve occupied historic sites.",
        goal: goal!(6; Min(HeritageSites,12),Policy(OrdinanceId::Heritage)),
    },
    AchievementDef {
        title: "Back on Our Feet",
        description: "Recover from three disasters and complete twenty repairs.",
        goal: goal!(3; Event(CivicEvent::SurvivedDisaster,3),Event(CivicEvent::RepairedDamage,20),Min(Happiness,65)),
    },
    AchievementDef {
        title: "A Blue Horizon",
        description: "Complete two megaprojects and serve one hundred thousand residents.",
        goal: goal!(6; Min(Population,100000),Min(CompletedProjects,2),Min(Approval,70)),
    },
];
pub const SCENARIOS:[ScenarioDef;8]=[
ScenarioDef{title:"Quiet Hollow",briefing:"A farming village wants opportunity without losing its peace. Grow carefully and keep the air clean.",deadline_months:60,seed:301,style:MapStyle::Highlands,climate:Climate::Temperate,start_population:800,start_money:45000,goal:goal!(6; Min(Population,4000),Min(Happiness,70),Max(Pollution,30),Max(Debt,0)),prize:15000},
ScenarioDef{title:"Gridlock Junction",briefing:"A busy commuter town is trapped in queues. Build useful transit and keep existing residents.",deadline_months:36,seed:719,style:MapStyle::Flatlands,climate:Climate::Temperate,start_population:12000,start_money:60000,goal:goal!(6; Min(Population,12000),Max(Traffic,45),Min(Transit,35),Min(Treasury,0)),prize:20000},
ScenarioDef{title:"River Delta",briefing:"Fast growth strains a flood-prone delta. Flood warnings arrive in months six and eighteen.",deadline_months:60,seed:1183,style:MapStyle::Delta,climate:Climate::Tropical,start_population:5000,start_money:55000,goal:goal!(6; Min(Population,12000),Min(PowerCoverage,95),Min(WaterCoverage,95),Max(Pollution,35)),prize:25000},
ScenarioDef{title:"Harbour Trouble",briefing:"The harbor has lost its customers. Reconnect freight, restore employment and balance the port economy.",deadline_months:48,seed:2101,style:MapStyle::Coast,climate:Climate::Temperate,start_population:9000,start_money:50000,goal:goal!(6; Operational(Kind::Seaport,1),Min(IndustrialJobs,5000),Min(NetIncome,2500),Max(Pollution,45)),prize:25000},
ScenarioDef{title:"Campus Comeback",briefing:"The last generation left for college and never returned. Build an educated, employed university town.",deadline_months:48,seed:3593,style:MapStyle::Highlands,climate:Climate::Alpine,start_population:6500,start_money:50000,goal:goal!(6; Min(Population,10000),Operational(Kind::University,1),Min(Education,70),Max(Unemployment,8)),prize:22000},
ScenarioDef{title:"Ashes to Avenues",briefing:"A recent fire damaged the town centre. Repair the streets and restore confidence before another dry season.",deadline_months:36,seed:4999,style:MapStyle::Flatlands,climate:Climate::Temperate,start_population:7000,start_money:45000,goal:goal!(6; Min(Population,6000),Event(CivicEvent::RepairedDamage,8),Operational(Kind::Fire,2),Min(PowerCoverage,95),Min(WaterCoverage,95)),prize:24000},
ScenarioDef{title:"Clean Slate",briefing:"A coal-dependent city wants a cleaner future. Replace dirty generation while preserving daily life.",deadline_months:48,seed:6121,style:MapStyle::Lakes,climate:Climate::Temperate,start_population:8000,start_money:65000,goal:goal!(6; Min(Population,8000),Min(RenewableShare,70),Max(Pollution,25),Min(PowerCoverage,95),Min(Treasury,0)),prize:26000},
ScenarioDef{title:"Capital Challenge",briefing:"Years of borrowing left little room to govern. Retire the debt and rebuild a credible reserve.",deadline_months:36,seed:7741,style:MapStyle::Flatlands,climate:Climate::Tropical,start_population:10000,start_money:8000,goal:goal!(12; Max(Debt,0),Max(BondsOutstanding,0),Min(NetIncome,0),Min(Treasury,20000),Min(Approval,65)),prize:30000},
];
pub const ORDINANCES: [OrdinanceDef; 17] = [
    OrdinanceDef {
        id: OrdinanceId::CleanAir,
        title: "Clean Air",
        description: "Cleaner industry cuts neighborhood pollution; inspections cost money.",
        monthly_base: 35,
        per_thousand: 8,
    },
    OrdinanceDef {
        id: OrdinanceId::FreeTransit,
        title: "Free Transit",
        description: "Free buses and trains attract riders; fares become a municipal subsidy.",
        monthly_base: 25,
        per_thousand: 14,
    },
    OrdinanceDef {
        id: OrdinanceId::Recycling,
        title: "Recycling",
        description: "Sorted collections double landfill capacity and reduce the waste burden.",
        monthly_base: 30,
        per_thousand: 9,
    },
    OrdinanceDef {
        id: OrdinanceId::SmallBusinessGrants,
        title: "Small Business Grants",
        description: "Local enterprises receive grants that support jobs and tax receipts.",
        monthly_base: 20,
        per_thousand: 12,
    },
    OrdinanceDef {
        id: OrdinanceId::Heritage,
        title: "Heritage",
        description: "Protect designated historic buildings and support their maintenance.",
        monthly_base: 20,
        per_thousand: 3,
    },
    OrdinanceDef {
        id: OrdinanceId::NoiseControl,
        title: "Noise Control",
        description: "Night-time noise limits improve wellbeing but reduce visitor receipts.",
        monthly_base: 25,
        per_thousand: 5,
    },
    OrdinanceDef {
        id: OrdinanceId::RentSupport,
        title: "Rent Support",
        description: "Targeted housing support improves happiness at a recurring cost.",
        monthly_base: 15,
        per_thousand: 16,
    },
    OrdinanceDef {
        id: OrdinanceId::Childcare,
        title: "Childcare",
        description: "Affordable childcare supports working households and public health.",
        monthly_base: 25,
        per_thousand: 11,
    },
    OrdinanceDef {
        id: OrdinanceId::PublicHealth,
        title: "Public Health",
        description: "Outreach clinics raise health and help vulnerable neighborhoods.",
        monthly_base: 35,
        per_thousand: 10,
    },
    OrdinanceDef {
        id: OrdinanceId::WaterConservation,
        title: "Water Conservation",
        description: "Efficient fixtures reduce water consumption by fifteen percent.",
        monthly_base: 20,
        per_thousand: 4,
    },
    OrdinanceDef {
        id: OrdinanceId::EnergyEfficiency,
        title: "Energy Efficiency",
        description: "Retrofits reduce electricity demand by twelve percent.",
        monthly_base: 25,
        per_thousand: 5,
    },
    OrdinanceDef {
        id: OrdinanceId::IndustrialSafety,
        title: "Industrial Safety",
        description: "Factory inspections reduce industrial fire and disaster losses.",
        monthly_base: 30,
        per_thousand: 7,
    },
    OrdinanceDef {
        id: OrdinanceId::TruckRoutes,
        title: "Truck Routes",
        description: "Managed freight routes reduce traffic near homes at a road-maintenance cost.",
        monthly_base: 25,
        per_thousand: 6,
    },
    OrdinanceDef {
        id: OrdinanceId::TourismPromotion,
        title: "Tourism Promotion",
        description: "Promote the city; visitor receipts depend on its parks and heritage.",
        monthly_base: 35,
        per_thousand: 5,
    },
    OrdinanceDef {
        id: OrdinanceId::SchoolMeals,
        title: "School Meals",
        description: "Reliable school meals improve education and child health.",
        monthly_base: 20,
        per_thousand: 8,
    },
    OrdinanceDef {
        id: OrdinanceId::PoliceAccountability,
        title: "Police Accountability",
        description: "Community oversight improves trust and lowers neighborhood crime.",
        monthly_base: 25,
        per_thousand: 6,
    },
    OrdinanceDef {
        id: OrdinanceId::DisasterPreparedness,
        title: "Disaster Preparedness",
        description: "Fund warning systems and resilient construction to reduce disaster damage.",
        monthly_base: 40,
        per_thousand: 8,
    },
];
pub const MEGAPROJECTS:[MegaprojectDef;4]=[
MegaprojectDef{title:"Arcology",description:"A dense, self-contained vertical community. Its completed civic systems improve housing and utilities.",months:24,cost:120000,goal:goal!(1; Min(Population,25000),Min(Education,60),Min(PowerCoverage,95),Min(WaterCoverage,95))},
MegaprojectDef{title:"Spaceport",description:"A launch complex connects a skilled city to an emerging space economy.",months:36,cost:180000,goal:goal!(1; Min(Population,40000),Min(Education,75),Operational(Kind::Airport,1),Min(NetIncome,2000))},
MegaprojectDef{title:"World Expo",description:"A permanent international exhibition brings visitors, trade and cultural confidence.",months:18,cost:80000,goal:goal!(1; Min(Population,20000),Min(Happiness,65),Operational(Kind::Stadium,1),Operational(Kind::Seaport,1))},
MegaprojectDef{title:"Science Center",description:"Shared laboratories expand medical research and technical education.",months:24,cost:100000,goal:goal!(1; Min(Population,30000),Min(Education,75),Operational(Kind::University,2),Operational(Kind::Hospital,2))},
];
pub const CHIRPER:[MessageDef;150]=[
MessageDef{text:"First moving van on our street this morning. Someone brought the neighbors fresh bread.",goal:goal!(1; Min(Population,100),Min(Population,1)),topic:0},
MessageDef{text:"The corner shop finally knows my name. This is starting to feel like home.",goal:goal!(1; Min(Population,100),Min(Population,1)),topic:0},
MessageDef{text:"Our little block has enough children for a sidewalk chalk league.",goal:goal!(1; Min(Population,100),Min(Population,1)),topic:0},
MessageDef{text:"New neighbors planted a tree before they unpacked their sofa. Good priorities.",goal:goal!(1; Min(Population,100),Min(Population,1)),topic:0},
MessageDef{text:"I can walk from home to work and still stop for breakfast. Keep building like this.",goal:goal!(1; Min(Population,100),Min(Population,1)),topic:0},
MessageDef{text:"The town map needs another fold. We have streets I have not explored yet.",goal:goal!(1; Min(Population,100),Min(Population,1)),topic:0},
MessageDef{text:"A welcome basket arrived before the internet installer. The neighbors are faster.",goal:goal!(1; Min(Population,100),Min(Population,1)),topic:0},
MessageDef{text:"The old empty lot is becoming homes. Hope the utilities arrive before the moving trucks.",goal:goal!(1; Min(Population,100),Min(Population,1)),topic:0},
MessageDef{text:"We started a neighborhood supper club. The first meeting required three extra tables.",goal:goal!(1; Min(Population,100),Min(Population,1)),topic:0},
MessageDef{text:"There are lights in the new apartments tonight. Every window is somebody starting over.",goal:goal!(1; Min(Population,100),Min(Population,1)),topic:0},
MessageDef{text:"The kettle stopped halfway through tea. Our block needs a reliable power connection.",goal:goal!(1; Max(PowerCoverage,85),Min(Population,1)),topic:1},
MessageDef{text:"Candles are romantic until you need to run a refrigerator.",goal:goal!(1; Max(PowerCoverage,85),Min(Population,1)),topic:1},
MessageDef{text:"The shop freezer alarm is becoming the soundtrack of this power shortage.",goal:goal!(1; Max(PowerCoverage,85),Min(Population,1)),topic:1},
MessageDef{text:"Our street has poles, but are they connected to a working generator?",goal:goal!(1; Max(PowerCoverage,85),Min(Population,1)),topic:1},
MessageDef{text:"One more blackout and the office printer might finally learn patience.",goal:goal!(1; Max(PowerCoverage,85),Min(Population,1)),topic:1},
MessageDef{text:"Watching the city lights stop at the next block is a very specific kind of envy.",goal:goal!(1; Max(PowerCoverage,85),Min(Population,1)),topic:1},
MessageDef{text:"New houses are lovely. Powered new houses would be even lovelier.",goal:goal!(1; Max(PowerCoverage,85),Min(Population,1)),topic:1},
MessageDef{text:"The clinic has asked us to keep the hallways clear during the outage.",goal:goal!(1; Max(PowerCoverage,85),Min(Population,1)),topic:1},
MessageDef{text:"Could we check the electricity map before zoning the next neighborhood?",goal:goal!(1; Max(PowerCoverage,85),Min(Population,1)),topic:1},
MessageDef{text:"I have named the blackout candle Kevin. Kevin is getting shorter.",goal:goal!(1; Max(PowerCoverage,85),Min(Population,1)),topic:1},
MessageDef{text:"The tap is coughing again. Somewhere between the pump and our street is a missing connection.",goal:goal!(1; Max(WaterCoverage,85),Min(Population,1)),topic:2},
MessageDef{text:"Our rooftop tank is nearly empty. Please check the neighborhood water supply.",goal:goal!(1; Max(WaterCoverage,85),Min(Population,1)),topic:2},
MessageDef{text:"I watered the basil with the last bottle. The basil now owes the household a favor.",goal:goal!(1; Max(WaterCoverage,85),Min(Population,1)),topic:2},
MessageDef{text:"A new pump helps only if electricity and pipes reach it. Ask me how I learned that.",goal:goal!(1; Max(WaterCoverage,85),Min(Population,1)),topic:2},
MessageDef{text:"The cafe menu currently offers dry toast and optimism.",goal:goal!(1; Max(WaterCoverage,85),Min(Population,1)),topic:2},
MessageDef{text:"Water pressure drops whenever the next block wakes up. Capacity matters too.",goal:goal!(1; Max(WaterCoverage,85),Min(Population,1)),topic:2},
MessageDef{text:"The swimming club has become a walking club until the taps work.",goal:goal!(1; Max(WaterCoverage,85),Min(Population,1)),topic:2},
MessageDef{text:"The plumber found a perfect pipe leading absolutely nowhere.",goal:goal!(1; Max(WaterCoverage,85),Min(Population,1)),topic:2},
MessageDef{text:"Please give the hospital reliable water before another ribbon cutting.",goal:goal!(1; Max(WaterCoverage,85),Min(Population,1)),topic:2},
MessageDef{text:"We can see the river from the kitchen, but still cannot fill the kettle.",goal:goal!(1; Max(WaterCoverage,85),Min(Population,1)),topic:2},
MessageDef{text:"My commute has three chapters and a disappointing sequel.",goal:goal!(1; Min(Traffic,55),Min(Population,1)),topic:3},
MessageDef{text:"The bus is stuck behind the same cars it could have replaced.",goal:goal!(1; Min(Traffic,55),Min(Population,1)),topic:3},
MessageDef{text:"The delivery driver has learned every traffic-light cycle by heart.",goal:goal!(1; Min(Traffic,55),Min(Population,1)),topic:3},
MessageDef{text:"A second way across town would save this bridge a lot of pressure.",goal:goal!(1; Min(Traffic,55),Min(Population,1)),topic:3},
MessageDef{text:"Our avenue is full, but the nearby street barely gets a visitor. Can routes be shared?",goal:goal!(1; Min(Traffic,55),Min(Population,1)),topic:3},
MessageDef{text:"The morning queue has reached the bakery. At least the view smells good.",goal:goal!(1; Min(Traffic,55),Min(Population,1)),topic:3},
MessageDef{text:"Freight trucks and school traffic should not need the same narrow shortcut.",goal:goal!(1; Min(Traffic,55),Min(Population,1)),topic:3},
MessageDef{text:"I left early enough to beat traffic and discovered that traffic had the same idea.",goal:goal!(1; Min(Traffic,55),Min(Population,1)),topic:3},
MessageDef{text:"Could new jobs be closer to the new homes? My bicycle supports this proposal.",goal:goal!(1; Min(Traffic,55),Min(Population,1)),topic:3},
MessageDef{text:"The traffic map is glowing warmer than my coffee. Please invest in transport.",goal:goal!(1; Min(Traffic,55),Min(Population,1)),topic:3},
MessageDef{text:"The bus arrived before I finished my book. A personal transport miracle.",goal:goal!(1; Min(Transit,25),Min(Population,1)),topic:4},
MessageDef{text:"Taking the train has given me back a quiet half-hour each morning.",goal:goal!(1; Min(Transit,25),Min(Population,1)),topic:4},
MessageDef{text:"The station cafe now sells more breakfasts than the parking lot has cars.",goal:goal!(1; Min(Transit,25),Min(Population,1)),topic:4},
MessageDef{text:"A useful bus stop is one that connects to somewhere people actually need to go.",goal:goal!(1; Min(Transit,25),Min(Population,1)),topic:4},
MessageDef{text:"My commute has become a walk, a ride and another walk. It works.",goal:goal!(1; Min(Transit,25),Min(Population,1)),topic:4},
MessageDef{text:"The driver recognizes the entire morning shift. Transit builds its own little community.",goal:goal!(1; Min(Transit,25),Min(Population,1)),topic:4},
MessageDef{text:"We tried a car-free weekend and discovered three parks near the station.",goal:goal!(1; Min(Transit,25),Min(Population,1)),topic:4},
MessageDef{text:"The bus lane moved a lot of people through very little space today.",goal:goal!(1; Min(Transit,25),Min(Population,1)),topic:4},
MessageDef{text:"Grandma visited without asking anyone for a lift. That is independence.",goal:goal!(1; Min(Transit,25),Min(Population,1)),topic:4},
MessageDef{text:"Rail and buses serving the same neighborhoods make the city feel smaller in a good way.",goal:goal!(1; Min(Transit,25),Min(Population,1)),topic:4},
MessageDef{text:"The washing came in smelling like the industrial district again.",goal:goal!(1; Min(Pollution,40),Min(Population,1)),topic:5},
MessageDef{text:"Our view of the hills comes with a gray filter nobody requested.",goal:goal!(1; Min(Pollution,40),Min(Population,1)),topic:5},
MessageDef{text:"The school moved outdoor games inside. Clean air should be basic infrastructure.",goal:goal!(1; Min(Pollution,40),Min(Population,1)),topic:5},
MessageDef{text:"I want jobs and a clear sky. Surely those can fit in the same plan.",goal:goal!(1; Min(Pollution,40),Min(Population,1)),topic:5},
MessageDef{text:"Please put some breathing room between homes and heavy industry.",goal:goal!(1; Min(Pollution,40),Min(Population,1)),topic:5},
MessageDef{text:"The riverbank picnic came with a less-than-appetizing industrial breeze.",goal:goal!(1; Min(Pollution,40),Min(Population,1)),topic:5},
MessageDef{text:"We planted flowers, but they cannot inspect factory exhausts.",goal:goal!(1; Min(Pollution,40),Min(Population,1)),topic:5},
MessageDef{text:"My child drew the sun behind a cloud of smoke. Time to fix the real picture.",goal:goal!(1; Min(Pollution,40),Min(Population,1)),topic:5},
MessageDef{text:"The environmental map explains why this lovely street is losing residents.",goal:goal!(1; Min(Pollution,40),Min(Population,1)),topic:5},
MessageDef{text:"A cleaner power source would be a fine gift to the downwind neighborhoods.",goal:goal!(1; Min(Pollution,40),Min(Population,1)),topic:5},
MessageDef{text:"The clinic queue reached the entrance before breakfast. We need more care nearby.",goal:goal!(1; Max(Health,55),Min(Population,1)),topic:6},
MessageDef{text:"A long ride to the hospital is the last thing a sick neighbor needs.",goal:goal!(1; Max(Health,55),Min(Population,1)),topic:6},
MessageDef{text:"Healthy neighborhoods need clean water as much as they need doctors.",goal:goal!(1; Max(Health,55),Min(Population,1)),topic:6},
MessageDef{text:"Our nurse is wonderful and has far too many patients.",goal:goal!(1; Max(Health,55),Min(Population,1)),topic:6},
MessageDef{text:"The park walking group could use safer air and a clinic within reach.",goal:goal!(1; Max(Health,55),Min(Population,1)),topic:6},
MessageDef{text:"Another family asked where the nearest working hospital is. That should be an easy answer.",goal:goal!(1; Max(Health,55),Min(Population,1)),topic:6},
MessageDef{text:"Please check hospital utilities. A building with no water cannot provide much care.",goal:goal!(1; Max(Health,55),Min(Population,1)),topic:6},
MessageDef{text:"Public health is a year-round job, not just an emergency headline.",goal:goal!(1; Max(Health,55),Min(Population,1)),topic:6},
MessageDef{text:"We organized rides for older neighbors. A local clinic would make those trips shorter.",goal:goal!(1; Max(Health,55),Min(Population,1)),topic:6},
MessageDef{text:"Happy to pay for care that is actually reachable from our street.",goal:goal!(1; Max(Health,55),Min(Population,1)),topic:6},
MessageDef{text:"The classroom has more pupils than coat hooks. Both matter on rainy days.",goal:goal!(1; Max(Education,50),Min(Population,1)),topic:7},
MessageDef{text:"Our apprentice wants to study here instead of moving away. Build that opportunity.",goal:goal!(1; Max(Education,50),Min(Population,1)),topic:7},
MessageDef{text:"A school on the map still needs funding, power and a route for families.",goal:goal!(1; Max(Education,50),Min(Population,1)),topic:7},
MessageDef{text:"The teacher bought supplies again. Please give education a dependable budget.",goal:goal!(1; Max(Education,50),Min(Population,1)),topic:7},
MessageDef{text:"The library reading circle is doing its best to fill a larger education gap.",goal:goal!(1; Max(Education,50),Min(Population,1)),topic:7},
MessageDef{text:"Local firms want skills that local children should be able to learn.",goal:goal!(1; Max(Education,50),Min(Population,1)),topic:7},
MessageDef{text:"New neighborhoods need school places before the children reach enrollment day.",goal:goal!(1; Max(Education,50),Min(Population,1)),topic:7},
MessageDef{text:"A university can bring talent, but the first classroom still comes first.",goal:goal!(1; Max(Education,50),Min(Population,1)),topic:7},
MessageDef{text:"My daughter asked why the science club meets in a hallway. Good question.",goal:goal!(1; Max(Education,50),Min(Population,1)),topic:7},
MessageDef{text:"Investing in students now sounds wiser than recruiting every specialist from elsewhere.",goal:goal!(1; Max(Education,50),Min(Population,1)),topic:7},
MessageDef{text:"The city feels easier to live in lately. Even the commute leaves room for dinner.",goal:goal!(1; Min(Happiness,70),Min(Population,1)),topic:8},
MessageDef{text:"We held a street party and ran out of chairs before we ran out of stories.",goal:goal!(1; Min(Happiness,70),Min(Population,1)),topic:8},
MessageDef{text:"Clean taps, steady lights and a nearby park. Quiet things can make a happy city.",goal:goal!(1; Min(Happiness,70),Min(Population,1)),topic:8},
MessageDef{text:"The neighborhood noticeboard is full of invitations instead of complaints.",goal:goal!(1; Min(Happiness,70),Min(Population,1)),topic:8},
MessageDef{text:"Friends visited for a weekend and started checking local apartment listings.",goal:goal!(1; Min(Happiness,70),Min(Population,1)),topic:8},
MessageDef{text:"I can imagine growing old on this street. That is a serious compliment.",goal:goal!(1; Min(Happiness,70),Min(Population,1)),topic:8},
MessageDef{text:"The market was busy, the bus was useful and nobody complained about the view.",goal:goal!(1; Min(Happiness,70),Min(Population,1)),topic:8},
MessageDef{text:"Thanks to everyone who keeps the everyday city working.",goal:goal!(1; Min(Happiness,70),Min(Population,1)),topic:8},
MessageDef{text:"There is a good kind of noise from the square tonight: people enjoying their city.",goal:goal!(1; Min(Happiness,70),Min(Population,1)),topic:8},
MessageDef{text:"For once, the residents meeting ended early because most things were working.",goal:goal!(1; Min(Happiness,70),Min(Population,1)),topic:8},
MessageDef{text:"The budget gap keeps growing. I would rather fix it before the bills choose for us.",goal:goal!(1; Max(NetIncome,-1),Min(Population,1)),topic:9},
MessageDef{text:"Ribbon cuttings are exciting; paying the monthly upkeep is part of the same project.",goal:goal!(1; Max(NetIncome,-1),Min(Population,1)),topic:9},
MessageDef{text:"The city cannot fund every wish this month. Please protect the essentials first.",goal:goal!(1; Max(NetIncome,-1),Min(Population,1)),topic:9},
MessageDef{text:"A small tax discussion now might avoid a much larger debt discussion later.",goal:goal!(1; Max(NetIncome,-1),Min(Population,1)),topic:9},
MessageDef{text:"We should know what a new building costs to run before buying it.",goal:goal!(1; Max(NetIncome,-1),Min(Population,1)),topic:9},
MessageDef{text:"Borrowing can build a bridge, but it cannot make the repayments disappear.",goal:goal!(1; Max(NetIncome,-1),Min(Population,1)),topic:9},
MessageDef{text:"The council budget meeting needs fewer slogans and more arithmetic.",goal:goal!(1; Max(NetIncome,-1),Min(Population,1)),topic:9},
MessageDef{text:"I support new services. I also support a plan to keep them open next year.",goal:goal!(1; Max(NetIncome,-1),Min(Population,1)),topic:9},
MessageDef{text:"Can we compare operating costs before the next expensive landmark?",goal:goal!(1; Max(NetIncome,-1),Min(Population,1)),topic:9},
MessageDef{text:"The treasury is asking for a quiet month. Maybe listen.",goal:goal!(1; Max(NetIncome,-1),Min(Population,1)),topic:9},
MessageDef{text:"Our block has workers ready to start. It needs reachable jobs.",goal:goal!(1; Min(Unemployment,15),Min(Population,1)),topic:10},
MessageDef{text:"The new apartments filled quickly. The help-wanted board did not.",goal:goal!(1; Min(Unemployment,15),Min(Population,1)),topic:10},
MessageDef{text:"I found a vacancy across town, but getting there would take most of the morning.",goal:goal!(1; Min(Unemployment,15),Min(Population,1)),topic:10},
MessageDef{text:"More shops and workshops would give this neighborhood a reason to stay.",goal:goal!(1; Min(Unemployment,15),Min(Population,1)),topic:10},
MessageDef{text:"My neighbor retrained for a job that has not arrived here yet.",goal:goal!(1; Min(Unemployment,15),Min(Population,1)),topic:10},
MessageDef{text:"Employment is infrastructure too. Homes need a working local economy.",goal:goal!(1; Min(Unemployment,15),Min(Population,1)),topic:10},
MessageDef{text:"The industrial park needs road and utility access before anyone can clock in.",goal:goal!(1; Min(Unemployment,15),Min(Population,1)),topic:10},
MessageDef{text:"Would love to spend my skills here instead of sending applications away.",goal:goal!(1; Min(Unemployment,15),Min(Population,1)),topic:10},
MessageDef{text:"The cafe cannot hire until enough customers have wages again.",goal:goal!(1; Min(Unemployment,15),Min(Population,1)),topic:10},
MessageDef{text:"Please balance the new homes with jobs that residents can actually reach.",goal:goal!(1; Min(Unemployment,15),Min(Population,1)),topic:10},
MessageDef{text:"The smallest park on the block has become the biggest part of our evening.",goal:goal!(1; Min(Parks,3),Min(Population,1)),topic:11},
MessageDef{text:"The bench under the trees has an unofficial waiting list.",goal:goal!(1; Min(Parks,3),Min(Population,1)),topic:11},
MessageDef{text:"We found a new path through town by following the green spaces.",goal:goal!(1; Min(Parks,3),Min(Population,1)),topic:11},
MessageDef{text:"A child counted nine kinds of leaf on the walk home. Better than counting parked cars.",goal:goal!(1; Min(Parks,3),Min(Population,1)),topic:11},
MessageDef{text:"The public garden has made strangers into familiar faces.",goal:goal!(1; Min(Parks,3),Min(Population,1)),topic:11},
MessageDef{text:"A little shade makes a long walk feel like a short one.",goal:goal!(1; Min(Parks,3),Min(Population,1)),topic:11},
MessageDef{text:"The park hosts chess, picnics and exactly one very opinionated pigeon.",goal:goal!(1; Min(Parks,3),Min(Population,1)),topic:11},
MessageDef{text:"We held the neighborhood meeting outside. Best attendance yet.",goal:goal!(1; Min(Parks,3),Min(Population,1)),topic:11},
MessageDef{text:"The playground is busy enough to deserve another one in the next district.",goal:goal!(1; Min(Parks,3),Min(Population,1)),topic:11},
MessageDef{text:"Green space is where the city takes a breath between buildings.",goal:goal!(1; Min(Parks,3),Min(Population,1)),topic:11},
MessageDef{text:"The shopkeepers are closing early again. Safer streets would help the whole block.",goal:goal!(1; Min(Crime,40),Min(Population,1)),topic:12},
MessageDef{text:"A nearby police building matters only if officers can reach our neighborhood.",goal:goal!(1; Min(Crime,40),Min(Population,1)),topic:12},
MessageDef{text:"Trust grows when public safety works with the people who live here.",goal:goal!(1; Min(Crime,40),Min(Population,1)),topic:12},
MessageDef{text:"The neighborhood watch needs a dependable contact at the station.",goal:goal!(1; Min(Crime,40),Min(Population,1)),topic:12},
MessageDef{text:"Empty workplaces and worried families are feeding the same problem.",goal:goal!(1; Min(Crime,40),Min(Population,1)),topic:12},
MessageDef{text:"Please fix the causes of crime as well as the response times.",goal:goal!(1; Min(Crime,40),Min(Population,1)),topic:12},
MessageDef{text:"The evening bus stop needs people to feel safe waiting there.",goal:goal!(1; Min(Crime,40),Min(Population,1)),topic:12},
MessageDef{text:"Community meetings are asking for accountable policing and more opportunity.",goal:goal!(1; Min(Crime,40),Min(Population,1)),topic:12},
MessageDef{text:"Our street should feel safe after the shops close.",goal:goal!(1; Min(Crime,40),Min(Population,1)),topic:12},
MessageDef{text:"A funded, connected police station would be a welcome start.",goal:goal!(1; Min(Crime,40),Min(Population,1)),topic:12},
MessageDef{text:"The old shopfront is protected now. My grandmother remembers its first owner.",goal:goal!(1; Min(HeritageSites,1),Min(Population,1)),topic:13},
MessageDef{text:"Visitors stopped to photograph a building we used to hurry past.",goal:goal!(1; Min(HeritageSites,1),Min(Population,1)),topic:13},
MessageDef{text:"Preservation works best when the old neighborhood still has a future.",goal:goal!(1; Min(HeritageSites,1),Min(Population,1)),topic:13},
MessageDef{text:"The historic bakery smells exactly as it did when I was small.",goal:goal!(1; Min(HeritageSites,1),Min(Population,1)),topic:13},
MessageDef{text:"Our district has a story you can read in the buildings.",goal:goal!(1; Min(HeritageSites,1),Min(Population,1)),topic:13},
MessageDef{text:"Keeping an old home alive takes repairs, not just a plaque.",goal:goal!(1; Min(HeritageSites,1),Min(Population,1)),topic:13},
MessageDef{text:"The heritage walk ends at a very modern cafe. Both are welcome.",goal:goal!(1; Min(HeritageSites,1),Min(Population,1)),topic:13},
MessageDef{text:"A familiar roofline survived another redevelopment plan. Glad we made room for it.",goal:goal!(1; Min(HeritageSites,1),Min(Population,1)),topic:13},
MessageDef{text:"We are collecting neighborhood stories for the civic museum.",goal:goal!(1; Min(HeritageSites,1),Min(Population,1)),topic:13},
MessageDef{text:"The city can grow without pretending it began yesterday.",goal:goal!(1; Min(HeritageSites,1),Min(Population,1)),topic:13},
MessageDef{text:"Another council meeting tonight. Please publish the numbers as clearly as the promises.",goal:goal!(1; Min(Months,12),Min(Population,1)),topic:14},
MessageDef{text:"I judge the mayor by the tap, the bus and the bill. All three get a vote.",goal:goal!(1; Min(Months,12),Min(Population,1)),topic:14},
MessageDef{text:"The neighborhood forum has ideas for the next budget. Come listen.",goal:goal!(1; Min(Months,12),Min(Population,1)),topic:14},
MessageDef{text:"A good election campaign should explain what the city can actually afford.",goal:goal!(1; Min(Months,12),Min(Population,1)),topic:14},
MessageDef{text:"Our district wants a voice before the route is decided, not after the concrete arrives.",goal:goal!(1; Min(Months,12),Min(Population,1)),topic:14},
MessageDef{text:"Council minutes are surprisingly interesting when your street is mentioned.",goal:goal!(1; Min(Months,12),Min(Population,1)),topic:14},
MessageDef{text:"Popularity is a useful signal. Reliable services are the work behind it.",goal:goal!(1; Min(Months,12),Min(Population,1)),topic:14},
MessageDef{text:"The city is old enough to have traditions and young enough to improve them.",goal:goal!(1; Min(Months,12),Min(Population,1)),topic:14},
MessageDef{text:"We asked for a public progress report. The mayor brought graphs. Promising start.",goal:goal!(1; Min(Months,12),Min(Population,1)),topic:14},
MessageDef{text:"My ballot has room for clean water, steady jobs and a plan for the next storm.",goal:goal!(1; Min(Months,12),Min(Population,1)),topic:14},
];
pub const ADVISORS:[AdvisorLineDef;76]=[
AdvisorLineDef{name:"Planning",text:"Zone homes beside a connected road network; isolated green squares cannot become a neighborhood."},
AdvisorLineDef{name:"Planning",text:"Growth needs homes, jobs and customers together. Read all three demand bars before adding large zones."},
AdvisorLineDef{name:"Planning",text:"Empty zones can be healthy spare capacity. Check utilities before trying to force their development."},
AdvisorLineDef{name:"Planning",text:"Higher land value supports denser buildings when education and city size also allow it."},
AdvisorLineDef{name:"Planning",text:"Build small additions and let the monthly cycle show whether the town can support them."},
AdvisorLineDef{name:"Planning",text:"Protect room for roads and services before filling every available block with housing."},
AdvisorLineDef{name:"Planning",text:"A district should describe a real neighborhood. Give it connected land and a clear purpose."},
AdvisorLineDef{name:"Planning",text:"Heritage designation is a continuing commitment; its maintenance belongs in the ordinary budget."},
AdvisorLineDef{name:"Planning",text:"Population milestones award grants once. A population dip cannot earn the same grant again."},
AdvisorLineDef{name:"Planning",text:"A town is more resilient when new development has more than one useful route to the rest of the city."},
AdvisorLineDef{name:"Utilities",text:"Power stations need a connected distribution network; capacity alone does not serve a remote house."},
AdvisorLineDef{name:"Utilities",text:"Water pumps require electricity, a nearby water source and a connected delivery network."},
AdvisorLineDef{name:"Utilities",text:"If supply meets demand but homes are unserved, look for a broken utility connection."},
AdvisorLineDef{name:"Utilities",text:"Growing zones increase consumption. Leave a margin before approving another neighborhood."},
AdvisorLineDef{name:"Utilities",text:"Roads carry municipal conduits; separate power lines and pipes can extend them beyond the street grid."},
AdvisorLineDef{name:"Utilities",text:"Conservation ordinances reduce demand, but their recurring cost still needs to be funded."},
AdvisorLineDef{name:"Utilities",text:"A citywide supply percentage can hide one isolated block. Inspect the utility maps too."},
AdvisorLineDef{name:"Utilities",text:"Water and sewage are different responsibilities. Adequate pumping does not remove wastewater."},
AdvisorLineDef{name:"Utilities",text:"Waste facilities must be connected and operating; a distant unserved landfill contributes little."},
AdvisorLineDef{name:"Utilities",text:"Recycling extends disposal capacity, while a new facility addresses a larger absolute waste load."},
AdvisorLineDef{name:"Utilities",text:"Cheap generation can carry expensive environmental costs for the neighborhoods downwind."},
AdvisorLineDef{name:"Utilities",text:"Restore utilities after a disaster before expecting repaired homes and services to recover."},
AdvisorLineDef{name:"Transport",text:"Traffic is measured on operating roads. A queue needs another route, more capacity or fewer car trips."},
AdvisorLineDef{name:"Transport",text:"Place transit where homes and workplaces can reach it; empty land does not generate many passengers."},
AdvisorLineDef{name:"Transport",text:"A road to the map edge provides the regional connection a new settlement needs."},
AdvisorLineDef{name:"Transport",text:"Avenues carry more traffic but cost more to build and maintain."},
AdvisorLineDef{name:"Transport",text:"Rail stations need a continuous useful railway and accessible local roads."},
AdvisorLineDef{name:"Transport",text:"Free transit attracts riders but transfers its fares into the public budget."},
AdvisorLineDef{name:"Transport",text:"Freight and commuters competing for one narrow bridge create a predictable bottleneck."},
AdvisorLineDef{name:"Transport",text:"A bus depot without water or electricity is a building, not a transport service."},
AdvisorLineDef{name:"Transport",text:"Connect both sides of a bridge before judging the development on the far bank."},
AdvisorLineDef{name:"Transport",text:"Transport funding changes effective network capacity; cutting it can be more costly than it looks."},
AdvisorLineDef{name:"Finance",text:"Your monthly balance is income minus running costs. Construction spending is separate."},
AdvisorLineDef{name:"Finance",text:"A loan or bond funds investment; interest remains due even while the calendar is paused for planning."},
AdvisorLineDef{name:"Finance",text:"Keep an emergency reserve instead of spending every population grant on expansion."},
AdvisorLineDef{name:"Finance",text:"Higher taxes raise revenue per resident but can weaken demand and satisfaction."},
AdvisorLineDef{name:"Finance",text:"Bonds have a maturity date. Plan for principal repayment before the final month arrives."},
AdvisorLineDef{name:"Finance",text:"Bankruptcy cancels debt at the cost of trust and a period of austerity; it is a last resort."},
AdvisorLineDef{name:"Finance",text:"An unfunded ordinance is not free. Its monthly charge scales with the population it serves."},
AdvisorLineDef{name:"Finance",text:"A megaproject reserves a slot and needs a sequence of payments, not one ceremonial announcement."},
AdvisorLineDef{name:"Finance",text:"Repaying expensive debt improves the room available for ordinary services."},
AdvisorLineDef{name:"Finance",text:"Judge a service cut by its effect on the whole city, not only its entry in this month's ledger."},
AdvisorLineDef{name:"Public Safety",text:"Fire protection relies on a funded station that can reach the neighborhood by road."},
AdvisorLineDef{name:"Public Safety",text:"A disaster warning gives time to prepare. Do not spend that time building in the impact area."},
AdvisorLineDef{name:"Public Safety",text:"Emergency drills improve preparedness for a limited time and have a real budget cost."},
AdvisorLineDef{name:"Public Safety",text:"Relief is available for a recent damaging emergency; it cannot be claimed repeatedly for one event."},
AdvisorLineDef{name:"Public Safety",text:"Repairing a damaged structure restores its function gradually instead of instantly restoring all residents."},
AdvisorLineDef{name:"Public Safety",text:"Industrial safety lowers exposure around factories and dangerous power facilities."},
AdvisorLineDef{name:"Public Safety",text:"Crime responds to services, employment and neighborhood conditions together."},
AdvisorLineDef{name:"Public Safety",text:"Police accountability improves trust while maintaining the need for actual local coverage."},
AdvisorLineDef{name:"Public Safety",text:"Keep recovery routes and utility links clear; one damaged crossing can isolate several services."},
AdvisorLineDef{name:"Public Safety",text:"A reactor emergency requires a reactor. A city without one cannot have a local meltdown."},
AdvisorLineDef{name:"Health",text:"Hospitals must be reachable and supplied; build care where people actually live."},
AdvisorLineDef{name:"Health",text:"Water shortages, waste and pollution can undermine health even beside a clinic."},
AdvisorLineDef{name:"Health",text:"Public health outreach complements hospitals; it does not replace their beds."},
AdvisorLineDef{name:"Health",text:"Childcare supports families and helps residents remain active in the local economy."},
AdvisorLineDef{name:"Health",text:"Health improvements take a consistent service budget rather than a single busy month."},
AdvisorLineDef{name:"Health",text:"A quieter residential district can improve wellbeing near disruptive activity."},
AdvisorLineDef{name:"Health",text:"School meals benefit health as well as classroom outcomes."},
AdvisorLineDef{name:"Health",text:"Following a disaster, repair essential utilities before opening damaged health facilities."},
AdvisorLineDef{name:"Education",text:"Primary schools support early growth; universities support the skills needed for advanced development."},
AdvisorLineDef{name:"Education",text:"A growing residential district needs new classroom capacity before the old school becomes overloaded."},
AdvisorLineDef{name:"Education",text:"Education funding affects how far institutions can serve surrounding neighborhoods."},
AdvisorLineDef{name:"Education",text:"An operating university is necessary for several advanced civic projects and rewards."},
AdvisorLineDef{name:"Education",text:"A central library is earned through sustained learning outcomes and existing school access."},
AdvisorLineDef{name:"Education",text:"School meals help students learn and remain healthy; include their ongoing cost in the budget."},
AdvisorLineDef{name:"Education",text:"Local employers benefit when training and employment opportunities grow together."},
AdvisorLineDef{name:"Education",text:"A science center is a long investment. Keep ordinary schools funded while it is being built."},
AdvisorLineDef{name:"Environment",text:"Industrial pollution should be separated from homes, healthcare and prized waterfronts."},
AdvisorLineDef{name:"Environment",text:"Parks improve nearby land value and give dense neighborhoods somewhere to breathe."},
AdvisorLineDef{name:"Environment",text:"Trees and green districts help the environment but cannot replace clean utilities."},
AdvisorLineDef{name:"Environment",text:"Clean air rules reduce industrial pollution while adding inspection costs."},
AdvisorLineDef{name:"Environment",text:"Water conservation reduces demand; it cannot reconnect a broken pipe."},
AdvisorLineDef{name:"Environment",text:"Preserved buildings bring cultural value and visitor activity when the neighborhood remains accessible."},
AdvisorLineDef{name:"Environment",text:"A botanical garden rewards a citywide commitment to green space and cleaner air."},
AdvisorLineDef{name:"Environment",text:"Check population-weighted pollution. Empty clean land should not hide an unhealthy residential district."},
];
pub const ADVISOR_TRIGGERS: [AdvisorTrigger; 90] = [
    AdvisorTrigger {
        line: 0,
        priority: 3,
        goal: goal!(1; Min(ResidentialTiles,1),Max(PowerCoverage,90)),
    },
    AdvisorTrigger {
        line: 1,
        priority: 3,
        goal: goal!(1; Min(Unemployment,12)),
    },
    AdvisorTrigger {
        line: 2,
        priority: 3,
        goal: goal!(1; Max(WaterCoverage,85)),
    },
    AdvisorTrigger {
        line: 3,
        priority: 1,
        goal: goal!(1; Min(LandValue,60)),
    },
    AdvisorTrigger {
        line: 4,
        priority: 1,
        goal: goal!(1; Max(Treasury,15000)),
    },
    AdvisorTrigger {
        line: 5,
        priority: 1,
        goal: goal!(1; Min(Population,1500)),
    },
    AdvisorTrigger {
        line: 6,
        priority: 1,
        goal: goal!(1; Min(Population,3000)),
    },
    AdvisorTrigger {
        line: 7,
        priority: 1,
        goal: goal!(1; Min(HeritageSites,1)),
    },
    AdvisorTrigger {
        line: 8,
        priority: 1,
        goal: goal!(1; Min(Population,500)),
    },
    AdvisorTrigger {
        line: 9,
        priority: 1,
        goal: goal!(1; Min(Traffic,45)),
    },
    AdvisorTrigger {
        line: 10,
        priority: 3,
        goal: goal!(1; Max(PowerCoverage,95)),
    },
    AdvisorTrigger {
        line: 11,
        priority: 3,
        goal: goal!(1; Max(WaterCoverage,95)),
    },
    AdvisorTrigger {
        line: 12,
        priority: 1,
        goal: goal!(1; Min(PowerCoverage,90),Max(WaterCoverage,80)),
    },
    AdvisorTrigger {
        line: 13,
        priority: 1,
        goal: goal!(1; Min(Population,3000),Max(PowerCoverage,95)),
    },
    AdvisorTrigger {
        line: 14,
        priority: 1,
        goal: goal!(1; Buildings(Kind::PowerLine,1)),
    },
    AdvisorTrigger {
        line: 15,
        priority: 1,
        goal: goal!(1; Policy(OrdinanceId::WaterConservation)),
    },
    AdvisorTrigger {
        line: 16,
        priority: 1,
        goal: goal!(1; Min(Population,1000),Max(PowerCoverage,99)),
    },
    AdvisorTrigger {
        line: 17,
        priority: 1,
        goal: goal!(1; Min(Garbage,30)),
    },
    AdvisorTrigger {
        line: 18,
        priority: 1,
        goal: goal!(1; Min(Garbage,45)),
    },
    AdvisorTrigger {
        line: 19,
        priority: 1,
        goal: goal!(1; Policy(OrdinanceId::Recycling)),
    },
    AdvisorTrigger {
        line: 20,
        priority: 1,
        goal: goal!(1; Buildings(Kind::Coal,1)),
    },
    AdvisorTrigger {
        line: 21,
        priority: 4,
        goal: goal!(1; Min(UnrepairedDamage,1),Max(PowerCoverage,95)),
    },
    AdvisorTrigger {
        line: 22,
        priority: 1,
        goal: goal!(1; Min(Traffic,55)),
    },
    AdvisorTrigger {
        line: 23,
        priority: 1,
        goal: goal!(1; Max(Transit,20),Min(Population,2000)),
    },
    AdvisorTrigger {
        line: 24,
        priority: 1,
        goal: goal!(1; Max(Population,1500)),
    },
    AdvisorTrigger {
        line: 25,
        priority: 1,
        goal: goal!(1; Min(Traffic,70)),
    },
    AdvisorTrigger {
        line: 26,
        priority: 1,
        goal: goal!(1; Buildings(Kind::Station,1)),
    },
    AdvisorTrigger {
        line: 27,
        priority: 1,
        goal: goal!(1; Policy(OrdinanceId::FreeTransit)),
    },
    AdvisorTrigger {
        line: 28,
        priority: 1,
        goal: goal!(1; Min(IndustrialJobs,2000),Min(Traffic,50)),
    },
    AdvisorTrigger {
        line: 29,
        priority: 1,
        goal: goal!(1; Buildings(Kind::BusDepot,1),Max(Transit,10)),
    },
    AdvisorTrigger {
        line: 30,
        priority: 1,
        goal: goal!(1; Buildings(Kind::Road,30)),
    },
    AdvisorTrigger {
        line: 31,
        priority: 1,
        goal: goal!(1; Min(Traffic,60)),
    },
    AdvisorTrigger {
        line: 32,
        priority: 3,
        goal: goal!(1; Max(NetIncome,-1)),
    },
    AdvisorTrigger {
        line: 33,
        priority: 1,
        goal: goal!(1; Min(Debt,10000)),
    },
    AdvisorTrigger {
        line: 34,
        priority: 1,
        goal: goal!(1; Max(Treasury,10000)),
    },
    AdvisorTrigger {
        line: 35,
        priority: 1,
        goal: goal!(1; Max(Happiness,50)),
    },
    AdvisorTrigger {
        line: 36,
        priority: 1,
        goal: goal!(1; Min(BondsOutstanding,1)),
    },
    AdvisorTrigger {
        line: 37,
        priority: 3,
        goal: goal!(1; Max(Treasury,0)),
    },
    AdvisorTrigger {
        line: 38,
        priority: 1,
        goal: goal!(1; Min(Months,12)),
    },
    AdvisorTrigger {
        line: 39,
        priority: 1,
        goal: goal!(1; Min(CompletedProjects,1)),
    },
    AdvisorTrigger {
        line: 40,
        priority: 1,
        goal: goal!(1; Min(Treasury,30000),Min(Debt,1)),
    },
    AdvisorTrigger {
        line: 41,
        priority: 1,
        goal: goal!(1; Max(Health,45)),
    },
    AdvisorTrigger {
        line: 42,
        priority: 3,
        goal: goal!(1; Buildings(Kind::Fire,1)),
    },
    AdvisorTrigger {
        line: 43,
        priority: 5,
        goal: goal!(1; Min(ActiveWarnings,1)),
    },
    AdvisorTrigger {
        line: 44,
        priority: 1,
        goal: goal!(1; Min(Months,6)),
    },
    AdvisorTrigger {
        line: 45,
        priority: 4,
        goal: goal!(1; Min(ReliefAvailable,1)),
    },
    AdvisorTrigger {
        line: 46,
        priority: 3,
        goal: goal!(1; Event(CivicEvent::RepairedDamage,1)),
    },
    AdvisorTrigger {
        line: 47,
        priority: 1,
        goal: goal!(1; Buildings(Kind::Industrial,10)),
    },
    AdvisorTrigger {
        line: 48,
        priority: 3,
        goal: goal!(1; Min(Crime,40)),
    },
    AdvisorTrigger {
        line: 49,
        priority: 1,
        goal: goal!(1; Policy(OrdinanceId::PoliceAccountability)),
    },
    AdvisorTrigger {
        line: 50,
        priority: 1,
        goal: goal!(1; Max(PowerCoverage,75)),
    },
    AdvisorTrigger {
        line: 51,
        priority: 1,
        goal: goal!(1; Buildings(Kind::Nuclear,1)),
    },
    AdvisorTrigger {
        line: 52,
        priority: 3,
        goal: goal!(1; Max(Health,60)),
    },
    AdvisorTrigger {
        line: 53,
        priority: 3,
        goal: goal!(1; Min(Pollution,35)),
    },
    AdvisorTrigger {
        line: 54,
        priority: 1,
        goal: goal!(1; Policy(OrdinanceId::PublicHealth)),
    },
    AdvisorTrigger {
        line: 55,
        priority: 1,
        goal: goal!(1; Policy(OrdinanceId::Childcare)),
    },
    AdvisorTrigger {
        line: 56,
        priority: 1,
        goal: goal!(1; Max(Health,75),Min(Population,3000)),
    },
    AdvisorTrigger {
        line: 57,
        priority: 1,
        goal: goal!(1; Policy(OrdinanceId::NoiseControl)),
    },
    AdvisorTrigger {
        line: 58,
        priority: 1,
        goal: goal!(1; Policy(OrdinanceId::SchoolMeals)),
    },
    AdvisorTrigger {
        line: 59,
        priority: 1,
        goal: goal!(1; Event(CivicEvent::SurvivedDisaster,1),Max(Health,70)),
    },
    AdvisorTrigger {
        line: 60,
        priority: 1,
        goal: goal!(1; Max(Education,50)),
    },
    AdvisorTrigger {
        line: 61,
        priority: 1,
        goal: goal!(1; Min(Population,3000),Max(Education,60)),
    },
    AdvisorTrigger {
        line: 62,
        priority: 1,
        goal: goal!(1; Max(Education,45)),
    },
    AdvisorTrigger {
        line: 63,
        priority: 1,
        goal: goal!(1; Min(Population,10000),Operational(Kind::University,1)),
    },
    AdvisorTrigger {
        line: 64,
        priority: 1,
        goal: goal!(1; Min(Education,55)),
    },
    AdvisorTrigger {
        line: 65,
        priority: 1,
        goal: goal!(1; Policy(OrdinanceId::SchoolMeals),Max(Education,70)),
    },
    AdvisorTrigger {
        line: 66,
        priority: 1,
        goal: goal!(1; Min(Unemployment,15),Min(Education,60)),
    },
    AdvisorTrigger {
        line: 67,
        priority: 1,
        goal: goal!(1; Min(Population,30000)),
    },
    AdvisorTrigger {
        line: 68,
        priority: 1,
        goal: goal!(1; Min(Pollution,40)),
    },
    AdvisorTrigger {
        line: 69,
        priority: 1,
        goal: goal!(1; Max(LandValue,35)),
    },
    AdvisorTrigger {
        line: 70,
        priority: 1,
        goal: goal!(1; Min(ForestTiles,50)),
    },
    AdvisorTrigger {
        line: 71,
        priority: 1,
        goal: goal!(1; Policy(OrdinanceId::CleanAir)),
    },
    AdvisorTrigger {
        line: 72,
        priority: 1,
        goal: goal!(1; Policy(OrdinanceId::WaterConservation),Max(WaterCoverage,90)),
    },
    AdvisorTrigger {
        line: 73,
        priority: 1,
        goal: goal!(1; Min(HeritageSites,1),Policy(OrdinanceId::Heritage),Min(Transit,20)),
    },
    AdvisorTrigger {
        line: 74,
        priority: 1,
        goal: goal!(1; Min(Parks,5)),
    },
    AdvisorTrigger {
        line: 75,
        priority: 1,
        goal: goal!(1; Min(Pollution,55)),
    },
    AdvisorTrigger {
        line: 0,
        priority: 5,
        goal: goal!(1; Max(PowerCoverage,50),Min(Population,100)),
    },
    AdvisorTrigger {
        line: 1,
        priority: 5,
        goal: goal!(1; Max(Unemployment,4),Min(Population,10000)),
    },
    AdvisorTrigger {
        line: 10,
        priority: 5,
        goal: goal!(1; Max(PowerCoverage,25)),
    },
    AdvisorTrigger {
        line: 11,
        priority: 5,
        goal: goal!(1; Max(WaterCoverage,25)),
    },
    AdvisorTrigger {
        line: 18,
        priority: 5,
        goal: goal!(1; Min(Garbage,80)),
    },
    AdvisorTrigger {
        line: 22,
        priority: 5,
        goal: goal!(1; Min(Traffic,90)),
    },
    AdvisorTrigger {
        line: 25,
        priority: 5,
        goal: goal!(1; Min(Traffic,85),Buildings(Kind::Avenue,1)),
    },
    AdvisorTrigger {
        line: 32,
        priority: 5,
        goal: goal!(1; Max(NetIncome,-1000)),
    },
    AdvisorTrigger {
        line: 33,
        priority: 5,
        goal: goal!(1; Min(BondsOutstanding,50000)),
    },
    AdvisorTrigger {
        line: 37,
        priority: 5,
        goal: goal!(1; Event(CivicEvent::DeclaredBankruptcy,1)),
    },
    AdvisorTrigger {
        line: 46,
        priority: 5,
        goal: goal!(1; Event(CivicEvent::SurvivedDisaster,3)),
    },
    AdvisorTrigger {
        line: 52,
        priority: 5,
        goal: goal!(1; Max(Health,25)),
    },
    AdvisorTrigger {
        line: 60,
        priority: 5,
        goal: goal!(1; Max(Education,20)),
    },
    AdvisorTrigger {
        line: 68,
        priority: 5,
        goal: goal!(1; Min(Pollution,80)),
    },
];
pub const HEADLINES: [MessageDef; 61] = [
    MessageDef {
        text: "New Neighbors Put BlueCity on the Map",
        goal: goal!(1; Min(Population,100)),
        topic: 0,
    },
    MessageDef {
        text: "Market Streets Welcome a New Generation",
        goal: goal!(1; Min(Population,100)),
        topic: 0,
    },
    MessageDef {
        text: "Building Boom Brings Questions About Services",
        goal: goal!(1; Min(Population,100)),
        topic: 0,
    },
    MessageDef {
        text: "A Growing Town Finds Its Own Identity",
        goal: goal!(1; Min(Population,100)),
        topic: 0,
    },
    MessageDef {
        text: "Council Celebrates the People Behind the Population",
        goal: goal!(1; Min(Population,100)),
        topic: 0,
    },
    MessageDef {
        text: "Outages Expose Gaps in the Power Network",
        goal: goal!(1; Max(PowerCoverage,85)),
        topic: 1,
    },
    MessageDef {
        text: "Businesses Ask for a Reliable Electricity Plan",
        goal: goal!(1; Max(PowerCoverage,85)),
        topic: 1,
    },
    MessageDef {
        text: "Generator Capacity Outruns Local Connections",
        goal: goal!(1; Max(PowerCoverage,85)),
        topic: 1,
    },
    MessageDef {
        text: "Utility Crews Trace a Broken Link",
        goal: goal!(1; Max(PowerCoverage,85)),
        topic: 1,
    },
    MessageDef {
        text: "New Development Tests the City Grid",
        goal: goal!(1; Max(PowerCoverage,85)),
        topic: 1,
    },
    MessageDef {
        text: "Low Pressure Raises Questions at City Hall",
        goal: goal!(1; Max(WaterCoverage,85)),
        topic: 2,
    },
    MessageDef {
        text: "Residents Call for Investment in Water Supply",
        goal: goal!(1; Max(WaterCoverage,85)),
        topic: 2,
    },
    MessageDef {
        text: "Pump Connections Become a Public Priority",
        goal: goal!(1; Max(WaterCoverage,85)),
        topic: 2,
    },
    MessageDef {
        text: "A Growing City Needs More Than New Pipes",
        goal: goal!(1; Max(WaterCoverage,85)),
        topic: 2,
    },
    MessageDef {
        text: "Council Reviews Water Shortage Response",
        goal: goal!(1; Max(WaterCoverage,85)),
        topic: 2,
    },
    MessageDef {
        text: "Commuters Ask for an Alternative to the Queue",
        goal: goal!(1; Min(Traffic,55)),
        topic: 3,
    },
    MessageDef {
        text: "Bridge Bottleneck Dominates Transport Debate",
        goal: goal!(1; Min(Traffic,55)),
        topic: 3,
    },
    MessageDef {
        text: "Freight and Families Compete for Road Space",
        goal: goal!(1; Min(Traffic,55)),
        topic: 3,
    },
    MessageDef {
        text: "Gridlock Puts Road Funding Under Scrutiny",
        goal: goal!(1; Min(Traffic,55)),
        topic: 3,
    },
    MessageDef {
        text: "The Search for a Shorter Journey Continues",
        goal: goal!(1; Min(Traffic,55)),
        topic: 3,
    },
    MessageDef {
        text: "More Residents Choose Public Transport",
        goal: goal!(1; Min(Transit,25)),
        topic: 4,
    },
    MessageDef {
        text: "Station District Welcomes New Footfall",
        goal: goal!(1; Min(Transit,25)),
        topic: 4,
    },
    MessageDef {
        text: "A Useful Bus Route Changes Daily Routines",
        goal: goal!(1; Min(Transit,25)),
        topic: 4,
    },
    MessageDef {
        text: "Transit Riders Bring Life to Local Shops",
        goal: goal!(1; Min(Transit,25)),
        topic: 4,
    },
    MessageDef {
        text: "Council Hears Calls for Better Connections",
        goal: goal!(1; Min(Transit,25)),
        topic: 4,
    },
    MessageDef {
        text: "Air Quality Becomes a Neighborhood Concern",
        goal: goal!(1; Min(Pollution,40)),
        topic: 5,
    },
    MessageDef {
        text: "Industrial Success Carries an Environmental Bill",
        goal: goal!(1; Min(Pollution,40)),
        topic: 5,
    },
    MessageDef {
        text: "Schools Adjust Outdoor Activities as Pollution Rises",
        goal: goal!(1; Min(Pollution,40)),
        topic: 5,
    },
    MessageDef {
        text: "Residents Seek Cleaner Growth",
        goal: goal!(1; Min(Pollution,40)),
        topic: 5,
    },
    MessageDef {
        text: "Downwind Districts Call for Fair Treatment",
        goal: goal!(1; Min(Pollution,40)),
        topic: 5,
    },
    MessageDef {
        text: "Budget Deficit Tests the Mayor's Priorities",
        goal: goal!(1; Max(NetIncome,-1)),
        topic: 6,
    },
    MessageDef {
        text: "Council Weighs Service Costs Against Revenue",
        goal: goal!(1; Max(NetIncome,-1)),
        topic: 6,
    },
    MessageDef {
        text: "Operating Bills Outpace the Tax Take",
        goal: goal!(1; Max(NetIncome,-1)),
        topic: 6,
    },
    MessageDef {
        text: "Treasury Warning Prompts Public Debate",
        goal: goal!(1; Max(NetIncome,-1)),
        topic: 6,
    },
    MessageDef {
        text: "City Seeks a Sustainable Path to Growth",
        goal: goal!(1; Max(NetIncome,-1)),
        topic: 6,
    },
    MessageDef {
        text: "Public Health Improvements Reach More Families",
        goal: goal!(1; Min(Health,70)),
        topic: 7,
    },
    MessageDef {
        text: "Care Teams Credit Consistent Local Investment",
        goal: goal!(1; Min(Health,70)),
        topic: 7,
    },
    MessageDef {
        text: "A Healthier City Looks Toward Prevention",
        goal: goal!(1; Min(Health,70)),
        topic: 7,
    },
    MessageDef {
        text: "Neighborhood Clinics Report Stronger Outcomes",
        goal: goal!(1; Min(Health,70)),
        topic: 7,
    },
    MessageDef {
        text: "Clean Utilities Join the Health Success Story",
        goal: goal!(1; Min(Health,70)),
        topic: 7,
    },
    MessageDef {
        text: "Students Find New Opportunities Close to Home",
        goal: goal!(1; Min(Education,65)),
        topic: 8,
    },
    MessageDef {
        text: "Employers Welcome a Better Skilled Workforce",
        goal: goal!(1; Min(Education,65)),
        topic: 8,
    },
    MessageDef {
        text: "Education Investment Begins to Pay Off",
        goal: goal!(1; Min(Education,65)),
        topic: 8,
    },
    MessageDef {
        text: "University Links Strengthen the Local Economy",
        goal: goal!(1; Min(Education,65)),
        topic: 8,
    },
    MessageDef {
        text: "Teachers Celebrate a City That Keeps Learning",
        goal: goal!(1; Min(Education,65)),
        topic: 8,
    },
    MessageDef {
        text: "Historic Streets Gain a Place in the City's Future",
        goal: goal!(1; Min(HeritageSites,1)),
        topic: 9,
    },
    MessageDef {
        text: "Preservation Plan Supports Familiar Shopfronts",
        goal: goal!(1; Min(HeritageSites,1)),
        topic: 9,
    },
    MessageDef {
        text: "Residents Share the Stories Behind Old Buildings",
        goal: goal!(1; Min(HeritageSites,1)),
        topic: 9,
    },
    MessageDef {
        text: "Heritage Visitors Discover Living Neighborhoods",
        goal: goal!(1; Min(HeritageSites,1)),
        topic: 9,
    },
    MessageDef {
        text: "Council Balances New Homes With Local History",
        goal: goal!(1; Min(HeritageSites,1)),
        topic: 9,
    },
    MessageDef {
        text: "Recovery Teams Reconnect the City",
        goal: goal!(1; Event(CivicEvent::SurvivedDisaster,1)),
        topic: 10,
    },
    MessageDef {
        text: "Neighbors Share Tools After the Emergency",
        goal: goal!(1; Event(CivicEvent::SurvivedDisaster,1)),
        topic: 10,
    },
    MessageDef {
        text: "Relief Funding Faces the Test of Real Repairs",
        goal: goal!(1; Event(CivicEvent::SurvivedDisaster,1)),
        topic: 10,
    },
    MessageDef {
        text: "Council Reviews Lessons From the Latest Disaster",
        goal: goal!(1; Event(CivicEvent::SurvivedDisaster,1)),
        topic: 10,
    },
    MessageDef {
        text: "A Rebuilt Street Marks a Step Toward Recovery",
        goal: goal!(1; Event(CivicEvent::SurvivedDisaster,1)),
        topic: 10,
    },
    MessageDef {
        text: "Public Trust Strengthens the Mayor's Mandate",
        goal: goal!(1; Min(Approval,65)),
        topic: 11,
    },
    MessageDef {
        text: "Residents Reward Steady Services With Support",
        goal: goal!(1; Min(Approval,65)),
        topic: 11,
    },
    MessageDef {
        text: "Council Publishes the Next Term's Priorities",
        goal: goal!(1; Min(Approval,65)),
        topic: 11,
    },
    MessageDef {
        text: "A Confident City Debates Its Next Big Investment",
        goal: goal!(1; Min(Approval,65)),
        topic: 11,
    },
    MessageDef {
        text: "The Everyday Work of Government Wins Notice",
        goal: goal!(1; Min(Approval,65)),
        topic: 11,
    },
    MessageDef {
        text: "Mayor's Cat: A Sunbeam, Three Council Papers and Absolutely No Comment",
        goal: goal!(1;Min(Months,1)),
        topic: 12,
    },
];
pub const HINTS: [MessageDef; 20] = [
    MessageDef {
        text: "Click a zone corner and its opposite corner to mark a rectangle.",
        goal: goal!(1; Max(Months,6)),
        topic: 0,
    },
    MessageDef {
        text: "Road and utility endpoints form an L when both coordinates differ.",
        goal: goal!(1; Max(Months,12)),
        topic: 1,
    },
    MessageDef {
        text: "Click the same tile twice to build a single tile with an area tool.",
        goal: goal!(1; Max(Months,12)),
        topic: 2,
    },
    MessageDef {
        text: "Roads carry municipal water and power conduits.",
        goal: goal!(1; Max(PowerCoverage,90)),
        topic: 3,
    },
    MessageDef {
        text: "A fresh town needs a road connection to the map edge.",
        goal: goal!(1; Max(Population,1000)),
        topic: 4,
    },
    MessageDef {
        text: "A water pump needs power and a nearby water source.",
        goal: goal!(1; Max(WaterCoverage,90)),
        topic: 5,
    },
    MessageDef {
        text: "Unserved buildings may have enough citywide capacity but no local connection.",
        goal: goal!(1; Max(PowerCoverage,99)),
        topic: 6,
    },
    MessageDef {
        text: "Pause the calendar while comparing routes and budgets.",
        goal: goal!(1; Max(Months,12)),
        topic: 7,
    },
    MessageDef {
        text: "Use the maps to distinguish a capacity shortage from a broken network.",
        goal: goal!(1; Max(WaterCoverage,95)),
        topic: 8,
    },
    MessageDef {
        text: "Averages are population-weighted where residents experience the problem.",
        goal: goal!(1; Min(Population,1000)),
        topic: 9,
    },
    MessageDef {
        text: "Keep a reserve for repairs and bond repayments.",
        goal: goal!(1; Max(Treasury,15000)),
        topic: 10,
    },
    MessageDef {
        text: "Grants and civic rewards can each be claimed only once per city.",
        goal: goal!(1; Min(Population,500)),
        topic: 11,
    },
    MessageDef {
        text: "A useful transit route links homes, jobs and services.",
        goal: goal!(1; Min(Traffic,45)),
        topic: 12,
    },
    MessageDef {
        text: "Policies have monthly costs. Winning an election earns another council slot, up to eight.",
        goal: goal!(1; Min(Population,1500)),
        topic: 13,
    },
    MessageDef {
        text: "Protect a heritage site before approving demolition around it.",
        goal: goal!(1; Min(HeritageSites,1)),
        topic: 14,
    },
    MessageDef {
        text: "Scenarios require their conditions to remain true for the stated period.",
        goal: goal!(1; Min(Months,1)),
        topic: 15,
    },
    MessageDef {
        text: "Disaster warnings leave time for a drill and an emergency reserve.",
        goal: goal!(1; Min(Months,6)),
        topic: 16,
    },
    MessageDef {
        text: "Repair damaged lots through recovery actions; their residents return over time.",
        goal: goal!(1; Event(CivicEvent::SurvivedDisaster,1)),
        topic: 17,
    },
    MessageDef {
        text: "Megaproject payments pause when the treasury cannot afford the next installment.",
        goal: goal!(1; Min(Population,25000)),
        topic: 18,
    },
    MessageDef {
        text: "January rewind restores the saved start of the current year once.",
        goal: goal!(1; Min(Months,12)),
        topic: 19,
    },
];

impl Metric {
    pub fn label(self) -> &'static str {
        match self {
            Self::Population => "Residents",
            Self::Treasury => "Treasury",
            Self::NetIncome => "Monthly balance",
            Self::Happiness => "Happiness",
            Self::Health => "Health",
            Self::Education => "Education",
            Self::Unemployment => "Unemployment",
            Self::Garbage => "Waste",
            Self::Transit => "Transit use",
            Self::Traffic => "Road traffic",
            Self::Pollution => "Residential pollution",
            Self::Crime => "Residential crime",
            Self::LandValue => "Land value",
            Self::PowerCoverage => "Power coverage",
            Self::WaterCoverage => "Water coverage",
            Self::Debt => "Loan debt",
            Self::Months => "Town age in months",
            Self::ResidentialTiles => "Housing lots",
            Self::CommercialJobs => "Commercial jobs",
            Self::IndustrialJobs => "Industrial jobs",
            Self::Parks => "Parks",
            Self::ForestTiles => "Forest tiles",
            Self::HeritageSites => "Heritage sites",
            Self::CompletedProjects => "Completed megaprojects",
            Self::Approval => "Public approval",
            Self::ElectionsWon => "Elections won",
            Self::DisastersSurvived => "Disasters survived",
            Self::RenewableShare => "Clean power share",
            Self::BondsOutstanding => "Bond principal",
            Self::SuccessfulScenarios => "Scenarios won",
            Self::OperationalAirport => "Operating airports",
            Self::OperationalPort => "Operating seaports",
            Self::ActiveWarnings => "Active disaster warnings",
            Self::UnrepairedDamage => "Unrepaired damaged lots",
            Self::ReliefAvailable => "Disaster relief available",
        }
    }
}
pub fn requirement_text(r: Requirement) -> String {
    match r {
        Requirement::Min(m, n) => format!("{} at least {n}", m.label()),
        Requirement::Max(m, n) => format!("{} at most {n}", m.label()),
        Requirement::Buildings(k, n) => format!("{n} {} facilities", k.label()),
        Requirement::Operational(k, n) => format!("{n} operating {} facilities", k.label()),
        Requirement::Event(e, n) => format!(
            "{n} recorded {}",
            match e {
                CivicEvent::BuiltTown => "town foundings",
                CivicEvent::BalancedMonth => "balanced months",
                CivicEvent::RepaidDebt => "debt repayments",
                CivicEvent::SurvivedDisaster => "disaster recoveries",
                CivicEvent::WonElection => "election wins",
                CivicEvent::SavedHeritage => "heritage designations",
                CivicEvent::CompletedProject => "project completions",
                CivicEvent::WonScenario => "scenario wins",
                CivicEvent::HeldDrill => "emergency drills",
                CivicEvent::RepairedDamage => "repaired lots",
                CivicEvent::LostElection => "election losses",
                CivicEvent::DeclaredBankruptcy => "bankruptcy filings",
            }
        ),
        Requirement::Policy(id) => format!("Enact {}", ORDINANCES[id as usize].title),
    }
}
pub fn describe_goal(goal: Goal) -> String {
    let mut s = goal
        .requirements
        .iter()
        .map(|&r| requirement_text(r))
        .collect::<Vec<_>>()
        .join("; ");
    if goal.months > 1 {
        s += &format!("; sustain {} months", goal.months);
    }
    s
}
pub fn requirement_progress(r: Requirement, f: &Facts) -> String {
    let current = match r {
        Requirement::Min(m, _) | Requirement::Max(m, _) => f.get(m),
        Requirement::Buildings(k, _) => f.count(k, false) as i64,
        Requirement::Operational(k, _) => f.count(k, true) as i64,
        Requirement::Event(e, _) => f.events[e as usize] as i64,
        Requirement::Policy(id) => i64::from(f.ordinances & (1 << id as u8) != 0),
    };
    format!("{} · currently {current}", requirement_text(r))
}
impl Unlock {
    pub fn label(self) -> &'static str {
        match self {
            Self::Services => "public service infrastructure",
            Self::Transit => "regional transit",
            Self::HigherEducation => "universities",
            Self::Seaport => "seaports",
            Self::Airport => "airports",
            Self::Nuclear => "nuclear generation",
            Self::Megaprojects => "the megaproject program",
        }
    }
}
