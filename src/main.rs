use colored::*;
use dialoguer::Input;
use glob::glob;
use id3::Tag;
use id3::TagLike;
use std::env;
use std::io::{self, BufRead};
use std::process::{exit, Command, Stdio};
use tempfile::tempdir;

fn main() {
    println!(
        "{}",
        "[!] tool assumes files in the format of /path/to/root/Artist - Album/DiscName/*.mp3"
            .red()
            .bold()
    );
    println!(
        "{}",
        "[!] some of these assumptions are not optional!"
            .red()
            .bold()
    );
    println!("{}", "[!] DiscName is often CD1, CD2".red());

    let args: Vec<String> = env::args().collect();
    let one_arg = args.iter().any(|arg| arg == "--one");

    if one_arg {
        println!("{}", "[+] --one passed, assuming one disc / one folder and flattening directory path by one...".magenta().bold());
    }

    // Create a temporary directory
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let temp_outfile = temp_dir.path().join("out.mp3");

    let pwd_output = Command::new("pwd")
        .output()
        .expect("Failed to execute command");

    let pwd_str = String::from_utf8_lossy(&pwd_output.stdout);
    let pwd_parts: Vec<&str> = pwd_str.trim().split('/').collect();

    // extract some data from pwd
    let named_dir: &str;

    if one_arg {
        named_dir = pwd_parts.last().unwrap();
    } else {
        named_dir = pwd_parts[pwd_parts.len() - 2];
    }
    let parts: Vec<&str> = named_dir.split('-').collect();
    let dir_artist = parts.get(0).map_or("", |s| s.trim()); // map_or() to chomp extra
    let dir_album = parts.get(1).map_or("", |s| s.trim()); // space characters
    let dir_disc = pwd_parts.last().unwrap().replace(char::is_whitespace, "");
    println!(
        "{}",
        "[?] guesses based on Directory Structure...".cyan().bold()
    );
    println!("{}", format!("[-] artist:\t\t{}", dir_artist).cyan().bold());
    println!("{}", format!("[-] album:\t\t{}", dir_album).cyan().bold());

    // Use glob to get a list of files matching the pattern
    let list: Vec<_> = glob("*.mp3")
        .expect("Failed to read glob pattern")
        .filter_map(Result::ok)
        .map(|entry| entry.display().to_string())
        .collect();

    if let Some(first_file) = list.first() {
        if let Ok(tag) = Tag::read_from_path(first_file) {
            println!(
                "{}",
                "[?] guesses based on First File ID3 Tags...".green().bold()
            );
            println!(
                "{}",
                format!("[-] Album:\t\t{:?}", tag.album()).green().bold()
            );
            println!(
                "{}",
                format!("[-] Album Artist:\t{:?}", tag.album_artist())
                    .green()
                    .bold()
            );
            println!(
                "{}",
                format!("[-] Artist:\t\t{:?}", tag.artist()).green().bold()
            );
            println!(
                "{}",
                format!("[-] Title (album?):\t{:?}", tag.title()).green()
            );
        } else {
            println!("Failed to read ID3 tags from the file");
        }
    } else {
        println!("No files found");
    }

    // we just manually enter all the data for now...
    let entered_artist = Input::<String>::new()
        .with_prompt("Enter Artist:")
        .interact()
        .expect("Failed to read input");

    let entered_album_name = Input::<String>::new()
        .with_prompt("Enter Album Name:")
        .interact()
        .expect("Failed to read input");

    println!("Artist: {}", entered_artist);
    println!("Album Name: {}", entered_album_name);

    let outfile: String;
    if one_arg {
        outfile = format!("/tmp/{}.mp3", entered_album_name);
    } else {
        outfile = format!("/tmp/{}, {}.mp3", entered_album_name, dir_disc);
    }
    println!(
        "{}",
        format!("[!] will generate => {}", outfile).red().bold()
    );

    let escaped_list_str = shlex::join(list.iter().map(|file| file.as_str()));
    let cat_command = format!("cat {} >> {}", escaped_list_str, temp_outfile.display());
    println!("{}", format!("[.] {}", cat_command).yellow());

    let mut continue_input = String::new();
    while continue_input.trim().to_lowercase() != "y" {
        continue_input = Input::<String>::new()
            .with_prompt("Continue (y)")
            .interact()
            .expect("Failed to read input");
    }

    // Execute the cat command in a shell with proper escaping
    let cat_output = Command::new("sh")
        .arg("-c")
        .arg(&cat_command)
        .output()
        .expect("Failed to execute command");
    println!(
        "cat command output: {}",
        String::from_utf8_lossy(&cat_output.stdout)
    );

    let mut ffmpeg_command = Command::new("ffmpeg")
        .arg("-i")
        .arg(temp_outfile)
        .arg("-acodec")
        .arg("copy")
        .arg(&outfile)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start ffmpeg");

    if let Some(stdout) = ffmpeg_command.stdout.take() {
        let reader = io::BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                println!("ffmpeg output: {}", line);
            }
        }
    }

    let status = ffmpeg_command.wait().expect("Failed to wait for ffmpeg");
    if !status.success() {
        eprintln!("Error: ffmpeg command failed");
        exit(1);
    }

    println!(
        "{}",
        format!("[!] Output file created at: {}", outfile)
            .green()
            .bold()
    );

    // Open the temporary output file with ID3 tag support
    let mut tag = Tag::read_from_path(&outfile).expect("Failed to read ID3 tags from output file");

    println!(
        "{}",
        "[+] writing the following ID3 Tags to Output File..."
            .yellow()
            .bold()
    );
    println!(
        "{}",
        format!("[-] Artist:\t\t{:?}", entered_artist)
            .yellow()
            .bold()
    );
    println!(
        "{}",
        format!("[-] Album Artist:\t{:?}", entered_artist)
            .yellow()
            .bold()
    );
    println!(
        "{}",
        format!("[-] Title:\t\t{:?}", entered_album_name)
            .yellow()
            .bold()
    );
    println!(
        "{}",
        format!("[-] Album:\t\t{:?}", entered_album_name)
            .yellow()
            .bold()
    );
    println!("{}", format!("[-] Genre:\t\tDJ Set").yellow().bold());

    // Modify the ID3 tags as needed
    tag.set_artist(entered_artist.clone());
    tag.set_album_artist(entered_artist.clone());
    tag.set_title(entered_album_name.clone());
    tag.set_album(entered_album_name.clone());
    tag.set_genre("DJ Set");

    // Write the modified ID3 tags back to the file
    tag.write_to_path(outfile.clone(), id3::Version::Id3v24)
        .expect("Failed to write modified ID3 tags to output file");
}
