//! Fast, display-free tests for the data layer and party mechanics. These run
//! on every `cargo test` and guard the extensibility contract: content is valid
//! and cross-referenced correctly.

use hero_of_the_overworld::data::{embedded_texture, Registry};
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
            assert!(reg.skill(s).is_some(), "character {} skill '{s}' missing", c.id);
        }
    }
    for e in &reg.data.enemies {
        for s in &e.skills {
            assert!(reg.skill(s).is_some(), "enemy {} skill '{s}' missing", e.id);
        }
    }
    // Every enemy referenced by an encounter exists.
    for enc in &reg.data.encounters {
        assert!(!enc.enemies.is_empty(), "encounter {} has no enemies", enc.id);
        for eid in &enc.enemies {
            assert!(reg.enemy(eid).is_some(), "encounter {} enemy '{eid}' missing", enc.id);
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

#[test]
fn xp_grants_levels_and_growth() {
    let reg = registry();
    let mut party = Party::from_registry(&reg);
    let atk_before = party.members[0].stats.attack;
    let hp_before = party.members[0].stats.max_hp;

    let leveled = party.grant_xp(1000);
    assert!(leveled.contains(&0), "member 0 should have leveled up");
    assert!(party.members[0].level > 1);
    assert!(party.members[0].stats.attack > atk_before, "attack should grow");
    assert!(party.members[0].stats.max_hp > hp_before, "max hp should grow");
}
