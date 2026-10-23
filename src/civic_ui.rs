//! Pure rows and their commands: civic window text and hit targets share ordering.
use crate::{
    civic, content,
    ui_model::{Command, Details, Window},
    State,
};
pub struct CivicRow {
    pub title: String,
    pub body: String,
    pub action: Option<(Command, &'static str)>,
}
fn row(
    title: impl Into<String>,
    body: impl Into<String>,
    action: Option<(Command, &'static str)>,
) -> CivicRow {
    CivicRow {
        title: title.into(),
        body: body.into(),
        action,
    }
}
fn cash(n: i64) -> String {
    format!("${n}")
}
pub fn records(st: &State, w: Window) -> Option<Vec<CivicRow>> {
    let c = &st.city;
    let g = &c.civic;
    let mut out = Vec::new();
    match w {
        Window::Ordinances => {
            for d in content::ORDINANCES {
                out.push(row(
                    format!(
                        "{} · {} · ${}/month",
                        if g.active(d.id) { "ON" } else { "OFF" },
                        d.title,
                        civic::ordinance_monthly_cost(c, d.id)
                    ),
                    d.description,
                    Some((
                        Command::Ordinance(d.id as usize),
                        if g.active(d.id) {
                            "Repeal"
                        } else {
                            "Enact $100"
                        },
                    )),
                ));
            }
        }
        Window::Scenarios => {
            if let Some(s) = &g.scenario {
                let d = content::SCENARIOS[s.id as usize];
                out.push(row(
                    format!("{} · {:?}", d.title, s.result),
                    format!(
                        "Month {} of {} · qualifying streak {} / {}",
                        c.month.saturating_sub(s.started_month),
                        s.deadline_month - s.started_month,
                        s.streak,
                        d.goal.months
                    ),
                    Some((Command::Details(Details::Scenario(s.id as usize)), "Goals")),
                ));
            }
            for (i, d) in content::SCENARIOS.iter().enumerate() {
                out.push(row(
                    format!(
                        "{} · {} months · prize ${}",
                        d.title, d.deadline_months, d.prize
                    ),
                    d.briefing,
                    Some((Command::Scenario(i), "Start")),
                ));
                out.push(row(
                    "Objectives",
                    content::describe_goal(d.goal),
                    Some((Command::Details(Details::Scenario(i)), "Details")),
                ));
            }
        }
        Window::Projects => {
            for (i, d) in content::MEGAPROJECTS.iter().enumerate() {
                let run = g.projects.iter().find(|p| p.id as usize == i);
                let (body, action) = if let Some(p) = run {
                    (
                        format!(
                            "{} / {} months · paid {} / {} · {}",
                            p.months_done,
                            d.months,
                            cash(p.paid),
                            cash(d.cost),
                            if p.complete {
                                "Complete"
                            } else if p.paused {
                                "Paused: check funds, site and utilities"
                            } else {
                                "Building"
                            }
                        ),
                        if p.complete {
                            None
                        } else {
                            Some((Command::CancelProject(i as u8), "Cancel"))
                        },
                    )
                } else {
                    (
                        format!(
                            "{} · {} months · deposit {}",
                            d.description,
                            d.months,
                            cash(d.cost / 4)
                        ),
                        Some((Command::Project(i), "Commission")),
                    )
                };
                out.push(row(format!("{} · {}", d.title, cash(d.cost)), body, action));
                out.push(row(
                    "Requirements",
                    content::describe_goal(d.goal),
                    Some((Command::Details(Details::Project(i)), "Details")),
                ));
            }
        }
        Window::Trade => {
            out.push(row("Regional links",if c.outside_connected(){"Outside road access is connected. Purchases still require a live utility or freight route and sufficient funds."}else{"No outside connection: connect a road to the region boundary before importing power or trading goods."},None));
            for (i, n) in c.ext.neighbors.iter().enumerate() {
                out.push(row(format!("{} · {}",n.region,if n.enabled{"Agreement enabled"}else{"Agreement disabled"}),format!("Power price ${} / 100 units · freight tariff ${} / 100 · last settlement ${}",n.unit_price,n.tariff,n.monthly_cost),Some((Command::NeighborToggle(i),if n.enabled{"Disable"}else{"Enable"}))));
                out.push(row(
                    format!("Power request: {} units", n.power),
                    format!(
                        "Purchased last cycle: {} · raise request by 5,000",
                        n.last_power
                    ),
                    Some((Command::NeighborPower(i, 5000), "Power +")),
                ));
                out.push(row(
                    "Reduce power request",
                    "Lower by 5,000 units; requests are capped at 100,000",
                    Some((Command::NeighborPower(i, -5000), "Power −")),
                ));
                out.push(row(
                    format!("Goods quota: {} units", n.goods),
                    format!(
                        "Last net goods: {} (imports positive, exports negative)",
                        n.last_goods
                    ),
                    Some((Command::NeighborGoods(i, 1000), "Goods +")),
                ));
                out.push(row(
                    "Reduce goods quota",
                    "Lower by 1,000; quota is capped at 50,000 goods",
                    Some((Command::NeighborGoods(i, -1000), "Goods −")),
                ));
                out.push(row(
                    format!("Freight tariff: ${} / 100 units", n.tariff),
                    "A higher tariff increases both imported-goods costs and export receipts",
                    Some((Command::NeighborTariff(i, 1), "Tariff +")),
                ));
                out.push(row(
                    "Lower freight tariff",
                    "Tariffs range from $0 to $25 per hundred units",
                    Some((Command::NeighborTariff(i, -1), "Tariff −")),
                ));
            }
        }
        Window::Budget => {
            out.push(row(
                format!(
                    "Treasury {} · monthly balance {}",
                    cash(c.money),
                    cash(c.income - c.expenses)
                ),
                format!(
                    "Loan debt {} · {} of 3 bond slots · credit restrictions until month {}",
                    cash(c.debt),
                    g.bonds.len(),
                    g.bankruptcy_until
                ),
                None,
            ));
            out.push(row(
                format!("Overdraft limit: -{}", cash(crate::civic::overdraft_limit(c))),
                "Crossing the settled limit triggers court protection, debt discharge and twelve months of austerity",
                None,
            ));
            for amount in [10000, 25000, 50000] {
                out.push(row(
                    format!("Issue {} bond", cash(amount)),
                    format!(
                        "60 months · {} issue fee · {} monthly interest",
                        cash(amount / 100),
                        cash(amount / 200)
                    ),
                    Some((Command::BondAmount(amount), "Issue")),
                ));
            }
            for (i, b) in g.bonds.iter().enumerate() {
                out.push(row(
                    format!("Bond {} · principal {}", i + 1, cash(b.principal)),
                    format!(
                        "{} months left · {} monthly interest",
                        b.months_left,
                        cash(b.monthly_interest)
                    ),
                    Some((Command::RepayBond(i), "Repay")),
                ));
            }
            out.push(row(
                "Repay municipal loan",
                format!(
                    "Repay up to $10,000 of the {} outstanding loan",
                    cash(c.debt)
                ),
                Some((Command::RepayLoan, "Repay")),
            ));
            out.push(row(
                "Emergency regional bailout",
                "$25,000 aid; $20,000 bond, tax conditions and six months of restricted credit",
                Some((Command::Bailout, "Bailout")),
            ));
            out.push(row("Restructure debt","Creditors reduce principal 20%; a 120-month bond and austerity replace current debt",Some((Command::Restructure,"Restructure"))));
            out.push(row("Bankruptcy protection","Discharge debt; reset treasury to $5,000; lose trust and accept twelve months of austerity",Some((Command::Bankruptcy,"Discharge"))));
            out.push(row(
                "Rewind to January",
                if g.rewind_used {
                    "This year's rewind has already been used."
                } else {
                    "Restore this year's January city and government checkpoint once."
                },
                Some((Command::Rewind, "Rewind")),
            ));
        }
        Window::Disasters => {
            for request in &c.ext.services.repair_requests {
                out.push(row(
                    "Repair crew queued",
                    format!(
                        "Lot {},{} · ${} due on arrival",
                        request.tile % u32::from(c.width),
                        request.tile / u32::from(c.width),
                        request.cost
                    ),
                    Some((Command::CancelRepair(request.tile), "Cancel")),
                ));
            }

            out.push(row(
                format!(
                    "Frequency: {}",
                    ["Off", "Low", "Normal", "High"][g.disaster_frequency.min(3) as usize]
                ),
                format!(
                    "{} warnings · {} damaged lots · drill protection until day {}",
                    g.warnings.len(),
                    g.damage.len(),
                    g.drill_until
                ),
                None,
            ));
            out.push(row(
                "Emergency drill",
                format!(
                    "Cost ${}; 180-day cooldown, 360 days of protection",
                    1000 + c.population / 10
                ),
                Some((Command::Drill, "Run drill")),
            ));
            out.push(row(
                "Regional disaster relief",
                "Claim verified recovery aid once within thirty days of a damaging disaster",
                Some((Command::Relief, "Claim aid")),
            ));
            if let Some([x, y]) = st.selected {
                if let Some(i) = c.idx(x, y) {
                    if let Some(d) = g.damage.iter().find(|d| d.tile as usize == i) {
                        out.push(row(
                            "Repair selected lot",
                            format!(
                                "Restore {} for {}",
                                d.previous.kind.label(),
                                cash(d.repair_cost)
                            ),
                            Some((Command::Repair(i as u32), "Repair")),
                        ));
                    }
                }
            }
            out.push(row(
                "Disaster frequency −",
                "Reduce automatic disaster frequency; manual sandbox events remain available",
                Some((Command::Frequency(-1), "Reduce")),
            ));
            out.push(row(
                "Disaster frequency +",
                "Increase automatic disaster frequency",
                Some((Command::Frequency(1), "Increase")),
            ));
            for (i, k) in civic::Hazard::ALL.into_iter().enumerate() {
                out.push(row(k.label(),"Schedule a real event with a three-day warning. Save first if you want to experiment.",Some((Command::Disaster(i),"Schedule"))));
            }
            for warning in &g.warnings {
                out.push(row(
                    format!("Warning: {}", warning.kind.label()),
                    format!(
                        "Impact in {} days",
                        warning.impact_day.saturating_sub(g.day)
                    ),
                    None,
                ));
            }
            for d in &g.damage {
                out.push(row(
                    format!(
                        "Damaged {} · lot {},{}",
                        d.previous.kind.label(),
                        d.tile % u32::from(c.width),
                        d.tile / u32::from(c.width)
                    ),
                    format!(
                        "Repair {} · residents and jobs return gradually",
                        cash(d.repair_cost)
                    ),
                    Some((Command::Repair(d.tile), "Repair")),
                ));
            }
        }
        Window::Districts => {
            out.push(row(
                format!("{} / 16 districts", g.districts.len()),
                "Create a ward around the selected tile; its first policy is Balanced",
                Some((Command::District, "Create")),
            ));
            for d in &g.districts {
                out.push(row(
                    format!(
                        "{} · {}",
                        d.name,
                        civic::DISTRICT_SPECIALIZATIONS[d.specialization.min(4) as usize]
                    ),
                    format!(
                        "{} residents · {} jobs · {}% local livability · {} lots",
                        d.population,
                        d.jobs,
                        d.happiness,
                        d.tiles.len()
                    ),
                    Some((Command::DistrictPolicy(d.id), "Policy +")),
                ));
                out.push(row(
                    "District name",
                    format!("Cycle an authored name for {}", d.name),
                    Some((Command::RenameDistrict(d.id), "Rename")),
                ));
                out.push(row(
                    "Remove district",
                    format!(
                        "Remove {}'s boundary and local policy; buildings remain",
                        d.name
                    ),
                    Some((Command::RemoveDistrict(d.id), "Remove")),
                ));
            }
            out.push(row(
                "Heritage register",
                "Protect an occupied building at least one year old. Registration costs $500.",
                Some((Command::Heritage, "Register")),
            ));
            for h in &g.heritage {
                out.push(row(
                    h.name.clone(),
                    format!(
                        "Lot {},{} · $25/month · ordinance controls demolition protection",
                        h.tile % u32::from(c.width),
                        h.tile / u32::from(c.width)
                    ),
                    Some((Command::RemoveHeritage(h.tile), "Revoke")),
                ));
            }
        }
        Window::Notebook => {
            if let Some(i) = g.next_milestone() {
                let d = content::MILESTONES[i];
                out.push(row(
                    format!("Next: {}", d.title),
                    content::describe_goal(d.goal),
                    Some((Command::Details(Details::Milestone(i)), "Details")),
                ));
            }
            for (i, n) in g.notebook.iter().enumerate().rev() {
                out.push(row(
                    format!("Month {} · {}", n.month, n.title),
                    n.text.clone(),
                    Some((Command::Details(Details::Note(i)), "Read")),
                ));
            }
            for (i, d) in content::MILESTONES.iter().enumerate() {
                out.push(row(
                    format!(
                        "{} {}",
                        if g.milestone_bits & (1 << i) != 0 {
                            "Complete:"
                        } else {
                            "Milestone:"
                        },
                        d.title
                    ),
                    content::describe_goal(d.goal),
                    Some((Command::Details(Details::Milestone(i)), "Details")),
                ));
            }
            for (i, d) in content::REQUESTS.iter().enumerate() {
                out.push(row(
                    format!(
                        "{} {}",
                        if g.request_bits & (1 << i) != 0 {
                            "Complete:"
                        } else {
                            "Request:"
                        },
                        d.title
                    ),
                    content::describe_goal(d.goal),
                    Some((Command::Details(Details::Request(i)), "Details")),
                ));
            }
            for (i, d) in content::ACHIEVEMENTS.iter().enumerate() {
                out.push(row(
                    format!(
                        "{} {}",
                        if g.achievement_bits & (1 << i) != 0 {
                            "Earned:"
                        } else {
                            "Achievement:"
                        },
                        d.title
                    ),
                    content::describe_goal(d.goal),
                    Some((Command::Details(Details::Achievement(i)), "Details")),
                ));
            }
        }
        Window::Evaluation => {
            let e = civic::evaluation(c);
            out.push(row(
                format!("Score {} / 1000 · grade {}", e.score, e.grade),
                format!(
                    "Approval {}% · population {} · happiness {}%",
                    g.approval, c.population, c.happiness
                ),
                None,
            ));
            out.push(row(
                "Public services",
                format!(
                    "Health {} · education {} · unemployment {}% · waste {}",
                    c.health, c.education, c.unemployment, c.garbage
                ),
                None,
            ));
            out.push(row(
                "Civic progress",
                format!(
                    "{} / 25 milestones · {} / 12 requests · {} / 8 achievements",
                    g.milestone_bits.count_ones(),
                    g.request_bits.count_ones(),
                    g.achievement_bits.count_ones()
                ),
                None,
            ));
            let people = &c.ext.people;
            out.push(row(
                "People and ages",
                format!(
                    "Children {} · adults {} · seniors {} · households {}",
                    people.children,
                    people.adults,
                    people.seniors,
                    people.households.len()
                ),
                None,
            ));
            out.push(row(
                "Births, deaths and migration",
                format!(
                    "Born {} · died {} · moved in {} · moved out {}",
                    people.births, people.deaths, people.migrants_in, people.migrants_out
                ),
                None,
            ));
            out.push(row(
                "Workforce",
                format!(
                    "{} available · {} employed · {} overqualified",
                    people.workforce, people.employed, people.overqualified
                ),
                None,
            ));
            for (i, label) in ["Basic", "Secondary", "College", "University"]
                .into_iter()
                .enumerate()
            {
                out.push(row(
                    format!("{} education", label),
                    format!(
                        "Residents {} · employed residents {} · matching jobs filled {} / {}",
                        people.education_levels[i],
                        people.employed_by_education[i],
                        people.filled_jobs_by_education[i],
                        people.jobs_by_education[i]
                    ),
                    None,
                ));
            }
            for (i, label) in [
                "Health",
                "Education",
                "Employment",
                "Air quality",
                "Safety",
                "Traffic",
                "Land value",
                "Waste",
                "Electricity",
                "Water",
                "Outside access",
                "Taxes",
                "Goods",
                "Transit",
                "Services",
                "Community",
            ]
            .into_iter()
            .enumerate()
            {
                out.push(row(
                    format!("Happiness factor: {}", label),
                    format!(
                        "{} / 100; calculated from the actual household and city conditions",
                        people.happiness_factors[i]
                    ),
                    None,
                ));
            }
            for medal in civic::Medal::ALL {
                out.push(row(
                    format!(
                        "{} {}",
                        if e.medals.contains(&medal) {
                            "Earned:"
                        } else {
                            "Medal:"
                        },
                        medal.label()
                    ),
                    medal.description(),
                    None,
                ));
            }
            for vote in g.elections.iter().rev() {
                out.push(row(
                    format!(
                        "Election, month {} · {}",
                        vote.month,
                        if vote.won { "Won" } else { "Lost" }
                    ),
                    format!("{}% support; every term is recorded", vote.votes),
                    None,
                ));
            }
        }
        Window::Chirper => {
            for (i, post) in g.chirper.iter().rev().enumerate() {
                out.push(row(
                    format!(
                        "Citizen {} · month {}",
                        post.citizen
                            .map(|n| n.to_string())
                            .unwrap_or_else(|| "Citywide".into()),
                        post.month
                    ),
                    content::CHIRPER[post.id as usize].text,
                    Some((Command::Meet(i), "Meet")),
                ));
            }
        }
        Window::Advisors => {
            for (i, post) in g.advisors.iter().rev().enumerate() {
                let d = content::ADVISORS[post.id as usize];
                out.push(row(
                    d.name,
                    d.text,
                    Some((Command::Details(Details::Advisor(i)), "Read")),
                ));
            }
            if out.is_empty() {
                out.push(row("Advisors are reviewing the city","Advice reacts to actual utility, transport, finance, safety and neighborhood conditions.",None));
            }
        }
        Window::Newspaper => {
            for (i, post) in g.headlines.iter().rev().enumerate() {
                out.push(row(
                    format!("BlueCity Post · month {}", post.month),
                    content::HEADLINES[post.id as usize].text,
                    Some((Command::Details(Details::Headline(i)), "Read")),
                ));
            }
            out.push(row(
                "The Mayor's Cat",
                content::HEADLINES[60].text,
                Some((Command::Details(Details::Cat), "Read")),
            ));
        }
        _ => return None,
    }
    Some(out)
}
pub fn rows(st: &State, w: Window) -> Option<Vec<(String, String)>> {
    records(st, w).map(|r| r.into_iter().map(|r| (r.title, r.body)).collect())
}
pub fn details(st: &State, d: Details) -> Result<(String, String), String> {
    let g = &st.city.civic;
    let f = g.current_facts(&st.city);
    let goal = |title: &str, brief: &str, goal: content::Goal| {
        (
            title.to_owned(),
            format!(
                "{}\n{}\nSustain {} month(s).",
                brief,
                goal.requirements
                    .iter()
                    .map(|&r| content::requirement_progress(r, &f))
                    .collect::<Vec<_>>()
                    .join("\n"),
                goal.months
            ),
        )
    };
    Ok(match d {
        Details::Milestone(i) => {
            let d = content::MILESTONES.get(i).ok_or("Unknown milestone.")?;
            goal(
                d.title,
                &format!("Grant ${}. {}",d.reward.cash,d.reward.unlock.map(|u|format!("Unlocks {}.",u.label())).unwrap_or_else(||"A permanent population milestone for this city.".into())),
                d.goal,
            )
        }
        Details::Request(i) => {
            let d = content::REQUESTS.get(i).ok_or("Unknown request.")?;
            goal(d.title, d.description, d.goal)
        }
        Details::Achievement(i) => {
            let d = content::ACHIEVEMENTS.get(i).ok_or("Unknown achievement.")?;
            goal(d.title, d.description, d.goal)
        }
        Details::Scenario(i) => {
            let d = content::SCENARIOS.get(i).ok_or("Unknown scenario.")?;
            goal(d.title, d.briefing, d.goal)
        }
        Details::Project(i) => {
            let d = content::MEGAPROJECTS.get(i).ok_or("Unknown project.")?;
            goal(d.title, d.description, d.goal)
        }
        Details::Note(i) => {
            let n = g
                .notebook
                .get(i)
                .ok_or("That note is no longer available.")?;
            (n.title.clone(), n.text.clone())
        }
        Details::Advisor(i) => {
            let n = g
                .advisors
                .iter()
                .rev()
                .nth(i)
                .ok_or("That advice is no longer available.")?;
            let d = content::ADVISORS[n.id as usize];
            (d.name.into(), d.text.into())
        }
        Details::Headline(i) => {
            let n = g
                .headlines
                .iter()
                .rev()
                .nth(i)
                .ok_or("That edition is no longer available.")?;
            (
                "BlueCity Post".into(),
                content::HEADLINES[n.id as usize].text.into(),
            )
        }
        Details::Cat => ("The Mayor's Cat".into(), content::HEADLINES[60].text.into()),
    })
}
