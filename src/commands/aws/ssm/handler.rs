use aws_sdk_ssm::Client;
use nix::sys::signal::{SigHandler, Signal, kill, signal as nix_signal};
use nix::unistd::Pid;
use serde_json::json;
use std::process::{Command, Stdio};
use tokio::signal::unix::{SignalKind, signal as tokio_signal};

use crate::commands::Output;
use crate::commands::aws::{
    arg::GlobalOptions,
    error::{
        handle_construction_failure, handle_dispatch_failure, handle_response_error,
        handle_service_error, handle_timeout_error, handle_unknown_error,
    },
    ssm::arg::{AWSSSMCommand, SSMSubCommand, StartArg},
};

impl AWSSSMCommand {
    pub async fn execute(self, opts: GlobalOptions) -> Result<Output, Box<dyn std::error::Error>> {
        let client = Client::new(&opts.sdk_config);
        match self.command {
            SSMSubCommand::Start(args) => args.execute(&client, opts).await,
        }
    }
}

impl StartArg {
    async fn execute(
        self,
        client: &Client,
        opts: GlobalOptions,
    ) -> Result<Output, Box<dyn std::error::Error>> {
        let instance_id = &self.id;
        if instance_id.is_empty() {
            return Err(format!("Error: Invalid instance ARN.").into());
        }

        // Check if session-manager-plugin is installed
        if !is_plugin_installed() {
            return Err(format!("Error: session-manager-plugin is not installed.").into());
        }

        // Start the session
        match client
            .start_session()
            .set_target(Some(instance_id.to_string()))
            .send()
            .await
        {
            Ok(response) => {
                let session_id = response.session_id().unwrap_or_default();
                let token_value = response.token_value().unwrap_or_default();
                let stream_url = response.stream_url().unwrap_or_default();

                // Prepare parameters for session-manager-plugin
                let session_params = json!({
                    "SessionId": session_id,
                    "TokenValue": token_value,
                    "StreamUrl": stream_url,
                });

                let start_session_params = json!({
                    "Target": instance_id,
                });

                // Get region from SDK config
                let region = opts
                    .sdk_config
                    .region()
                    .map(|r| r.as_ref())
                    .unwrap_or("us-east-1");

                // Invoke session-manager-plugin
                let mut command = Command::new("session-manager-plugin");
                command
                    .arg(session_params.to_string())
                    .arg(region)
                    .arg("StartSession")
                    .arg("knife") // profile name (can be any identifier)
                    .arg(start_session_params.to_string())
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit());

                // Start the session-manager-plugin process
                let mut child = match command.spawn() {
                    Ok(child) => child,
                    Err(e) => {
                        return Err(format!("Failed to start interactive session: {}", e).into());
                    }
                };
                // The child inherits knife's process group, by default, all processes in the same
                // pg receive the same set of signals delivered from terminal
                // At this point, we ignore all possible terminate signal so knife is not exited before ssm.
                let old_int = unsafe { nix_signal(Signal::SIGINT, SigHandler::SigIgn) }?; // ctrl+c
                let old_quit = unsafe { nix_signal(Signal::SIGQUIT, SigHandler::SigIgn) }?; // ctrl+\
                let old_tstp = unsafe { nix_signal(Signal::SIGTSTP, SigHandler::SigIgn) }?; // ctrl+z
                let old_cont = unsafe { nix_signal(Signal::SIGCONT, SigHandler::SigIgn) }?; // fg

                let child_pid = Pid::from_raw(child.id() as i32);

                // SIGTERM/SIGHUP may be sent directly to knife's pid only, not to the
                // whole pg, so forward them to our child explicitly.
                let sig_task = tokio::spawn(async move {
                    let mut sigterm = tokio_signal(SignalKind::terminate()).unwrap();
                    let mut sighup = tokio_signal(SignalKind::hangup()).unwrap();
                    tokio::select! {
                        _ = sigterm.recv() => {
                            let _ = kill(child_pid, Signal::SIGTERM);
                        }
                        _ = sighup.recv() => {
                            let _ = kill(child_pid, Signal::SIGHUP);
                        }
                    }
                });

                let result = child.wait();

                // clean our background sig task
                sig_task.abort();

                // recover the signal handling of knife
                unsafe {
                    let _ = nix_signal(Signal::SIGINT, old_int);
                    let _ = nix_signal(Signal::SIGQUIT, old_quit);
                    let _ = nix_signal(Signal::SIGTSTP, old_tstp);
                    let _ = nix_signal(Signal::SIGCONT, old_cont);
                }

                // Wait for the child process to complete
                let exit_status = result?;
                if !exit_status.success() {
                    return Err(format!("Session ended with code: {}", exit_status).into());
                }
                let output = Output::new(opts.verbose);
                output.stderr("Session ended");
                Ok(output)
            }
            Err(err) => {
                use aws_sdk_ssm::error::SdkError;
                let error_msg = match err {
                    SdkError::ServiceError(service_err) => {
                        handle_service_error(service_err.err(), "SSM session")
                    }
                    SdkError::DispatchFailure(dispatch_err) => {
                        handle_dispatch_failure(&dispatch_err, "SSM session")
                    }
                    SdkError::ConstructionFailure(err) => {
                        handle_construction_failure(&err, "SSM session", false)
                    }
                    SdkError::TimeoutError(err) => handle_timeout_error(&err, "SSM session", false),
                    SdkError::ResponseError(err) => {
                        handle_response_error(&err, "SSM session", false)
                    }
                    _ => handle_unknown_error(&err, "SSM session", false),
                };
                Err(error_msg.into())
            }
        }
    }
}

/// Check if session-manager-plugin is installed
fn is_plugin_installed() -> bool {
    Command::new("which")
        .arg("session-manager-plugin")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
