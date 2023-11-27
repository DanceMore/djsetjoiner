use std::env;
use std::io::{self, BufRead};
use std::process::{Command, Stdio, exit};
use glob::glob;
use shlex::quote;

fn main() {
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
        io::stdin().read_line(&mut continue_input).expect("Failed to read line");
        continue_input = continue_input.trim().to_string();
    }

    let rm_output = Command::new("rm")
        .arg("/tmp/out.mp3")
        .stdout(Stdio::piped())  // Capture stdout of the child process
        .spawn()
        .expect("Failed to execute command");

    // Execute the cat command in a shell with proper escaping
    let escaped_list_str = shlex::join(list.iter().map(|file| file.as_str()));
    let cat_command = format!("cat {} >> /tmp/out.mp3", escaped_list_str);
    println!("{}", cat_command);

    let cat_output = Command::new("sh")
        .arg("-c")
        .arg(&cat_command)
        .output()
        .expect("Failed to execute command");


    let mut ffmpeg_command = Command::new("ffmpeg")
        .arg("-i")
        .arg("/tmp/out.mp3")
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
