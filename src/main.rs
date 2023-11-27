use std::env;
use std::process::{Command, exit};

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

    let list_output = Command::new("ls")
        .arg("*.mp3")
        .output()
        .expect("Failed to execute command");

    let list_str = String::from_utf8_lossy(&list_output.stdout);
    let list: Vec<&str> = list_str.trim().split('\n').collect();

    let outfile = format!("/tmp/{}{}.mp3", album_name, album_number);

    println!("album name: {}", album_name);
    println!("album number: {}", album_number);
    println!("will generate {}", outfile);

    let mut continue_input = String::new();
    while continue_input.to_lowercase() != "y" {
        println!("continue (y): ");
        continue_input.clear();
        std::io::stdin().read_line(&mut continue_input).expect("Failed to read line");
        continue_input = continue_input.trim().to_string();
    }

    Command::new("rm")
        .arg("/tmp/out.mp3")
        .output()
        .expect("Failed to execute command");

    for x in list {
        Command::new("cat")
            .arg(x)
            .arg(">>")
            .arg("/tmp/out.mp3")
            .output()
            .expect("Failed to execute command");
    }

    Command::new("ffmpeg")
        .arg("-i")
        .arg("/tmp/out.mp3")
        .arg("-acodec")
        .arg("copy")
        .arg(outfile)
        .output()
        .expect("Failed to execute command");
}
