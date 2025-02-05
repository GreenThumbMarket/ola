use std::process::Command;

pub fn structure_reasoning(goals: &str, return_type: &str, warnings: &str) -> String {
    let input_data = format!("Goals: {}\nReturn Type: {}\nWarnings: {}", goals, return_type, warnings);
    let output = Command::new("curl")
        .args(&[
            "-X", "POST",
            "http://localhost:11411",
            "-d", &input_data,
        ])
        .output()
        .expect("Failed to execute curl");
    println!("Output: {}", String::from_utf8_lossy(&output.stdout));
    String::from_utf8_lossy(&output.stdout).to_string()
}