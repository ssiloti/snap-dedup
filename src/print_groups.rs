use std::collections::HashMap;
use file_group::FileGroup;

pub fn print_size_groups(groups: &HashMap<u64, FileGroup>)
{
	
	for (size, ref files) in groups
	{
		println!("Size {}", size);
		for d in &files.dest
		{
			println!("   Dest: {}", d.display());
		}
		for s in &files.src
		{
			println!("   Src: {}", s.display());
		}
	}
}
