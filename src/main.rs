use glob::glob;
use std::io::{self, BufRead};
use std::process::{exit, Command, Stdio};
use tempfile::tempdir;

fn main() {
    // Create a temporary directory
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let temp_outfile = temp_dir.path().join("out.mp3");

    let pwd_output = Command::new("pwd")
        .output()
        .expect("Failed to execute command");

    let pwd_str = String::from_utf8_lossy(&pwd_output.stdout);
    let pwd_parts: Vec<&str> = pwd_str.trim().split('/').collect();

    let album_name = pwd_parts[pwd_parts.len() - 2]
        .replace(&['[', ']', '(', ')', '@', '&', '!', '#'][..], "")
        .replace(char::is_whitespace, "");

    let album_number = pwd_parts.last().unwrap().replace(char::is_whitespace, "");

    println!("album name: {}", album_name);
    println!("album number: {}", album_number);

    // Use glob to get a list of files matching the pattern
    let list: Vec<_> = glob("*.mp3")
        .expect("Failed to read glob pattern")
        .filter_map(Result::ok)
        .map(|entry| entry.display().to_string())
        .collect();

    let outfile = format!("/tmp/{}{}.mp3", album_name, album_number);
    println!("will generate {}", outfile);

    let mut continue_input = String::new();
    while continue_input.to_lowercase() != "y" {
        println!("continue (y): ");
        continue_input.clear();
        io::stdin()
            .read_line(&mut continue_input)
            .expect("Failed to read line");
        continue_input = continue_input.trim().to_string();
    }

    // Execute the cat command in a shell with proper escaping
    let escaped_list_str = shlex::join(list.iter().map(|file| file.as_str()));
    let cat_command = format!("cat {} >> {}", escaped_list_str, temp_outfile.display());
    println!("{}", cat_command);

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

    println!("Output file created at: {}", outfile);
}
