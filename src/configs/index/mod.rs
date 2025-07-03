use crate::configs;
use crate::configs::{get_load_configs_from_etc, index, PATH_CONFIGS_JSON, PATH_ETC, PATH_WORKDIR};

mod default_index;
const PATH_INDEX_JSON: &str = "index/";
pub fn get_part_path_to_index() -> String{
    format!("{}{}",configs::get_part_path_to_configs(),PATH_INDEX_JSON)
}
fn get_path_index_json(file_name:String) -> String {
    let folder = index::get_part_path_to_index();
    let file_path = format!("{}{}", folder, file_name);
    file_path
}