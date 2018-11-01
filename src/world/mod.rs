use std::error;
use std::fmt;

mod internal {

    // Included to trigger a rebuild when the file is changed:
    #[cfg(debug_assertions)]
    const _WORLD_GRAMMAR: &str = include_str!("world.pest");

    #[derive(Parser)]
    #[grammar = "world/world.pest"]
    pub struct WorldParser;
}

#[derive(Debug)]
pub struct ParseError;

impl error::Error for ParseError {
    fn description(&self) -> &str {
        "invalid world file"
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "got parse error")
    }
}

#[derive(Debug, PartialEq)]
pub enum Alignment {
    Unaligned,
    Good,
    Neutral,
    Evil,
}

#[derive(Debug)]
pub struct ZoneDef {
    pub nr: u32,
    pub n_rooms: u32,
    pub name: String,
    pub align: Option<Alignment>,
    pub owner: Option<String>,
    pub uid: Option<u32>,
}

#[derive(Debug)]
pub struct WorldList {
    pub n_things: u32,
    pub n_monsters: u32,
    pub n_creatables: u32,
    pub zones: Vec<ZoneDef>,
    pub file_names: Vec<String>,
}

use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::RuleType;

fn parse_zones(p: &mut Pairs<self::internal::Rule>) -> Vec<ZoneDef> {
    let mut res = vec![];

    use self::internal::Rule;

    for zone in p {
        if zone.as_rule() != Rule::zone_defs {
            break;
        }
        let mut zd = zone.into_inner();
        let nr: u32 = zd.next().unwrap().as_str().parse().unwrap();
        let n_rooms: u32 = zd.next().unwrap().as_str().parse().unwrap();
        let name: String = zd.next().unwrap().as_str().to_owned();
        let align = if let Some(align) = zd.next() {
            match align.as_str().to_ascii_lowercase().as_str() {
                "unaligned" => Some(Alignment::Unaligned),
                "good" => Some(Alignment::Good),
                "neutral" => Some(Alignment::Neutral),
                "evil" => Some(Alignment::Evil),
                _ => None,
            }
        } else {
            None
        };
        let owner = if let Some(owner) = zd.next() {
            Some(owner.as_str().to_owned())
        } else {
            None
        };
        let uid = if let Some(uid) = zd.next() {
            Some(uid.as_str().parse::<u32>().unwrap())
        } else {
            None
        };
        res.push(ZoneDef {
            nr,
            n_rooms,
            name,
            align,
            owner,
            uid,
        });
    }

    res
}

pub fn parse(world: &str) -> Result<WorldList, ParseError> {
    use pest::Parser;

    let result = internal::WorldParser::parse(internal::Rule::world, world);
    if result.is_err() {
        println!("Parse error: {:?}", result);
        return Err(ParseError);
    }
    let mut result = result.unwrap().next().unwrap().into_inner();

    let things = result.next().unwrap();
    let monsters = result.next().unwrap();
    let creatables = result.next().unwrap();
    let zones = result.next().unwrap();

    fn get_num<T: RuleType>(p: Pair<T>) -> u32 {
        p.into_inner().next().unwrap().as_str().parse().unwrap()
    }
    let things = get_num(things);
    let monsters = get_num(monsters);
    let creatables = get_num(creatables);
    let _zones = get_num(zones);

    let zones = parse_zones(&mut result);

    Ok(WorldList {
        n_things: things,
        n_monsters: monsters,
        n_creatables: creatables,
        zones,
        file_names: vec![],
    })
}

#[cfg(test)]
mod tests {

    #[test]
    fn parse_world() {
        use crate::world::Alignment;

        let world = include_str!("world.test");
        let result = crate::world::parse(world);
        assert!(result.is_ok(), "Not correct parse result: {:?}", result);
        let result = result.unwrap();
        assert_eq!(result.n_things, 1234);
        assert_eq!(result.n_monsters, 234);
        assert_eq!(result.n_creatables, 345);
        assert_eq!(result.zones.len(), 6);
        assert_eq!(result.zones[0].nr, 1);
        assert_eq!(result.zones[0].n_rooms, 24);
        assert_eq!(result.zones[0].align, None);
        assert_eq!(result.zones[0].name, "first");
        assert_eq!(result.zones[0].uid, None);
        assert_eq!(result.zones[1].nr, 2);
        assert_eq!(result.zones[1].n_rooms, 53);
        assert_eq!(result.zones[1].align, Some(Alignment::Unaligned));
        assert_eq!(result.zones[1].owner, Some("Kaladrin".to_owned()));
        assert_eq!(result.zones[2].nr, 3);
        assert_eq!(result.zones[2].align, Some(Alignment::Good));
        assert_eq!(result.zones[3].nr, 10);
        assert_eq!(result.zones[3].align, Some(Alignment::Evil));
        assert_eq!(result.zones[4].nr, 11);
        assert_eq!(result.zones[4].align, Some(Alignment::Neutral));
        assert_eq!(result.zones[5].nr, 12);
        assert_eq!(result.zones[5].name, "six and space");
        assert_eq!(result.zones[5].uid, Some(75));
    }
}
