use std::{collections, io::BufRead};
use std::collections::HashMap;
use crate::{error::CmdError, gfa};
use log;
#[derive(Debug, Default, Clone, Copy)]
struct RemoveRegion {
    start: usize,
    end: usize,
}
fn filter_elements_by_cumulative_sum(a: &[i32], intervals: &[(i32, i32)]) -> Vec<i32> {
    let mut cumsum = 0;
    let mut keep = vec![1; a.len()];  // 初始化为1，表示初始时保留所有元素
    let mut interval_index = 0; // 当前处理的区间索引

    // 通过一次遍历处理累计和和区间检查
    for (i, &value) in a.iter().enumerate() {
        cumsum += value;

        // 更新当前区间到合适的位置
        while interval_index < intervals.len() && cumsum > intervals[interval_index].1 {
            interval_index += 1;
        }

        // 检查当前累计和是否处于当前区间内
        if interval_index < intervals.len() && cumsum >= intervals[interval_index].0 {
            keep[i] = 0;  // 标记当前元素及之前元素为不保留
            let start_cumsum = cumsum - value; // 计算进入区间前的累计和
            for j in (0..=i).rev() {
                keep[j] = 0;
                cumsum -= a[j];
                if cumsum <= start_cumsum {
                    break;
                }
            }

        }
    }

    // 根据keep数组过滤原始数组
    a.iter()
        .enumerate()
        .filter(|&(i, _)| keep[i] == 1)
        .map(|(_, &val)| val)
        .collect()
}


pub fn prune_gfa(gfa: String, output: String) -> Result<(), CmdError> {
    let bed = std::fs::File::open("test.bed").map_err(CmdError::FileOpenError)?;
    let bed = std::io::BufReader::new(bed);
    let mut removes: HashMap<String, Vec<RemoveRegion>> = HashMap::new();
    for line in bed.lines() {
        let line = line.map_err(CmdError::LineReadError)?;
        let tem = line.split_whitespace().collect::<Vec<&str>>();
        let start = tem[1].parse::<usize>().map_err(|_|CmdError::ParseError)?;
        let end = tem[2].parse::<usize>().map_err(|_|CmdError::ParseError)?;
        let remove_region = RemoveRegion { start, end };
        removes.entry(tem[0].to_owned())
            .or_insert(Vec::new())
            .push(remove_region);
    }


    let gfa_parser = gfa::GFAParserBuilder::new()
        .get_walks(true)
        .get_segments(true)
        .build();
    let gfa_obj = gfa_parser.parse_file(gfa)?;
    log::debug!("GFA file parsed successfully");
    let walk_num = gfa_obj.walks.len();
    log::debug!("Total number of walks: {}", walk_num);
    let segment_map = gfa_obj.get_segment_len();
    for  walk in gfa_obj.walks {
        let merge_name = format!("{}#{}#{}", &walk.sample, &walk.haptype, &walk.chroms);
        let mut remove_regions = removes.get(&merge_name);
        let a = walk.unit;
        match remove_regions {
            Some(regions) => {
            for region in regions {
                if walk.ranges.start < region.start && walk.ranges.end > region.end {
                    for i in 0..walk.unit {

                }
                else {
                    log::debug!("No remove regions found for {}", merge_name);
                }
            }
        }
            None => {
                log::debug!("No  path found for {}", merge_name);
            }

    }
    }
    let mut i: usize = 0;
    let mut j: usize = 0;

    Ok(())
}
