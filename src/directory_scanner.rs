use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use file_group::FileGroup;

// returns a map of file sizes to paths
// the first root_dir is the destination dir
// files from the subsequent root_dirs will only be included if they match
// the size of at least one file in the destination dir
pub fn scan_directories(root_dirs: &[PathBuf], min_size: u64) -> HashMap<u64, FileGroup>
{
	let mut size_groups = HashMap::new();
	scan_directory(&root_dirs[0], true, min_size, &mut size_groups);
	
	for d in root_dirs[1..].iter() {
		scan_directory(&d, false, min_size, &mut size_groups);
	}
	
	return size_groups;
}

fn scan_directory(dir: &Path, dest: bool, min_size: u64, groups: &mut HashMap<u64, FileGroup>)
{
	for entry in match fs::read_dir(dir) { Ok(d) => d, Err(_) => { return; } } {
		let entry = match entry { Ok(e) => e, Err(_) => { continue; } };
		let etype = match entry.file_type() { Ok(t) => t, Err(_) => { continue; } };
		if etype.is_dir() {
			scan_directory(&entry.path(), dest, min_size, groups);
			continue;
		}
		
		if !etype.is_file() { continue; }
		
		let md = match entry.metadata() { Ok(md) => md, Err(_) => { continue; } };
		if md.len() < min_size { continue; }
		
		if dest {
			let group = groups.entry(md.len()).or_insert(FileGroup::new());
			group.dest.push(entry.path());
		}
		else {
			let group = match groups.get_mut(&md.len()) { Some(g) => g, None => { continue; } };
			group.src.push(entry.path());
		}
	}
}
