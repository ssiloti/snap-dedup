use btrfs;
use btrfs::FileExtent;
use file_group::FileGroup;
use std::collections::HashSet;
use std::path::PathBuf;

pub fn scan_extents(mut files: FileGroup) -> FileGroup
{
	let mut extent_set : HashSet<u64> = HashSet::new();
	
	let mut dup_check = |f: &PathBuf| {
		let extents = match btrfs::get_file_extent_map_for_path(f) { Ok(e) => e, Err(_) => { return true; } };
		extent_set.insert(extents[0].physical)
	};
	
	files.dest.retain(&mut dup_check);
	files.src.retain(&mut dup_check);
	files
}
