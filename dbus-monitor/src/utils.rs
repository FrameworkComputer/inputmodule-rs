use std::process::Child;
use std::process::Command;
use std::time::Duration;
use std::time::Instant;

pub fn run_command_with_timeout(
    command: &str,
    timeout_seconds: u64,
) -> Result<Child, Box<dyn std::error::Error>> {
    let mut child_process = Command::new("bash").arg("-c").arg(command).spawn()?;

    let start_time = Instant::now();
    while start_time.elapsed() < Duration::from_secs(timeout_seconds) {
        if let Some(exit_status) = child_process.try_wait()? {
            println!(
                "Command finished before the specified duration. Exit status: {:?}",
                exit_status
            );
            return Ok(child_process);
        }
    }

    child_process.kill()?; // Attempt to kill the process

    println!("Command terminated after {} seconds", timeout_seconds);
    Ok(child_process)
}
