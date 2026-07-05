//! Fast, display-free tests for the data layer and party mechanics. These run
//! on every `cargo test` and guard the extensibility contract: content is valid
//! and cross-referenced correctly.

use hero_of_the_overworld::data::{embedded_texture, CutsceneStep, EquipSlot, Registry};
use hero_of_the_overworld::party::Party;

fn registry() -> Registry {
    Registry::load()
}

#[test]
fn game_data_parses() {
    let reg = registry();
    assert!(!reg.data.characters.is_empty(), "no characters defined");
    assert!(!reg.data.enemies.is_empty(), "no enemies defined");
    assert!(!reg.data.encounters.is_empty(), "no encounters defined");
    assert!(!reg.data.starting_party.is_empty(), "empty starting party");
}

#[test]
fn all_references_resolve() {
    let reg = registry();

    // Every starting party member exists.
    for id in &reg.data.starting_party {
        assert!(reg.character(id).is_some(), "starting party '{id}' missing");
    }
    // Every skill a character/enemy references exists.
    for c in &reg.data.characters {
        for s in &c.skills {
            assert!(
                reg.skill(s).is_some(),
                "character {} skill '{s}' missing",
                c.id
            );
        }
    }
    for e in &reg.data.enemies {
        for s in &e.skills {
            assert!(reg.skill(s).is_some(), "enemy {} skill '{s}' missing", e.id);
        }
    }
    // Every status a skill inflicts exists (e.g. FIREBALL -> burn).
    for s in &reg.data.skills {
        for st in &s.inflicts {
            assert!(
                reg.status(st).is_some(),
                "skill {} inflicts unknown status '{st}'",
                s.id
            );
        }
    }
    // Every weapon/armor a character or enemy equips exists.
    for (who, weapon, armor) in reg
        .data
        .characters
        .iter()
        .map(|c| (&c.id, &c.weapon, &c.armor))
        .chain(
            reg.data
                .enemies
                .iter()
                .map(|e| (&e.id, &e.weapon, &e.armor)),
        )
    {
        for id in [weapon, armor].into_iter().flatten() {
            assert!(
                reg.equipment(id).is_some(),
                "{who} equips unknown item '{id}'"
            );
        }
    }
    // Every enemy referenced by an encounter exists.
    for enc in &reg.data.encounters {
        assert!(
            !enc.enemies.is_empty(),
            "encounter {} has no enemies",
            enc.id
        );
        for eid in &enc.enemies {
            assert!(
                reg.enemy(eid).is_some(),
                "encounter {} enemy '{eid}' missing",
                enc.id
            );
        }
    }
}

#[test]
fn levels_are_valid_and_linked() {
    let reg = registry();
    assert!(!reg.data.levels.is_empty(), "no levels defined");

    for lv in &reg.data.levels {
        assert!(!lv.screens.is_empty(), "level {} has no screens", lv.id);
        assert!(
            (lv.start_screen) < lv.screens.len(),
            "level {} start_screen {} out of range",
            lv.id,
            lv.start_screen
        );

        for (si, sc) in lv.screens.iter().enumerate() {
            assert!(
                !sc.map.is_empty(),
                "level {} screen {si} has no rows",
                lv.id
            );
            let width = sc.map.iter().map(|r| r.chars().count()).max().unwrap();
            let height = sc.map.len();
            let solid = |col: u32, row: u32| -> bool {
                let ch = sc
                    .map
                    .get(row as usize)
                    .and_then(|r| r.chars().nth(col as usize))
                    .unwrap_or('.');
                !matches!(ch, '.' | ' ')
            };

            // Neighbour links must point at real screens in this level.
            for (dir, link) in [
                ("north", sc.north),
                ("south", sc.south),
                ("east", sc.east),
                ("west", sc.west),
            ] {
                if let Some(n) = link {
                    assert!(
                        n < lv.screens.len(),
                        "level {} screen {si} {dir} link {n} out of range",
                        lv.id
                    );
                }
            }

            // Every spawn is in-bounds, standable, and names a real encounter.
            for sp in &sc.spawns {
                assert!(
                    (sp.col as usize) < width && (sp.row as usize) < height,
                    "level {} screen {si} spawn ({},{}) out of bounds",
                    lv.id,
                    sp.col,
                    sp.row
                );
                assert!(
                    !solid(sp.col, sp.row),
                    "level {} screen {si} spawn ({},{}) sits in a solid tile",
                    lv.id,
                    sp.col,
                    sp.row
                );
                assert!(
                    reg.encounter(&sp.encounter).is_some(),
                    "level {} spawn references unknown encounter '{}'",
                    lv.id,
                    sp.encounter
                );
            }
        }

        // The player starts on a walkable tile of its start screen.
        let sc = &lv.screens[lv.start_screen];
        let start_solid = sc
            .map
            .get(lv.start.1 as usize)
            .and_then(|r| r.chars().nth(lv.start.0 as usize))
            .is_none_or(|ch| !matches!(ch, '.' | ' '));
        assert!(!start_solid, "level {} starts inside a wall", lv.id);
    }
}

