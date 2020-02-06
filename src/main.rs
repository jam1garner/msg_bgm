use std::collections::HashMap;
use std::io::{Cursor, prelude::*};
use hash40::{Hash40, hash40};
use prc::param::ParamKind;

// name <- [mbst] <- name_id <- [db_root] <- stream_set_id <- [stream_set] <- info# <- info_id <-
// <- [assigned_info] <- stream_id <- [stream_property] <- data_name# -> [arc stream:/] -> file

const DB_ROOT: Hash40 = hash40!("db_root");
const STREAM_PROPERTY: Hash40 = hash40!("stream_property");
const STREAM_ID: Hash40 = hash40!("stream_id");
const DATA_NAMES: &[Hash40] = &[
    hash40!("data_name0"),
    hash40!("data_name1"),
    hash40!("data_name2"),
    hash40!("data_name3"),
    hash40!("data_name4")
];
const INFOS: &[Hash40] = &[
    hash40!("info0"),
    hash40!("info1"),
    hash40!("info2"),
    hash40!("info3"),
    hash40!("info4"),
    hash40!("info5"),
    hash40!("info6"),
    hash40!("info7"),
    hash40!("info8"),
    hash40!("info9"),
    hash40!("info10"),
    hash40!("info11"),
    hash40!("info12"),
    hash40!("info13"),
    hash40!("info14"),
    hash40!("info15"),
];
const ASSIGNED_INFO: Hash40 = hash40!("assigned_info");
const STREAM_SET_ID: Hash40 = hash40!("stream_set_id");
const STREAM_SET: Hash40 = hash40!("stream_set");
const NAME_ID: Hash40 = hash40!("name_id");
const INFO_ID: Hash40 = hash40!("info_id");

fn clean_up_string(strg: &String) -> String {
    strg.split("\x0E\0\x02\x02P")
        .collect::<String>()
        .split("\x0E\0\x02\x02d")
        .collect::<String>()
        .split("\0")
        .collect()
}

fn main() {
    hash40::set_labels(hash40::read_labels("Labels.csv").unwrap());

    let msg_bgm = include_str!("msg_bgm.csv");
    const PREFIX: &str = "bgm_title_";
    let id_to_name = msg_bgm.split('\n')
        .filter(|s| s.starts_with(PREFIX))
        .map(|s|{
            let s = s.split(',').collect::<Vec<_>>();
            if let &[key, ..] = s.as_slice() {
                let key = String::from(&key[PREFIX.len()..]);
                let value = s[1..].join(",");
                let value = String::from(&value[1..value.len() - 2]);
                (key, value)
            } else {
                panic!("Bad format {:?}", s);
            }
        })
        .collect::<HashMap<String, String>>();

    let mut ui_bgm_db = Cursor::new(&include_bytes!("ui_bgm_db.prc")[..]);

    let ui_bgm_db: HashMap<_,_> = prc::read_stream(&mut ui_bgm_db).unwrap().into_iter().collect();

    let db_root = ui_bgm_db.get(&DB_ROOT).unwrap();
    let db_root: &Vec<ParamKind> = db_root.unwrap();

    let stream_property: &Vec<ParamKind> = ui_bgm_db.get(&STREAM_PROPERTY).unwrap().unwrap();

    let stream_id_to_data_names =
        stream_property
            .into_iter()
            .map(|s|{
                let s = s.unwrap_as_hashmap().unwrap();
                let stream_id: &Hash40 = s.get(&STREAM_ID).unwrap().unwrap();
                let data_names =
                    DATA_NAMES
                        .iter()
                        .map(|data_name|{
                            s.get(data_name).unwrap().unwrap::<String>()
                        })
                        .filter(|data_name| !data_name.trim().is_empty())
                        .cloned()
                        .collect::<Vec<_>>();
                (*stream_id, data_names)
            })
            .collect::<HashMap<_,_>>();

    let assigned_info: &Vec<ParamKind> = ui_bgm_db.get(&ASSIGNED_INFO).unwrap().unwrap();

    let info_id_to_stream_id =
        assigned_info
            .into_iter()
            .map(|ass_info|{
                let ass_info = ass_info.unwrap_as_hashmap().unwrap();
                let info_id: Hash40 = *ass_info.get(&INFO_ID).unwrap().unwrap(); 
                let stream_id: Hash40 = *ass_info.get(&STREAM_ID).unwrap().unwrap(); 
                (info_id, stream_id)
            })
            .collect::<HashMap<_,_>>();

    let stream_set: &Vec<ParamKind> = ui_bgm_db.get(&STREAM_SET).unwrap().unwrap();

    let stream_set_id_to_infos =
        stream_set
            .into_iter()
            .map(|s|{
                let s = s.unwrap_as_hashmap().unwrap();
                let stream_set_id: Hash40 = *s.get(&STREAM_SET_ID).unwrap().unwrap();
                let infos =
                    INFOS
                        .iter()
                        .map(|info|{
                            *s.get(info).unwrap().unwrap::<Hash40>()
                        })
                        .filter(|info| info != &hash40!(""))
                        .collect::<Vec<_>>();
                (stream_set_id, infos)
            })
            .collect::<HashMap<_,_>>();

    let name_to_file = db_root.into_iter()
        .filter_map(|bgm|{
            let bgm = bgm.unwrap_as_hashmap().unwrap();
            let stream_set_id: &Hash40 = bgm.get(&STREAM_SET_ID).unwrap().unwrap();
            let name_id: &String = bgm.get(&NAME_ID).unwrap().unwrap();
            if stream_set_id == &hash40!("") {
                return None;
            }
            let data_names =
                stream_set_id_to_infos.get(stream_set_id)
                    .unwrap()
                    .into_iter()
                    .map(|info|{
                        let stream_id = info_id_to_stream_id.get(info).unwrap();
                        stream_id_to_data_names.get(stream_id)
                            .unwrap()
                            .into_iter()
                            .cloned()
                    })
                    .flatten()
                    .collect::<Vec<_>>();
                    
            Some((
                clean_up_string(id_to_name.get(name_id)?),
                data_names,
            ))
        })
        .collect::<Vec<_>>();

    let mut csv_out_file = std::fs::File::create("song_name_to_file.csv").unwrap();
    let mut tsv_out_file = std::fs::File::create("song_name_to_file.tsv").unwrap();

    name_to_file.into_iter()
        .for_each(|(name, file)|{
            writeln!(csv_out_file, "{},{}", name, file.join(",")).unwrap();
            writeln!(tsv_out_file, "{}\t{}", name, file.join("\t")).unwrap();
        });
}
