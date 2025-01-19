use std::path::PathBuf;

use clap::Parser;
use cliclack::{
    input,
    log::{info, remark, step, success},
};
use email_address_parser::{EmailAddress, ParsingOptions};
use path_slash::PathBufExt as _;
use soldeer_core::{
    auth::{execute_login, Credentials},
    errors::AuthError,
    Result,
};

/// Log into the central repository to push packages
///
/// The credentials are saved by default into ~/.soldeer.
/// If you want to overwrite that location, use the SOLDEER_LOGIN_FILE env var.
#[derive(Debug, Clone, Default, Parser, bon::Builder)]
#[builder(on(String, into))]
#[clap(after_help = "For more information, read the README.md")]
#[non_exhaustive]
pub struct Login {
    /// Specify the email without prompting.
    #[arg(long)]
    pub email: Option<String>,

    /// Specify the password without prompting.
    #[arg(long)]
    pub password: Option<String>,
}

pub(crate) async fn login_command(cmd: Login) -> Result<()> {
    remark("If you do not have an account, please visit soldeer.xyz to create one.")?;

    let email: String = match cmd.email {
        Some(email) => {
            if EmailAddress::parse(&email, Some(ParsingOptions::default())).is_none() {
                return Err(AuthError::InvalidCredentials.into());
            }
            step(format!("Email: {email}"))?;
            email
        }
        None => input("Email address")
            .validate(|input: &String| {
                if input.is_empty() {
                    Err("Email is required")
                } else {
                    match EmailAddress::parse(input, Some(ParsingOptions::default())) {
                        None => Err("Invalid email address"),
                        Some(_) => Ok(()),
                    }
                }
            })
            .interact()?,
    };

    let password = match cmd.password {
        Some(pw) => pw,
        None => cliclack::password("Password").mask('▪').interact()?,
    };

    let token_path = execute_login(&Credentials { email, password }).await?;
    success("Login successful")?;
    info(format!(
        "Login details saved in: {}",
        PathBuf::from_slash_lossy(&token_path).to_string_lossy() /* normalize separators */
    ))?;
    Ok(())
}
