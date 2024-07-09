use std::{
    fs::{read_dir, File},
    io::{Read, Result as IOResult, Write},
    path::Path,
};

use clap::Parser;

#[derive(Parser, Debug)]
struct IgnoreAggregatorArgs {
    #[arg(short, long)]
    reference_directory: String,

    #[arg(short, long)]
    output_aggregated: String,
}

fn check_args(args: &IgnoreAggregatorArgs) -> Result<(), String> {
    let reference_path = Path::new(&args.reference_directory);

    if !(reference_path.exists() && reference_path.is_dir()) {
        return Err(format!(
            "Invalid reference directory {}",
            args.reference_directory
        ));
    }

    let output_file = Path::new(&args.output_aggregated);
    if output_file.exists() {
        return Err(format!(
            "Invalid output file. {:?} already exists.",
            output_file
        ));
    }
    if let Some(output_parent) = output_file.parent() {
        if output_parent.to_str().unwrap() == "" {
            () // it's OK, don't check for existence
        } else if !(output_parent.exists()) {
            return Err(format!(
                "Invalid output path, couldn't find directory to create output file. ({:?} not found)",
                output_parent
            ));
        }
    }
    // else we are at root maybe
    Ok(())
}

fn list_git_ignore_files(reference_directory: &str) -> IOResult<Vec<String>> {
    let reference_path = Path::new(reference_directory);
    let mut to_explore = vec![reference_path.to_path_buf()];
    let mut git_ignore_file_paths = vec![];

    while let Some(path) = to_explore.pop() {
        let Ok(read_dir) = read_dir(path.clone()) else {
            println!("[Error] Failed reading {:?}", &path);
            continue;
        };
        for entry in read_dir {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    to_explore.push(entry_path);
                } else if entry_path.is_file()
                    && entry_path.file_name().expect(
                        "Expected to be able to extract filename while iterating in ReadDir",
                    ) == ".gitignore"
                {
                    // unwrap because the former condition already has an "expect" on `entry_path.file_name()`
                    git_ignore_file_paths.push(entry_path.to_str().unwrap().to_owned());
                }
            }
        }
    }

    Ok(git_ignore_file_paths)
}

fn re_reference_git_ignore_file(git_ignore_path: &str) -> IOResult<Vec<String>> {
    let git_ignore_path = Path::new(git_ignore_path);

    let git_ignore_parent = git_ignore_path.parent().unwrap();

    let mut git_file = File::open(git_ignore_path)?;
    let mut git_ignore_str = String::new();
    git_file.read_to_string(&mut git_ignore_str)?;

    let re_referenced_git_ignore_lines: Vec<String> = git_ignore_str
        .lines()
        .filter(|&line| !line.starts_with('#'))
        .map(|ignore| ignore.strip_prefix('/').unwrap_or(ignore))
        .map(|ignore| git_ignore_parent.join(Path::new(&ignore)))
        .map(|path| path.to_str().unwrap().to_owned())
        .collect();

    Ok(re_referenced_git_ignore_lines)
}

fn main() {
    let args = IgnoreAggregatorArgs::parse();
    match check_args(&args) {
        Ok(()) => (),
        Err(msg) => {
            println!("[Error] {msg}");
            return;
        }
    }
    println!("Scanning for .gitignore files");
    match list_git_ignore_files(&args.reference_directory) {
        Ok(git_ignore_file_paths) => {
            let mut output_file = match File::options()
                .create_new(true)
                .write(true)
                .open(args.output_aggregated)
            {
                Ok(file) => file,
                Err(err) => {
                    println!("[Error] {err:?}");
                    return;
                }
            };
            println!("Found {} git ignore files", git_ignore_file_paths.len());
            for path in git_ignore_file_paths {
                println!("{path:?}");
                // write gitignore path for reference
                output_file.write_all(b"# ").unwrap();
                output_file.write_all(path.as_bytes()).unwrap();
                output_file.write(b"\n").unwrap();
                let re_referenced = re_reference_git_ignore_file(&path).unwrap();
                for ignore in re_referenced {
                    output_file.write_all(ignore.as_bytes()).unwrap();
                    output_file.write(b"\n").unwrap();
                }
            }
        }
        Err(err) => {
            println!("[Error] {err:?}");
            return;
        }
    }

    // println!("Args: {args:?}");
}