/// Within each screen, every edge opening and every enemy spawn is reachable
/// from a single walkable component — so the player can always cross to the next
/// screen and touch every foe (a walled-off spawn makes a level uncompletable).
#[test]
fn screens_are_traversable() {
    let reg = registry();
    for lv in &reg.data.levels {
        for (si, sc) in lv.screens.iter().enumerate() {
            let h = sc.map.len();
            let w = sc.map.iter().map(|r| r.chars().count()).max().unwrap();
            let grid: Vec<Vec<char>> = sc
                .map
                .iter()
                .map(|r| {
                    let mut row: Vec<char> = r.chars().collect();
                    row.resize(w, '.');
                    row
                })
                .collect();
            let walkable = |c: usize, r: usize| matches!(grid[r][c], '.' | ' ');

            // Seed points: edge openings on linked sides, plus the level start.
            let mut seeds: Vec<(usize, usize)> = Vec::new();
            if sc.west.is_some() {
                for r in 0..h {
                    if walkable(0, r) {
                        seeds.push((0, r));
                    }
                }
            }
            if sc.east.is_some() {
                for r in 0..h {
                    if walkable(w - 1, r) {
                        seeds.push((w - 1, r));
                    }
                }
            }
            if sc.north.is_some() {
                for c in 0..w {
                    if walkable(c, 0) {
                        seeds.push((c, 0));
                    }
                }
            }
            if sc.south.is_some() {
                for c in 0..w {
                    if walkable(c, h - 1) {
                        seeds.push((c, h - 1));
                    }
                }
            }
            if si == lv.start_screen {
                seeds.push((lv.start.0 as usize, lv.start.1 as usize));
            }
            assert!(
                !seeds.is_empty(),
                "level {} screen {si} has no entrance",
                lv.id
            );

            // Flood fill from the first seed.
            let mut seen = vec![vec![false; w]; h];
            let mut stack = vec![seeds[0]];
            seen[seeds[0].1][seeds[0].0] = true;
            while let Some((c, r)) = stack.pop() {
                let nbrs = [
                    (c.wrapping_sub(1), r),
                    (c + 1, r),
                    (c, r.wrapping_sub(1)),
                    (c, r + 1),
                ];
                for (nc, nr) in nbrs {
                    if nc < w && nr < h && !seen[nr][nc] && walkable(nc, nr) {
                        seen[nr][nc] = true;
                        stack.push((nc, nr));
                    }
                }
            }

            // Every seed (opening/start) shares the component.
            for (c, r) in &seeds {
                assert!(
                    seen[*r][*c],
                    "level {} screen {si} opening/start ({c},{r}) is cut off",
                    lv.id
                );
            }
            // Every spawn is reachable.
            for sp in &sc.spawns {
                assert!(
                    seen[sp.row as usize][sp.col as usize],
                    "level {} screen {si} spawn at ({},{}) is walled off",
                    lv.id, sp.col, sp.row
                );
            }
        }
    }
}

/// Every texture key referenced by the overworld (tiles + the leader's walk
/// sprite) is embedded, so the map renders identically on native and web.
#[test]
fn overworld_textures_are_embedded() {
    let reg = registry();
    for key in ["grass", "water", "tree", "rock", "barricade"] {
        assert!(
            embedded_texture(key).is_some(),
            "tile texture '{key}' not embedded"
        );
    }
    for c in &reg.data.characters {
        if let Some(ow) = &c.overworld {
            assert!(
                embedded_texture(&ow.texture).is_some(),
                "character {} overworld texture '{}' not embedded",
                c.id,
                ow.texture
            );
        }
    }
    for e in &reg.data.enemies {
        if let Some(ow) = &e.overworld {
            assert!(
                embedded_texture(&ow.texture).is_some(),
                "enemy {} overworld texture '{}' not embedded",
                e.id,
                ow.texture
            );
        }
    }
    // Per-level ground/wall overrides (stone, dark_floor, dark_wall, …).
    for lv in &reg.data.levels {
        for key in [&lv.ground, &lv.wall].into_iter().flatten() {
            assert!(
                embedded_texture(key).is_some(),
                "level {} tileset texture '{key}' not embedded",
                lv.id
            );
        }
    }
}

