//! Fast, display-free tests for the data layer and party mechanics. These run
//! on every `cargo test` and guard the extensibility contract: content is valid
//! and cross-referenced correctly.

use hero_of_the_overworld::data::{
    embedded_texture, AttackAnim, CutsceneStep, EquipSlot, Registry, TargetKind,
};
use hero_of_the_overworld::party::{FieldUse, Party, ACTIVE_PARTY};

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
        // Every level-unlocked skill must resolve, and be learned past level 1
        // (a level-1 unlock belongs in `skills`, not the learnset).
        for ls in &c.learnset {
            assert!(
                reg.skill(&ls.skill).is_some(),
                "character {} learnset skill '{}' missing",
                c.id,
                ls.skill
            );
            assert!(
                ls.level > 1,
                "character {} learns '{}' at level {} — level-1 skills go in `skills`",
                c.id,
                ls.skill,
                ls.level
            );
        }
    }
    for e in &reg.data.enemies {
        for s in &e.skills {
            assert!(reg.skill(s).is_some(), "enemy {} skill '{s}' missing", e.id);
        }
    }
    // Every status a skill inflicts exists (e.g. FIREBALL -> burn), and any
    // projectile attack animation names an embedded texture.
    for s in &reg.data.skills {
        for st in &s.inflicts {
            assert!(
                reg.status(st).is_some(),
                "skill {} inflicts unknown status '{st}'",
                s.id
            );
        }
        // A projectile, boomerang, or crowd animation names an embedded texture
        // (the bolt/axe art, or the swarm's walk sheet).
        let anim_tex = match &s.anim {
            AttackAnim::Projectile { texture }
            | AttackAnim::Boomerang { texture }
            | AttackAnim::Crowd { texture } => Some(texture),
            _ => None,
        };
        if let Some(texture) = anim_tex {
            assert!(
                embedded_texture(texture).is_some(),
                "skill {} animation texture '{texture}' not embedded",
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

/// Tool enemies (ballistae and future siege engines) are well-formed: the skill
/// they fire resolves, their operate chance is a valid probability, and every
/// encounter that fields one also fields at least one *aware* (non-tool) foe to
/// work it — a tool with no crew crumbles instantly, so a lone-tool encounter is
/// a content bug. Guards the tool-enemy extensibility contract.
#[test]
fn tool_enemy_references_resolve() {
    let reg = registry();

    let is_tool = |eid: &str| reg.enemy(eid).is_some_and(|e| e.tool.is_some());

    for e in &reg.data.enemies {
        let Some(tool) = &e.tool else { continue };
        assert!(
            reg.skill(&tool.skill).is_some(),
            "tool enemy {} fires unknown skill '{}'",
            e.id,
            tool.skill
        );
        assert!(
            (0.0..=1.0).contains(&tool.operate_chance),
            "tool enemy {} has an out-of-range operate_chance {}",
            e.id,
            tool.operate_chance
        );
    }

    for enc in &reg.data.encounters {
        if enc.enemies.iter().any(|eid| is_tool(eid)) {
            assert!(
                enc.enemies.iter().any(|eid| !is_tool(eid)),
                "encounter {} fields a tool with no aware crew to work it",
                enc.id
            );
        }
    }
}

/// Mimic enemies are well-formed: every skill on a mimic's copy allow-list
/// resolves, the power nerf is a positive percentage, and the per-turn chance is a
/// valid probability. Guards the mimicry extensibility contract (a future variant
/// with a wider list or different nerf stays validated).
#[test]
fn mimic_references_resolve() {
    let reg = registry();

    for e in &reg.data.enemies {
        let Some(mim) = &e.mimicry else { continue };
        assert!(
            !mim.copyable.is_empty(),
            "mimic {} has an empty copy allow-list",
            e.id
        );
        for id in &mim.copyable {
            assert!(
                reg.skill(id).is_some(),
                "mimic {} can copy unknown skill '{id}'",
                e.id
            );
        }
        assert!(
            mim.power_pct > 0,
            "mimic {} has a non-positive mimicry power_pct {}",
            e.id,
            mim.power_pct
        );
        assert!(
            (0.0..=1.0).contains(&mim.chance),
            "mimic {} has an out-of-range mimicry chance {}",
            e.id,
            mim.chance
        );
    }
}

/// The clone enemies are faithful stat-mirrors of the party: each `clone_*` foe
/// carries the exact stats and gear of the hero it doubles, so a mirror match is a
/// true reflection. Guards against the party defs and their clones drifting apart.
#[test]
fn clones_mirror_the_party() {
    let reg = registry();

    for (clone_id, hero_id) in [
        ("clone_roland", "swordsman"),
        ("clone_elara", "mage"),
        ("clone_gareth", "hermit"),
        ("clone_brenn", "axeman"),
        ("clone_captain", "captain"),
    ] {
        let clone = reg
            .enemy(clone_id)
            .unwrap_or_else(|| panic!("missing clone enemy '{clone_id}'"));
        let hero = reg
            .character(hero_id)
            .unwrap_or_else(|| panic!("missing hero '{hero_id}'"));

        let (c, h) = (&clone.stats, &hero.stats);
        assert_eq!(
            (c.max_hp, c.max_mp, c.attack, c.defense, c.magic, c.speed),
            (h.max_hp, h.max_mp, h.attack, h.defense, h.magic, h.speed),
            "{clone_id} stats must mirror {hero_id}"
        );
        assert_eq!(
            clone.weapon, hero.weapon,
            "{clone_id} weapon must mirror {hero_id}"
        );
        assert_eq!(
            clone.armor, hero.armor,
            "{clone_id} armor must mirror {hero_id}"
        );
        // A clone should share the hero's sprite sheet (a dark reflection of them).
        assert_eq!(
            clone.sprite.texture, hero.sprite.texture,
            "{clone_id} should wear {hero_id}'s sprite"
        );
        // And it must point back at the hero it doubles, so the mirror match only
        // fields it when that hero is in the active line-up.
        assert_eq!(
            clone.mirrors.as_deref(),
            Some(hero_id),
            "{clone_id} must mirror {hero_id}"
        );
    }

    // The mirror-match boss lists a clone for every recruitable hero (one per
    // character def), and is a boss fight. Which of these actually take the field
    // is filtered to the active line-up at battle start — see
    // `mirror_match_fields_only_active_members`.
    let enc = reg
        .encounter("mirror_match")
        .expect("missing mirror_match encounter");
    assert!(enc.boss, "mirror_match should play the boss theme");
    assert_eq!(
        enc.enemies,
        vec![
            "clone_roland",
            "clone_elara",
            "clone_gareth",
            "clone_brenn",
            "clone_captain"
        ],
        "mirror_match should list one clone of each recruitable hero"
    );
    // A clone for every character def, so any active line-up meets its mirror.
    assert_eq!(
        enc.enemies.len(),
        reg.data.characters.len(),
        "mirror_match should carry exactly one clone per recruitable character"
    );
}

/// The mirror match is **conditional on the active line-up**: each clone only
/// takes the field when the hero it doubles is active, so bringing a different
/// party reflects a different set of shadow-doubles. Ordinary encounters, whose
/// foes carry no `mirrors` target, are unaffected.
#[test]
fn mirror_match_fields_only_active_members() {
    use hero_of_the_overworld::data::active_encounter_enemies;
    use std::collections::HashSet;

    let reg = registry();
    let enc = reg.encounter("mirror_match").expect("missing mirror_match");

    let fielded = |active: &HashSet<&str>| -> Vec<String> {
        active_encounter_enemies(&reg, &enc.enemies, active)
            .iter()
            .map(|s| s.to_string())
            .collect()
    };

    // The opening trio faces its three reflections, in the encounter's order.
    let opening: HashSet<&str> = ["swordsman", "mage", "hermit"].into_iter().collect();
    assert_eq!(
        fielded(&opening),
        ["clone_roland", "clone_elara", "clone_gareth"]
    );

    // A line-up built from the later recruits reflects a different set of clones —
    // proof the fight tracks whoever you actually bring, not a fixed roster.
    let veterans: HashSet<&str> = ["swordsman", "axeman", "captain"].into_iter().collect();
    assert_eq!(
        fielded(&veterans),
        ["clone_roland", "clone_brenn", "clone_captain"]
    );

    // Leave the mage in reserve and her dark double never appears.
    let no_mage: HashSet<&str> = ["swordsman", "hermit"].into_iter().collect();
    assert_eq!(fielded(&no_mage), ["clone_roland", "clone_gareth"]);

    // A lone swordsman faces only his own reflection.
    let solo: HashSet<&str> = ["swordsman"].into_iter().collect();
    assert_eq!(fielded(&solo), ["clone_roland"]);

    // An ordinary encounter (foes with no `mirrors`) is unchanged regardless of
    // who is active — the filter is the identity for it.
    if let Some(other) = reg.data.encounters.iter().find(|e| {
        e.enemies
            .iter()
            .all(|id| reg.enemy(id).is_some_and(|d| d.mirrors.is_none()))
    }) {
        let empty: HashSet<&str> = HashSet::new();
        let fielded: Vec<&String> = active_encounter_enemies(&reg, &other.enemies, &empty);
        assert_eq!(
            fielded.len(),
            other.enemies.len(),
            "ordinary encounter '{}' must be unaffected by the mirror filter",
            other.id
        );
    }
}

/// The DEMON KING is the scripted, unwinnable climax of chapter 1: an invincible
/// boss whose encounter turns a party wipe into a chapter transition. Guards the
/// whole mechanism — the invincible flag, the lethal kit, and the defeat hooks —
/// and confirms he is actually placed at the end of the DEMON FACILITY.
#[test]
fn demon_king_is_the_unwinnable_chapter_boss() {
    let reg = registry();

    // The king himself: invincible (cannot be killed) with resolvable skills.
    let king = reg.enemy("demon_king").expect("missing demon_king enemy");
    assert!(king.invincible, "the demon king must be invincible");
    assert!(
        !king.skills.is_empty(),
        "the demon king needs an attack kit"
    );
    for id in &king.skills {
        assert!(
            reg.skill(id).is_some(),
            "demon king references unknown skill '{id}'"
        );
    }
    // No ordinary foe should be invincible — that flag is the scripted boss's alone.
    for e in &reg.data.enemies {
        if e.id != "demon_king" {
            assert!(!e.invincible, "ordinary enemy '{}' is invincible", e.id);
        }
    }

    // The encounter: a boss fight that scripts the defeat into a chapter jump.
    let enc = reg
        .encounter("demon_king")
        .expect("missing demon_king encounter");
    assert!(enc.boss, "the demon king fight should play the boss theme");
    assert_eq!(enc.enemies, vec!["demon_king"]);
    assert!(
        enc.defeat_advances_chapter,
        "losing to the demon king must advance the chapter"
    );
    let cs = enc
        .defeat_cutscene
        .as_deref()
        .expect("the demon king needs a defeat cutscene");
    assert!(
        reg.cutscene(cs).is_some(),
        "demon king defeat cutscene '{cs}' does not resolve"
    );

    // Placement: the king stands at the end of the DEMON FACILITY, and nowhere
    // does an ordinary spawn field this scripted boss.
    let facility = reg
        .data
        .levels
        .iter()
        .find(|l| l.id == "demonfacility")
        .expect("missing demonfacility level");
    let placed = facility
        .screens
        .iter()
        .flat_map(|s| &s.spawns)
        .any(|sp| sp.encounter == "demon_king");
    assert!(placed, "the demon king is not placed in the demon facility");
}

/// The CASTAWAY SHORE is the first region of chapter 2 — where the party washes up
/// after the DEMON KING flings them to the surface. Guards its chapter tag, its
/// beach theming (sand / barricade / coconut palms), and that it fields the new
/// shore foes; and confirms the chapter mechanism it depends on lines up (the
/// king's defeat advances into exactly the chapter this level lives in).
#[test]
fn castaway_shore_is_the_chapter_2_landing() {
    let reg = registry();

    let shore = reg
        .data
        .levels
        .iter()
        .find(|l| l.id == "castawayshore")
        .expect("missing castawayshore level");
    assert_eq!(shore.chapter, 2, "the castaway shore is a chapter 2 region");
    assert_eq!(shore.ground.as_deref(), Some("sand"), "the shore is sand");
    assert_eq!(
        shore.wall.as_deref(),
        Some("barricade"),
        "the shore's walls are the pirates' wooden barricades"
    );
    assert_eq!(
        shore.tree.as_deref(),
        Some("coconut_tree"),
        "the shore is lined with coconut palms"
    );

    // It is the landing region the DEMON KING's defeat delivers the party into: the
    // first (and, for now, only) chapter-2 level in progression order.
    let first_ch2 = reg.data.levels.iter().find(|l| l.chapter == 2);
    assert_eq!(
        first_ch2.map(|l| l.id.as_str()),
        Some("castawayshore"),
        "the castaway shore should be the first chapter 2 region"
    );

    // The pirate gunner is a real ranged foe: its shot resolves and flies a bullet.
    let gunner = reg.enemy("pirate_gunner").expect("missing pirate_gunner");
    assert!(
        gunner.skills.iter().any(|s| s == "pistol_shot"),
        "the gunner should carry a pistol shot"
    );
    let shot = reg.skill("pistol_shot").expect("missing pistol_shot skill");
    assert!(
        matches!(&shot.anim, AttackAnim::Projectile { texture } if embedded_texture(texture).is_some()),
        "the pistol shot should fire an embedded projectile"
    );

    // Every foe the shore fields resolves and is a shore denizen (crabs, the
    // pirate crew, and their captain — the boss who then joins the party).
    let shore_foes = [
        "beach_crab",
        "pirate_grunt",
        "pirate_gunner",
        "pirate_captain",
    ];
    for id in shore_foes {
        assert!(reg.enemy(id).is_some(), "missing shore enemy '{id}'");
    }
    for sp in shore.screens.iter().flat_map(|s| &s.spawns) {
        let enc = reg.encounter(&sp.encounter).unwrap_or_else(|| {
            panic!(
                "shore spawn references unknown encounter '{}'",
                sp.encounter
            )
        });
        for eid in &enc.enemies {
            assert!(
                shore_foes.contains(&eid.as_str()),
                "castaway shore fields a non-shore foe '{eid}'"
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
    for key in [
        "grass",
        "water",
        "tree",
        "rock",
        "barricade",
        "chest",
        "mimic",
    ] {
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
    // Per-level ground/wall/tree overrides (stone, dark_floor, coconut_tree, …).
    for lv in &reg.data.levels {
        for key in [&lv.ground, &lv.wall, &lv.tree].into_iter().flatten() {
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
                // A choreography actor is drawn from a real character/enemy sprite.
                CutsceneStep::Place { character, .. } => assert!(
                    reg.character(character).is_some() || reg.enemy(character).is_some(),
                    "cutscene {} places unknown character '{character}'",
                    cs.id
                ),
                // The remaining steps carry no content id to validate.
                CutsceneStep::Say { .. }
                | CutsceneStep::Walk { .. }
                | CutsceneStep::Turn { .. }
                | CutsceneStep::Leave { .. }
                | CutsceneStep::Pan { .. }
                | CutsceneStep::Wait { .. } => {}
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

/// The PIRATE CAPTAIN is a boss you fight who then joins the party: the enemy and
/// the recruited character share one sprite, the boss encounter is winnable (not
/// invincible), and the shore's clear cutscene recruits him. Guards the whole
/// fight-then-join wiring, including the musket both forms carry.
#[test]
fn pirate_captain_is_a_boss_who_joins() {
    let reg = registry();

    // The recruitable hero form, with his musket and a resolvable learnset.
    let hero = reg.character("captain").expect("missing captain character");
    assert!(
        hero.skills.iter().any(|s| s == "musket_shot"),
        "the captain fights with a musket"
    );
    for ls in &hero.learnset {
        assert!(
            reg.skill(&ls.skill).is_some(),
            "captain learnset skill '{}' does not resolve",
            ls.skill
        );
    }

    // The boss form: winnable (NOT invincible), sharing the hero's sprite so foe and
    // ally look the same, and fielded by a boss encounter.
    let boss = reg
        .enemy("pirate_captain")
        .expect("missing pirate_captain enemy");
    assert!(!boss.invincible, "the captain must be beatable");
    assert_eq!(
        boss.sprite.texture, hero.sprite.texture,
        "the captain boss and ally should share a sprite"
    );
    let enc = reg
        .encounter("pirate_captain")
        .expect("missing pirate_captain encounter");
    assert!(enc.boss, "the captain fight should play the boss theme");

    // The shore recruits him on clear: its clear cutscene has a Recruit(captain)
    // step, and he is actually placed as a spawn in the level.
    let shore = reg
        .data
        .levels
        .iter()
        .find(|l| l.id == "castawayshore")
        .expect("missing castawayshore");
    assert_eq!(
        shore.clear_cutscene.as_deref(),
        Some("captain_joins"),
        "clearing the shore should play captain_joins"
    );
    let cs = reg
        .cutscene("captain_joins")
        .expect("missing captain_joins");
    let recruits_captain = cs
        .steps
        .iter()
        .any(|s| matches!(s, CutsceneStep::Recruit { character } if character == "captain"));
    assert!(recruits_captain, "captain_joins must recruit the captain");
    let placed = shore
        .screens
        .iter()
        .flat_map(|s| &s.spawns)
        .any(|sp| sp.encounter == "pirate_captain");
    assert!(placed, "the captain boss is not placed on the shore");
}

/// The captain's finale, ALL HANDS, is a `Crowd` attack — the new animation type
/// that floods the screen with allies. Guards the whole wiring: the skill is a
/// screen-wide crowd whose swarm art is embedded, and the captain learns it as his
/// **final** move (his highest-level learnset entry).
#[test]
fn captains_finale_is_a_crowd_attack() {
    let reg = registry();

    let all_hands = reg.skill("all_hands").expect("missing all_hands skill");
    assert_eq!(
        all_hands.target,
        TargetKind::AllEnemies,
        "ALL HANDS should sweep the whole enemy line"
    );
    match &all_hands.anim {
        AttackAnim::Crowd { texture } => assert!(
            embedded_texture(texture).is_some(),
            "the crowd's swarm texture '{texture}' is not embedded"
        ),
        other => panic!("ALL HANDS should be a Crowd attack, got {other:?}"),
    }

    // It is the captain's final learned move: present, and the deepest learnset entry.
    let captain = reg.character("captain").expect("missing captain");
    let last = captain
        .learnset
        .iter()
        .max_by_key(|ls| ls.level)
        .expect("captain has a learnset");
    assert_eq!(
        last.skill, "all_hands",
        "ALL HANDS should be the captain's final (highest-level) move"
    );
}

/// Only the active line-up fights: the roster can hold more than [`ACTIVE_PARTY`]
/// members, and the overworld reorder (mirrored here by a plain swap) is what pulls
/// a reserve into the first `ACTIVE_PARTY` slots — the ones the battle seats.
#[test]
fn battle_line_up_caps_the_active_party() {
    assert_eq!(ACTIVE_PARTY, 3, "a battle seats three heroes");

    let reg = registry();
    let mut party = Party::from_registry(&reg);
    for id in ["mage", "hermit", "captain"] {
        party.recruit(&reg, id);
    }
    assert!(
        party.members.len() > ACTIVE_PARTY,
        "the roster can run deeper than the battle line-up"
    );

    // The captain joins last, so he starts on the bench (outside the first
    // ACTIVE_PARTY). Reordering (a menu MOVE UP, here a swap) deploys him.
    let bench = party.members.len() - 1;
    assert!(bench >= ACTIVE_PARTY, "captain starts in reserve");
    let active_before: Vec<_> = party.members[..ACTIVE_PARTY]
        .iter()
        .map(|m| m.def_id.clone())
        .collect();
    assert!(
        !active_before.contains(&"captain".to_string()),
        "captain should not fight until swapped in"
    );
    // The leader (Roland, slot 0) is fixed — he walks the overworld, so reordering
    // never touches slot 0. Deploying a reserve swaps within slots 1.., here slot 2.
    assert_eq!(
        party.members[0].def_id, "swordsman",
        "Roland leads the roster at slot 0"
    );
    party.members.swap(ACTIVE_PARTY - 1, bench);
    assert_eq!(
        party.members[0].def_id, "swordsman",
        "the leader stays put when the rest reorder"
    );
    assert!(
        party.members[..ACTIVE_PARTY]
            .iter()
            .any(|m| m.def_id == "captain"),
        "swapping a reserve up puts them in the active line-up"
    );
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

/// Restorative items can be used from the field: a potion heals the chosen hero
/// and is consumed only when it actually does something (a full-HP hero refuses
/// it, wasting nothing).
#[test]
fn field_item_heals_and_is_spent_only_when_useful() {
    let reg = registry();
    let mut party = Party::from_registry(&reg);
    party.add_item("potion", 2);
    // Wound the leader so a potion has something to mend.
    party.members[0].hp = 1;

    let idx = party.items.iter().position(|s| s.id == "potion").unwrap();
    match party.use_item_in_field(&reg, idx, 0) {
        FieldUse::Restored { hp, mp } => {
            assert!(hp > 0, "the potion should restore HP");
            assert_eq!(mp, 0);
        }
        _ => panic!("a wounded hero should be healed by a potion"),
    }
    assert!(party.members[0].hp > 1, "HP went up");
    assert_eq!(
        party.items.iter().find(|s| s.id == "potion").unwrap().count,
        1,
        "one potion was spent"
    );

    // Full HP now (heal is generous): the second potion is refused and NOT spent.
    party.members[0].hp = party.members[0].stats.max_hp;
    let idx = party.items.iter().position(|s| s.id == "potion").unwrap();
    assert!(matches!(
        party.use_item_in_field(&reg, idx, 0),
        FieldUse::NoEffect
    ));
    assert_eq!(
        party.items.iter().find(|s| s.id == "potion").unwrap().count,
        1,
        "a wasted use spends nothing"
    );
}

/// Items only top up the living — a potion can't revive a downed hero (that's
/// MEND's job), and a refused use spends nothing.
#[test]
fn field_items_do_not_revive() {
    let reg = registry();
    let mut party = Party::from_registry(&reg);
    party.add_item("potion", 1);
    party.members[0].hp = 0; // downed
    let idx = party.items.iter().position(|s| s.id == "potion").unwrap();
    assert!(matches!(
        party.use_item_in_field(&reg, idx, 0),
        FieldUse::NoEffect
    ));
    assert!(!party.members[0].is_alive(), "a potion must not revive");
    assert_eq!(
        party.items.iter().find(|s| s.id == "potion").unwrap().count,
        1,
        "a refused use spends nothing"
    );
}

/// Healing moves work from the field too: a hero who knows MEND can cast it on an
/// ally, spending the caster's MP; an unaffordable cast is refused for free.
#[test]
fn field_heal_move_restores_hp_and_bills_mp() {
    let reg = registry();
    // MEND belongs to ELARA alone now, and is her last-unlocked (capstone) move —
    // build a party of just her and level her up until she has learned it.
    let mut party = Party::default();
    assert!(party.recruit(&reg, "mage"), "ELARA (mage) should recruit");
    while !party.members[0].skills.iter().any(|id| id == "mend") {
        assert!(
            party.members[0].level < 50,
            "ELARA should learn MEND by levelling"
        );
        party.grant_xp(&reg, 1000);
    }

    // It shows up as a field-usable healing move.
    let moves = party.field_heal_skills(&reg, 0);
    assert!(
        moves.iter().any(|id| id == "mend"),
        "MEND should be a field heal move"
    );

    // Wound her and have her mend herself.
    party.members[0].hp = 1;
    party.members[0].mp = party.members[0].stats.max_mp;
    let mp_before = party.members[0].mp;
    match party.use_heal_skill_in_field(&reg, 0, "mend", 0) {
        FieldUse::Restored { hp, .. } => assert!(hp > 0, "MEND should restore HP"),
        _ => panic!("MEND should heal a wounded ally"),
    }
    assert!(party.members[0].hp > 1);
    assert!(party.members[0].mp < mp_before, "casting spent MP");

    // Drain MP below the cost: the next cast is refused and bills nothing.
    party.members[0].hp = 1;
    party.members[0].mp = 0;
    assert!(matches!(
        party.use_heal_skill_in_field(&reg, 0, "mend", 0),
        FieldUse::NotEnoughMp
    ));
    assert_eq!(party.members[0].mp, 0, "a refused cast spends no MP");
}

/// MEND revives: cast on a **downed** ally it brings them back up (a non-reviving
/// heal would refuse a fallen target). Exercised through the field-use path.
#[test]
fn field_mend_revives_a_downed_ally() {
    let reg = registry();
    assert!(
        reg.skill("mend").is_some_and(|s| s.revives),
        "MEND should be flagged as reviving"
    );

    let mut party = Party::default();
    assert!(party.recruit(&reg, "mage")); // ELARA — the caster (member 0)
    assert!(party.recruit(&reg, "swordsman")); // ROLAND — will be downed (member 1)
    while !party.members[0].skills.iter().any(|id| id == "mend") {
        assert!(party.members[0].level < 50, "ELARA should learn MEND");
        party.grant_xp(&reg, 1000);
    }

    // Knock ROLAND out, then have ELARA mend him back.
    party.members[1].hp = 0;
    assert!(!party.members[1].is_alive(), "ROLAND is down");
    party.members[0].mp = party.members[0].stats.max_mp;
    match party.use_heal_skill_in_field(&reg, 0, "mend", 1) {
        FieldUse::Restored { hp, .. } => assert!(hp > 0, "revive restores HP"),
        _ => panic!("MEND should revive a downed ally"),
    }
    assert!(
        party.members[1].is_alive(),
        "the fallen ally is back on their feet"
    );
}

/// MEND is exclusive to ELARA — no other character knows it from the start or via
/// their learnset. The warrior and scout get their own capstone instead.
#[test]
fn only_elara_learns_mend() {
    let reg = registry();
    for c in &reg.data.characters {
        let has_mend =
            c.skills.iter().any(|s| s == "mend") || c.learnset.iter().any(|l| l.skill == "mend");
        if c.id == "mage" {
            assert!(has_mend, "ELARA should learn MEND");
        } else {
            assert!(
                !has_mend,
                "{} must not know MEND (it's ELARA's alone)",
                c.id
            );
        }
    }
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
    party.grant_xp(&reg, 1000);
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

    let leveled = party.grant_xp(&reg, 1000);
    assert!(
        leveled.iter().any(|e| e.member == 0),
        "member 0 should have leveled up"
    );
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

/// Characters unlock skills as they level up: a member starts without their
/// learnset moves, gains them on reaching the unlock level (reported back), and a
/// mid-game recruit brought up to the party's level arrives already knowing them.
#[test]
fn levelling_unlocks_skills() {
    let reg = registry();

    // Pick any character with a learnset and its earliest unlock.
    let (cid, unlock_level, unlock_skill) = reg
        .data
        .characters
        .iter()
        .find_map(|c| {
            c.learnset
                .iter()
                .min_by_key(|l| l.level)
                .map(|l| (c.id.clone(), l.level, l.skill.clone()))
        })
        .expect("some character should have a learnset to exercise");

    // A fresh, level-1 party member does not yet know the gated skill...
    let mut party = Party::default();
    assert!(party.recruit(&reg, &cid));
    let idx = party.members.len() - 1;
    assert!(
        !party.members[idx].skills.contains(&unlock_skill),
        "{unlock_skill} should be locked at level 1"
    );

    // ...but earns it on reaching the unlock level, and it's reported as learned.
    let mut announced = false;
    while party.members[idx].level < unlock_level {
        for ev in party.grant_xp(&reg, 1000) {
            if ev.member == idx && ev.learned.contains(&unlock_skill) {
                announced = true;
            }
        }
    }
    assert!(
        party.members[idx].skills.contains(&unlock_skill),
        "{unlock_skill} should be known at level {unlock_level}"
    );
    assert!(announced, "learning {unlock_skill} should be reported");

    // A recruit joining an already-high-level party arrives knowing it outright.
    let mut veteran = Party::default();
    // Level an initial member well past the unlock so the party level is high.
    veteran.recruit(&reg, &reg.data.starting_party[0]);
    while veteran.level() < unlock_level {
        veteran.grant_xp(&reg, 1000);
    }
    assert!(veteran.recruit(&reg, &cid));
    let late = veteran.members.len() - 1;
    assert!(
        veteran.members[late].skills.contains(&unlock_skill),
        "a recruit at level {} should already know {unlock_skill}",
        veteran.level()
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
    party.grant_xp(&reg, 1000);
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

/// Consumable items are well-formed and every reference around them resolves: an
/// item's icon is embedded and its inflicted statuses exist; enemy drop tables
/// name real items at valid odds. This is the items extensibility contract —
/// adding an item / drop is pure data, and this guards those data edits.
#[test]
fn item_and_drop_references_resolve() {
    let reg = registry();

    for it in &reg.data.items {
        assert!(
            embedded_texture(&it.icon).is_some(),
            "item {} icon '{}' not embedded",
            it.id,
            it.icon
        );
        assert!(
            !it.description.trim().is_empty(),
            "item {} has no description",
            it.id
        );
        assert!(it.price >= 0, "item {} has a negative price", it.id);
        for sid in &it.effect.inflicts {
            assert!(
                reg.status(sid).is_some(),
                "item {} inflicts unknown status '{sid}'",
                it.id
            );
        }
    }

    for e in &reg.data.enemies {
        for d in &e.drops {
            assert!(
                reg.item(&d.item).is_some(),
                "enemy {} drops unknown item '{}'",
                e.id,
                d.item
            );
            assert!(
                (0.0..=1.0).contains(&d.chance),
                "enemy {} drop '{}' has an out-of-range chance {}",
                e.id,
                d.item,
                d.chance
            );
        }
    }
}

/// Equipment actually changes a battler's effective stats (the extensibility
/// contract for gear: bonuses apply without touching battle code).
#[test]
fn equipping_gear_changes_effective_stats() {
    let reg = registry();
    // ROLAND, with iron_sword + leather_armor (looked up by id — the characters
    // collection is filename-sorted, so index 0 isn't necessarily him).
    let sword = reg.character("swordsman").expect("missing swordsman");
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
            // A stock line is either a piece of equipment or a consumable item.
            assert!(
                reg.equipment(&s.item).is_some() || reg.item(&s.item).is_some(),
                "shop {} stocks unknown ware '{}'",
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

/// Townsfolk are well-formed and placed on standable ground: every NPC
/// appearance has a name and an embedded sprite, and every placement is
/// in-bounds, standable, names a real appearance, shows a registered emote
/// bubble, and resolves any one-time cutscene / recruit it carries. Guards the
/// NPC extensibility contract the same way spawns and shops are guarded — a
/// missing emote or a recruit pointing at a phantom character would otherwise
/// only panic (or silently break) once the town is reached in play.
#[test]
fn npcs_are_valid_and_placed() {
    let reg = registry();

    for npc in &reg.data.npcs {
        assert!(!npc.name.trim().is_empty(), "npc {} has no name", npc.id);
        assert!(
            embedded_texture(&npc.sprite.texture).is_some(),
            "npc {} sprite texture '{}' not embedded",
            npc.id,
            npc.sprite.texture
        );
    }

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
            for np in &sc.npcs {
                assert!(
                    (np.col as usize) < width && (np.row as usize) < height,
                    "level {} screen {si} npc ({},{}) out of bounds",
                    lv.id,
                    np.col,
                    np.row
                );
                assert!(
                    !solid(np.col, np.row),
                    "level {} screen {si} npc ({},{}) sits in a solid tile",
                    lv.id,
                    np.col,
                    np.row
                );
                assert!(
                    reg.npc(&np.npc).is_some(),
                    "level {} screen {si} npc references unknown appearance '{}'",
                    lv.id,
                    np.npc
                );
                let emote_key = format!("{}_emote", np.emote);
                assert!(
                    embedded_texture(&emote_key).is_some(),
                    "level {} screen {si} npc emote '{}' has no embedded texture",
                    lv.id,
                    np.emote
                );
                if let Some(cs) = &np.cutscene {
                    assert!(
                        reg.cutscene(cs).is_some(),
                        "level {} screen {si} npc names unknown cutscene '{cs}'",
                        lv.id
                    );
                }
                if let Some(rec) = &np.recruits {
                    assert!(
                        reg.character(rec).is_some(),
                        "level {} screen {si} npc recruits unknown character '{rec}'",
                        lv.id
                    );
                }
            }
        }
    }
}

/// Every house stamped on a screen fits entirely within that screen's bounds
/// (its fixed 6×4 footprint can't hang off the map), and the shared house
/// tileset (`house_0`..`house_23`) is fully embedded. A house whose base row
/// fell off the grid would inject its solid wall out of bounds.
#[test]
fn houses_fit_and_tileset_is_embedded() {
    // House tileset dimensions, mirrored from `crate::overworld` (kept in sync
    // by this very test — a resized tileset would need both updated).
    const HOUSE_COLS: u32 = 6;
    const HOUSE_ROWS: u32 = 4;
    for i in 0..(HOUSE_COLS * HOUSE_ROWS) {
        assert!(
            embedded_texture(&format!("house_{i}")).is_some(),
            "house tile 'house_{i}' not embedded"
        );
    }

    let reg = registry();
    for lv in &reg.data.levels {
        for (si, sc) in lv.screens.iter().enumerate() {
            let width = sc.map.iter().map(|r| r.chars().count()).max().unwrap() as u32;
            let height = sc.map.len() as u32;
            for h in &sc.houses {
                assert!(
                    h.col + HOUSE_COLS <= width && h.row + HOUSE_ROWS <= height,
                    "level {} screen {si} house at ({},{}) overhangs the {}x{} screen",
                    lv.id,
                    h.col,
                    h.row,
                    width,
                    height
                );
            }
        }
    }
}

/// A talkable NPC that recruits must be backed by a cutscene whose script
/// actually adds that character to the party — otherwise the townsfolk vanishes
/// on being talked to but never joins. Covers HARBORWATCH's BRENN specifically.
#[test]
fn recruiting_npcs_actually_recruit() {
    let reg = registry();
    for lv in &reg.data.levels {
        for sc in &lv.screens {
            for np in &sc.npcs {
                let (Some(rec), Some(cs_id)) = (&np.recruits, &np.cutscene) else {
                    continue;
                };
                let cs = reg
                    .cutscene(cs_id)
                    .unwrap_or_else(|| panic!("npc cutscene '{cs_id}' missing"));
                assert!(
                    cs.steps.iter().any(|s| matches!(
                        s,
                        CutsceneStep::Recruit { character } if character == rec
                    )),
                    "level {} npc recruits '{rec}' but cutscene '{cs_id}' never adds them",
                    lv.id
                );
            }
        }
    }
}

/// Chests and mimics are well-formed and placed on standable ground: every chest
/// holds *something* and names only real loot, every mimic names a real
/// encounter, and neither is stranded in a wall or off the map. Guards the
/// treasure/ambush extensibility contract exactly as spawns and shops are guarded.
#[test]
fn chests_and_mimics_are_valid_and_placed() {
    let reg = registry();

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

            for ch in &sc.chests {
                assert!(
                    (ch.col as usize) < width && (ch.row as usize) < height,
                    "level {} screen {si} chest ({},{}) out of bounds",
                    lv.id,
                    ch.col,
                    ch.row
                );
                assert!(
                    !solid(ch.col, ch.row),
                    "level {} screen {si} chest ({},{}) sits in a solid tile",
                    lv.id,
                    ch.col,
                    ch.row
                );
                assert!(ch.gold >= 0, "level {} chest has negative gold", lv.id);
                assert!(
                    ch.gold > 0 || ch.item.is_some() || ch.equipment.is_some(),
                    "level {} screen {si} chest ({},{}) is empty",
                    lv.id,
                    ch.col,
                    ch.row
                );
                if let Some(id) = &ch.item {
                    assert!(
                        reg.item(id).is_some(),
                        "level {} chest holds unknown item '{id}'",
                        lv.id
                    );
                }
                if let Some(id) = &ch.equipment {
                    assert!(
                        reg.equipment(id).is_some(),
                        "level {} chest holds unknown equipment '{id}'",
                        lv.id
                    );
                }
            }

            for m in &sc.mimics {
                assert!(
                    (m.col as usize) < width && (m.row as usize) < height,
                    "level {} screen {si} mimic ({},{}) out of bounds",
                    lv.id,
                    m.col,
                    m.row
                );
                assert!(
                    !solid(m.col, m.row),
                    "level {} screen {si} mimic ({},{}) sits in a solid tile",
                    lv.id,
                    m.col,
                    m.row
                );
                assert!(
                    reg.encounter(&m.encounter).is_some(),
                    "level {} screen {si} mimic references unknown encounter '{}'",
                    lv.id,
                    m.encounter
                );
            }
        }
    }
}
