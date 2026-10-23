use bluecity::{
    city::{City, Kind, Terrain, Tile},
    civic::DamageRecord,
    content::OrdinanceId,
    mayor::govern,
};
fn town() -> (City, usize) {
    let mut c = City::new(17, true);
    for t in &mut c.tiles {
        if t.kind == Kind::Residential {
            t.level = 2;
            t.population = 48;
        }
    }
    let depot = c
        .tiles
        .iter()
        .position(|t| t.kind == Kind::Empty && t.terrain == Terrain::Grass && t.connected)
        .unwrap();
    c.tiles[depot].kind = Kind::MaintenanceDepot;
    // Existing roads keep this fixture's policy check independent of new
    // backbone construction and its construction spending.
    for y in 0..48 {
        let i = c.idx(24, y).unwrap();
        if c.tiles[i].kind == Kind::Empty {
            c.tiles[i].kind = Kind::Road;
        }
    }
    for x in 12..37 {
        let i = c.idx(x, 0).unwrap();
        if c.tiles[i].kind == Kind::Empty {
            c.tiles[i].kind = Kind::Road;
        }
    }
    c.money = 1_000_000;
    c.month = 24;
    c.initialize_expansion();
    c.ext.day = 720;
    c.civic.day = 720;
    c.civic.disaster_frequency = 0;
    c.refresh();
    assert!(c.population > 2500);
    (c, depot)
}
#[test]
fn a_ruined_sole_maintenance_depot_does_not_block_a_paid_replacement() {
    let (mut c, depot) = town();
    c.ext.cells[depot].rubble = true;
    c.ext.cells[depot].abandoned = true;
    let money = c.money;
    govern(&mut c, 1);
    let working: Vec<_> = c
        .tiles
        .iter()
        .enumerate()
        .filter(|(i, t)| {
            t.kind == Kind::MaintenanceDepot
                && !c.ext.cells[*i].rubble
                && !c.ext.cells[*i].abandoned
        })
        .collect();
    assert_eq!(working.len(), 1);
    assert!(
        c.ext.cells[working[0].0].construction < 100,
        "replacement uses ordinary construction"
    );
    assert!(
        money - c.money >= Kind::MaintenanceDepot.cost() + 15,
        "demolition and replacement are paid"
    );
    c.validate().unwrap();
}
#[test]
fn pending_or_temporary_outage_depots_do_not_cause_duplicates() {
    for unfinished in [false, true] {
        let (mut c, depot) = town();
        if unfinished {
            c.ext.cells[depot].construction = 25;
        } else {
            c.ext.cells[depot].outage_days = 10;
        }
        govern(&mut c, 1);
        assert_eq!(
            c.tiles
                .iter()
                .filter(|t| t.kind == Kind::MaintenanceDepot)
                .count(),
            1
        );
        assert_eq!(c.tiles[depot].kind, Kind::MaintenanceDepot);
    }
}
#[test]
fn renewal_is_limited_to_three_sites_and_charges_real_demolition_costs() {
    let (base, _) = town();
    let mut ruined = base.clone();
    let mut intact = base;
    let sites: Vec<_> = (38..43).map(|x| ruined.idx(x, 38).unwrap()).collect();
    for &i in &sites {
        ruined.tiles[i] = Tile::new(Terrain::Grass);
        intact.tiles[i] = Tile::new(Terrain::Grass);
        ruined.ext.cells[i].rubble = true;
    }
    govern(&mut ruined, 1);
    govern(&mut intact, 1);
    assert_eq!(
        sites
            .iter()
            .filter(|&&i| !ruined.ext.cells[i].rubble)
            .count(),
        3
    );
    assert_eq!(intact.money - ruined.money, 45);
}
#[test]
fn renewal_preserves_occupied_heritage_and_registered_damage_sites() {
    let (mut c, depot) = town();
    let home = c
        .tiles
        .iter()
        .rposition(|t| t.kind == Kind::Residential && t.population > 0)
        .unwrap();
    c.ext.cells[home].rubble = true;
    let park = c.tiles.iter().position(|t| t.kind == Kind::Park).unwrap();
    c.ext.cells[park].built_month = 0;
    let mut civic = std::mem::take(&mut c.civic);
    civic
        .toggle_ordinance(&mut c, OrdinanceId::Heritage)
        .unwrap();
    civic.designate_heritage(&mut c, park as u32).unwrap();
    c.civic = civic;
    c.ext.cells[park].rubble = true;
    let previous = c.tiles[depot];
    c.ext.cells[depot].rubble = true;
    c.civic.damage.push(DamageRecord {
        tile: depot as u32,
        previous,
        repair_cost: 500,
        day: 720,
    });
    govern(&mut c, 1);
    assert_eq!(c.tiles[home].kind, Kind::Residential);
    assert!(c.tiles[home].population > 0);
    assert!(c.ext.cells[home].rubble);
    assert_eq!(c.tiles[park].kind, Kind::Park);
    assert!(c.ext.cells[park].rubble);
    assert!(c.civic.protected(park as u32));
    assert_eq!(c.tiles[depot].kind, Kind::MaintenanceDepot);
    assert!(c.ext.cells[depot].rubble);
    assert!(c.civic.damage.iter().any(|d| d.tile == depot as u32));
}
#[test]
fn renewal_does_not_spend_the_cash_reserve() {
    let (mut c, _) = town();
    c.money = 2514;
    c.debt = 20000;
    let i = c.idx(40, 38).unwrap();
    c.tiles[i] = Tile::new(Terrain::Grass);
    c.ext.cells[i].rubble = true;
    govern(&mut c, 1);
    assert!(c.ext.cells[i].rubble);
}