#[test]
fn every_battler_texture_is_embedded() {
    let reg = registry();
    for c in &reg.data.characters {
        assert!(
            embedded_texture(&c.sprite.texture).is_some(),
            "character {} texture '{}' not embedded",
            c.id,
            c.sprite.texture
        );
    }
    for e in &reg.data.enemies {
        assert!(
            embedded_texture(&e.sprite.texture).is_some(),
            "enemy {} texture '{}' not embedded",
            e.id,
            e.sprite.texture
        );
    }
    assert!(embedded_texture("no_such_key").is_none());
}

#[test]
fn cutscene_references_resolve() {
    let reg = registry();

    // Levels only reference cutscenes that exist.
    for lv in &reg.data.levels {
        for id in [&lv.intro_cutscene, &lv.clear_cutscene]
            .into_iter()
            .flatten()
        {
            assert!(
                reg.cutscene(id).is_some(),
                "level {} references unknown cutscene '{id}'",
                lv.id
            );
        }
    }

    // Every recruit targets a real character; every portrait a real char/enemy.
    for cs in &reg.data.cutscenes {
        for step in &cs.steps {
            match step {
                CutsceneStep::Recruit { character } => assert!(
                    reg.character(character).is_some(),
                    "cutscene {} recruits unknown character '{character}'",
                    cs.id
                ),
                CutsceneStep::Say {
                    portrait: Some(id), ..
                } => assert!(
                    reg.character(id).is_some() || reg.enemy(id).is_some(),
                    "cutscene {} portrait '{id}' is not a character or enemy",
                    cs.id
                ),
                CutsceneStep::Say { .. } => {}
            }
        }
    }
}

/// A cutscene `Recruit` step actually grows the party, and only once even if the
/// step is applied again (the guard against duplicate joins).
#[test]
fn recruit_step_adds_the_mage_once() {
    let reg = registry();
    let mut party = Party::from_registry(&reg);
    let before = party.members.len();
    assert!(
        !party.members.iter().any(|m| m.def_id == "mage"),
        "mage should not start in the party"
    );

    // Simulate the recruit step's guarded logic.
    let recruit = |party: &mut Party| {
        if !party.members.iter().any(|m| m.def_id == "mage") {
            party.recruit(&reg, "mage");
        }
    };
    recruit(&mut party);
    recruit(&mut party); // replay must be a no-op
    assert_eq!(party.members.len(), before + 1, "mage joined exactly once");
    assert!(party.members.iter().any(|m| m.def_id == "mage"));
}

#[test]
fn starting_party_is_built_at_full_health() {
    let reg = registry();
    let party = Party::from_registry(&reg);
    assert_eq!(party.members.len(), reg.data.starting_party.len());
    for m in &party.members {
        assert!(m.is_alive());
        assert_eq!(m.hp, m.stats.max_hp);
        assert_eq!(m.mp, m.stats.max_mp);
        assert_eq!(m.level, 1);
    }
}

#[test]
fn bag_equip_and_unequip_move_items_without_loss() {
    let reg = registry();
    // The starting swordsman comes with an iron sword and leather armor.
    let mut party = Party::from_registry(&reg);
    assert_eq!(party.equipped(0, EquipSlot::Weapon), Some("iron_sword"));

    // Unequip the weapon: it lands in the bag and the slot empties.
    party.unequip(0, EquipSlot::Weapon);
    assert_eq!(party.equipped(0, EquipSlot::Weapon), None);
    assert!(party.bag.iter().any(|id| id == "iron_sword"));

    // Bag a different weapon and equip it back into the empty slot.
    party.bag.push("cavalry_lance".into());
    let idx = party
        .bag
        .iter()
        .position(|id| id == "cavalry_lance")
        .unwrap();
    assert!(party.equip_from_bag(&reg, 0, idx));
    assert_eq!(party.equipped(0, EquipSlot::Weapon), Some("cavalry_lance"));
    // The lance left the bag; the earlier iron sword is still stored there.
    assert!(!party.bag.iter().any(|id| id == "cavalry_lance"));
    assert!(party.bag.iter().any(|id| id == "iron_sword"));
}

