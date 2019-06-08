mod directory_scanner;
mod extent_scanner;
mod file_group;
mod print_groups;

extern crate btrfs;

use directory_scanner::*;
use extent_scanner::*;
use print_groups::*;

use std::env;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};

// copied from btrfs because the original is retarded
// it tries to open the source for writing
// it only issues a single dedup ioctl which will typically only dedup the first 16MiB of the file
extern crate libc;
use std::fs;
use std::error::Error;

pub fn deduplicate_files_with_source <
	AsPath1: AsRef <Path>,
	AsPath2: AsRef <Path>,
> (
	source_filename: AsPath1,
	dest_filenames: & [AsPath2],
) -> Result <(), String> {

	let source_filename = source_filename.as_ref();

	// nothing to do unless there is are no dest filenames

	if dest_filenames.is_empty () {
		return Ok (());
	}

	// open files

	let source_file_metadata =
		try! (

		fs::metadata (
			source_filename,
		).map_err (
			|io_error|

			format! (
				"Error getting metadata for {:?}: {}",
				source_filename,
				io_error.description ())

		)

	);

	let source_file_descriptor =
		try! (

		btrfs::FileDescriptor::open (
			source_filename,
			libc::O_RDONLY,
		).map_err (
			|error|

			format! (
				"Error opening file: {:?}: {}",
				source_filename,
				error)

		)

	);

	let mut target_file_descriptors: Vec <btrfs::FileDescriptor> =
		Vec::new ();

	for dest_filename in dest_filenames {

		let dest_filename =
			dest_filename.as_ref ();

		let target_file_descriptor =
			try! (

			btrfs::FileDescriptor::open (
				dest_filename,
				libc::O_RDWR,
			).map_err (
				|error|

				format! (
					"Error opening file: {:?}: {}",
					dest_filename,
					error)

			)

		);

		target_file_descriptors.push (
			target_file_descriptor);

	}

	// create data structures

	let mut dedupe_range =
		btrfs::DedupeRange {

		src_offset: 0,

		src_length:
			source_file_metadata.len (),

		dest_infos:
			target_file_descriptors.iter ().map (
				|target_file_descriptor|

			btrfs::DedupeRangeDestInfo {
				dest_fd: target_file_descriptor.get_value () as i64,
				dest_offset: 0,
				bytes_deduped: 0,
				status: btrfs::DedupeRangeStatus::Same,
			}

		).collect (),

	};

	// perform dedupe

	while dedupe_range.src_offset < source_file_metadata.len()
	{
		try! (btrfs::deduplicate_range(source_file_descriptor.get_value(), &mut dedupe_range));
		dedupe_range.dest_infos.retain(|x| x.status == btrfs::DedupeRangeStatus::Same);
		if dedupe_range.dest_infos.is_empty() { break; }
		dedupe_range.src_offset += dedupe_range.dest_infos[0].bytes_deduped;
		dedupe_range.src_length -= dedupe_range.dest_infos[0].bytes_deduped;
		for r in &mut dedupe_range.dest_infos
		{ r.dest_offset = dedupe_range.src_offset; }
		
	}
	

	// process result

	// TODO

	Ok (())

}

fn main() {
    let paths = Vec::from_iter(env::args_os().skip(1).map(|x| PathBuf::from(x)));
	if paths.len() < 2 {
		println!("Usage: snap-dedup primary snapshot [snapshot...]");
		return;
	}
	
	let size_groups = scan_directories(&paths, 1024*1024*4);
	//print_size_groups(&size_groups);
	let content_groups = size_groups.into_iter()
		.map(|x| x.1)
		.filter(|x| x.dest.len() + x.src.len() > 1)
		.map(|x| scan_extents(x))
		.filter(|x| x.dest.len() + x.src.len() > 1);
	
	for g in content_groups
	{
	/*
		for d in &g.dest
		{
			println!("   Dest: {}", d.display());
		}
		println!(" ");
		for s in &g.src
		{
			println!("   Src: {}", s.display());
		}
		println!("==================================================");
		*/
		for s in &g.src
		{
			for d in &g.dest
			{
				match deduplicate_files_with_source(s, &[d])
				{
					Ok(_) => println!("Dedup successful {} -> {}", s.display(), d.display()),
					Err(s) => println!("Dedup failed: {}", s),
				};
			}
		}
	}
}
