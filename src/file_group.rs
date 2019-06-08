use std::path::PathBuf;

// grouping of destination and source files
pub struct FileGroup
{
	pub dest: Vec<PathBuf>,
	pub src: Vec<PathBuf>,
}

impl FileGroup
{
	pub fn new() -> FileGroup
	{
		FileGroup {
			dest: Vec::new(),
			src: Vec::new(),
		}
	}
}