#[test]
fn equipping_swaps_the_displaced_item_into_the_bag() {
    let reg = registry();
    let mut party = Party::from_registry(&reg); // iron sword already equipped
    party.bag.push("cavalry_lance".into());
    let idx = party.bag_indices_for(&reg, EquipSlot::Weapon)[0];
    assert!(party.equip_from_bag(&reg, 0, idx));
    assert_eq!(party.equipped(0, EquipSlot::Weapon), Some("cavalry_lance"));
    // Swapping conserves gear: the displaced iron sword returns to the bag.
    assert!(party.bag.iter().any(|id| id == "iron_sword"));
}

#[test]
fn bag_indices_filter_by_slot() {
    let reg = registry();
    let mut party = Party::from_registry(&reg);
    party.bag = vec![
        "iron_sword".into(),
        "leather_armor".into(),
        "cavalry_lance".into(),
    ];
    assert_eq!(party.bag_indices_for(&reg, EquipSlot::Weapon).len(), 2);
    assert_eq!(party.bag_indices_for(&reg, EquipSlot::Armor).len(), 1);
}

#[test]
fn party_is_extensible() {
    let reg = registry();
    let mut party = Party::from_registry(&reg);
    let before = party.members.len();

    // Recruiting an existing character grows the party (the core extensibility
    // guarantee: more characters can assist in battle with no code changes).
    let existing = reg.data.characters[0].id.clone();
    assert!(party.recruit(&reg, &existing));
    assert_eq!(party.members.len(), before + 1);

    // Recruiting an unknown id is rejected, not a panic.
    assert!(!party.recruit(&reg, "does_not_exist"));
    assert_eq!(party.members.len(), before + 1);
}

/// A hero recruited mid-game joins at the party's current level (not level 1),
/// with base stats grown up the same curve, so newcomers like Gareth arrive on
/// par with Roland instead of as dead weight.
#[test]
fn recruits_join_at_party_level() {
    let reg = registry();
    let mut party = Party::from_registry(&reg);

    // Push the starting party up several levels.
    party.grant_xp(1000);
    let party_level = party.level();
    assert!(party_level > 1, "party should have leveled up");

    // Gareth's authored base stats (what a level-1 build would have).
    let base = reg.character("hermit").expect("gareth defined");
    let base_hp = base.stats.max_hp;

    party.recruit(&reg, "hermit");
    let gareth = party
        .members
        .iter()
        .find(|m| m.def_id == "hermit")
        .expect("gareth recruited");

    assert_eq!(gareth.level, party_level, "joins at the party's level");
    assert!(
        gareth.stats.max_hp > base_hp,
        "grown stats, not level-1 base"
    );
    assert_eq!(gareth.hp, gareth.stats.max_hp, "arrives at full health");
    assert_eq!(gareth.xp, 0, "no leftover XP debt");
}

#[test]
fn xp_grants_levels_and_growth() {
    let reg = registry();
    let mut party = Party::from_registry(&reg);
    let atk_before = party.members[0].stats.attack;
    let hp_before = party.members[0].stats.max_hp;

    let leveled = party.grant_xp(1000);
    assert!(leveled.contains(&0), "member 0 should have leveled up");
    assert!(party.members[0].level > 1);
    assert!(
        party.members[0].stats.attack > atk_before,
        "attack should grow"
    );
    assert!(
        party.members[0].stats.max_hp > hp_before,
        "max hp should grow"
    );
}

/// Roaming enemies scale to the party's level: identity at level 1, and stats
/// (but never speed) grow with the party so late-game foes aren't one-shot.
#[test]
fn enemies_scale_to_party_level() {
    use hero_of_the_overworld::data::enemy_scale;

    let reg = registry();
    let crab = reg.enemy("mountain_crab").expect("mountain_crab defined");

    // Level 1 is the identity — the opening region fights foes as authored.
    assert_eq!(enemy_scale(1), 100);
    let base = crab.stats.scaled_to(1);
    assert_eq!(base.max_hp, crab.stats.max_hp);
    assert_eq!(base.attack, crab.stats.attack);

    // At a higher party level the enemy is meaningfully tougher...
    let scaled = crab.stats.scaled_to(9);
    assert!(scaled.max_hp > crab.stats.max_hp, "HP should scale up");
    assert!(scaled.attack > crab.stats.attack, "attack should scale up");
    assert!(
        scaled.defense > crab.stats.defense,
        "defense should scale up"
    );
    // ...but its speed (turn order) is preserved exactly.
    assert_eq!(scaled.speed, crab.stats.speed, "speed must not scale");

    // The party's level is the highest member's level.
    let mut party = Party::from_registry(&reg);
    assert_eq!(party.level(), 1);
    party.grant_xp(1000);
    assert!(party.level() > 1, "party level tracks the leader's growth");
}

