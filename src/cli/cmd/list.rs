use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use lazy_static::lazy_static;
use prettytable::{
    format::{FormatBuilder, LinePosition, LineSeparator, TableFormat},
    row, Attr, Cell, Row, Table,
};
use serde::Deserialize;
use std::process::ExitCode;

use crate::cli::cmd::FlakeHubClient;

use super::CommandExecute;

/// Lists key FlakeHub resources.
#[derive(Parser)]
pub(crate) struct ListSubcommand {
    #[command(subcommand)]
    cmd: Subcommands,

    #[clap(from_global)]
    host: String,

    #[clap(from_global)]
    backend_host: String,
}

#[derive(Subcommand)]
enum Subcommands {
    /// Lists all currently public flakes on FlakeHub.
    Flakes,
    /// List all currently public organizations on FlakeHub.
    Orgs,
}

lazy_static! {
    static ref TABLE_FORMAT: TableFormat = FormatBuilder::new()
        .borders('|')
        .padding(1, 1)
        .separators(
            &[LinePosition::Top, LinePosition::Title, LinePosition::Bottom],
            LineSeparator::new('-', '+', '+', '+'),
        )
        .build();
}

#[async_trait::async_trait]
impl CommandExecute for ListSubcommand {
    async fn execute(self) -> color_eyre::Result<ExitCode> {
        use Subcommands::*;

        let client = FlakeHubClient::new(&self.backend_host)?;

        match self.cmd {
            Flakes => {
                let pb = ProgressBar::new_spinner();
                pb.set_style(ProgressStyle::default_spinner());
                match client.flakes().await {
                    Ok(flakes) => {
                        if flakes.is_empty() {
                            println!("No results");
                        } else {
                            let mut table = Table::new();
                            table.set_format(*TABLE_FORMAT);
                            table.set_titles(row!["Flake", "FlakeHub URL"]);

                            for flake in flakes {
                                table.add_row(Row::new(vec![
                                    Cell::new(&flake.name()).with_style(Attr::Bold),
                                    Cell::new(&flake.url(&self.host)).with_style(Attr::Dim),
                                ]));
                            }

                            table.printstd();
                        }
                    }
                    Err(e) => {
                        println!("Error: {e}");
                    }
                }
            }
            Orgs => {
                let pb = ProgressBar::new_spinner();
                pb.set_style(ProgressStyle::default_spinner());
                match client.orgs().await {
                    Ok(orgs) => {
                        if orgs.is_empty() {
                            println!("No results");
                        } else {
                            let mut table = Table::new();
                            table.set_format(*TABLE_FORMAT);
                            table.set_titles(row!["Organization", "FlakeHub URL"]);

                            for org in orgs {
                                let url = format!("{}/org/{}", self.host, org);
                                table.add_row(Row::new(vec![
                                    Cell::new(&org).with_style(Attr::Bold),
                                    Cell::new(&url).with_style(Attr::Dim),
                                ]));
                            }

                            table.printstd();
                        }
                    }
                    Err(e) => {
                        println!("Error: {e}");
                    }
                }
            }
        }

        Ok(ExitCode::SUCCESS)
    }
}

#[derive(Deserialize)]
pub(super) struct Flake {
    org: String,
    project: String,
}

impl Flake {
    fn name(&self) -> String {
        format!("{}/{}", self.org, self.project)
    }

    fn url(&self, host: &str) -> String {
        format!("{}/flake/{}/{}", host, self.org, self.project)
    }
}

#[derive(Deserialize)]
pub(super) struct Org {
    pub(super) name: String,
}