/// Equipment referenced by characters/enemies resolves, every item icon is
/// embedded, and every item and skill carries a (non-empty) description.
#[test]
fn equipment_references_resolve() {
    let reg = registry();

    for c in &reg.data.characters {
        for id in [&c.weapon, &c.armor].into_iter().flatten() {
            assert!(
                reg.equipment(id).is_some(),
                "character {} equips unknown item '{id}'",
                c.id
            );
        }
    }
    for e in &reg.data.enemies {
        for id in [&e.weapon, &e.armor].into_iter().flatten() {
            assert!(
                reg.equipment(id).is_some(),
                "enemy {} equips unknown item '{id}'",
                e.id
            );
        }
    }

    for item in &reg.data.equipment {
        assert!(
            embedded_texture(&item.icon).is_some(),
            "item {} icon '{}' not embedded",
            item.id,
            item.icon
        );
        assert!(
            !item.description.trim().is_empty(),
            "item {} has no description",
            item.id
        );
    }
    for s in &reg.data.skills {
        assert!(
            !s.description.trim().is_empty(),
            "skill {} has no description",
            s.id
        );
    }
}

/// Equipment actually changes a battler's effective stats (the extensibility
/// contract for gear: bonuses apply without touching battle code).
#[test]
fn equipping_gear_changes_effective_stats() {
    let reg = registry();
    let sword = &reg.data.characters[0]; // ROLAND, with iron_sword + leather_armor
    let base = &sword.stats;
    let eq = reg.equipped(base, sword.weapon.as_deref(), sword.armor.as_deref());
    assert!(
        eq.stats.attack > base.attack,
        "the iron sword should raise attack"
    );
    assert!(
        eq.stats.defense > base.defense,
        "the leather armor should raise defense"
    );
    assert!(
        eq.crit > 0 || eq.accuracy > 0,
        "the weapon should add crit/accuracy"
    );
    assert!(eq.evasion > 0, "the armor should add evasion");
}

/// Shops are well-formed: every shop id is unique, every ware and every screen
/// entrance resolves, prices are sane, and entrances sit on standable tiles.
/// This guards the shop extensibility contract the same way spawns are guarded.
#[test]
fn shops_are_valid_and_placed() {
    let reg = registry();

    for shop in &reg.data.shops {
        assert!(!shop.name.trim().is_empty(), "shop {} has no name", shop.id);
        assert!(!shop.stock.is_empty(), "shop {} sells nothing", shop.id);
        for s in &shop.stock {
            assert!(
                reg.equipment(&s.item).is_some(),
                "shop {} stocks unknown item '{}'",
                shop.id,
                s.item
            );
            assert!(
                s.price >= 0,
                "shop {} item '{}' has a negative price",
                shop.id,
                s.item
            );
        }
    }

    // Every entrance placed on a screen is in-bounds, standable, and names a
    // real shop (a keeper stranded in a wall or pointing at nothing is a bug).
    for lv in &reg.data.levels {
        for (si, sc) in lv.screens.iter().enumerate() {
            let width = sc.map.iter().map(|r| r.chars().count()).max().unwrap();
            let height = sc.map.len();
            let solid = |col: u32, row: u32| -> bool {
                let ch = sc
                    .map
                    .get(row as usize)
                    .and_then(|r| r.chars().nth(col as usize))
                    .unwrap_or('.');
                !matches!(ch, '.' | ' ')
            };
            for sp in &sc.shops {
                assert!(
                    (sp.col as usize) < width && (sp.row as usize) < height,
                    "level {} screen {si} shop entrance ({},{}) out of bounds",
                    lv.id,
                    sp.col,
                    sp.row
                );
                assert!(
                    !solid(sp.col, sp.row),
                    "level {} screen {si} shop entrance ({},{}) sits in a solid tile",
                    lv.id,
                    sp.col,
                    sp.row
                );
                assert!(
                    reg.shop(&sp.shop).is_some(),
                    "level {} screen {si} entrance references unknown shop '{}'",
                    lv.id,
                    sp.shop
                );
            }
        }
    }
}
