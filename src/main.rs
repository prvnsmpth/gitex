use clap::{Parser, Subcommand};
use colored::Colorize;
use git2::{Branch, BranchType, Oid, Repository};
use std::{collections::HashMap, error::Error};

/// gx - git xtended
#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create and manage stacked PRs and commits
    Stack {
        #[command(subcommand)]
        command: StackCommands,
    },
}

#[derive(Subcommand, Debug)]
enum StackCommands {
    /// List all commits in the current stack
    List,
}

fn get_local_branches(repo: &Repository) -> Result<HashMap<Oid, Branch>, Box<dyn Error>> {
    let mut branches = HashMap::new();
    for branch in repo.branches(Some(BranchType::Local))? {
        let (branch, _) = branch?;
        let maybe_oid = branch.get().target();
        match maybe_oid {
            Some(oid) => {
                branches.insert(oid, branch);
            }
            None => {
                let branch_name = branch.name()?.unwrap_or("<unknown branch>");
                println!("Error: Branch {branch_name} has no target.");
            }
        }
    }
    Ok(branches)
}

fn list_stack(repo: &Repository) -> Result<(), Box<dyn Error>> {
    let head = repo.head()?;
    if !head.is_branch() {
        println!("Error: HEAD is not currently pointing to a local branch. Switch to a local branch to list the stack.");
        return Ok(());
    }

    let local_branches = get_local_branches(repo)?;

    let mut curr = head.peel_to_commit();
    let mut num_commits = 0;
    while let Ok(commit) = curr {
        let commit_id = commit.id();
        let commit_hash = &commit_id.to_string()[0..7];

        let commit_desc = commit.summary().unwrap_or("<no summary>");
        let commit_time = commit.time().seconds().to_string();
        let commit_author = commit.author().name().unwrap_or("Unknown").bold();

        let commit_branch =  local_branches
            .get(&commit_id)
            .and_then(|branch| branch.name().ok().flatten());
        
        let fmt_commit_hash = commit_hash.red().bold();
        let fmt_commit_desc = commit_desc.bold();
        let fmt_commit_time = format!("({})", commit_time).green().bold();
        let fmt_commit_author = format!("<{}>", commit_author).blue().bold();
        
        match commit_branch {
            Some(branch) => {
                println!(
                    "* {} - {} {} {} {}",
                    fmt_commit_hash,
                    format!("({})", branch).yellow().bold(),
                    fmt_commit_desc,
                    fmt_commit_time,
                    fmt_commit_author,
                );
            }
            None => {
                println!(
                    "* {} - {} {} {}",
                    fmt_commit_hash,
                    fmt_commit_desc,
                    fmt_commit_time,
                    fmt_commit_author,
                );
            }
        }
        
        num_commits += 1;
        if num_commits == 10 {
            break;
        }

        if commit.parent_count() > 1 {
            println!("Error: Commit {commit_hash} has more than one parent. Stacked PRs are not supported.");
            return Ok(());
        }

        curr = commit.parent(0);
    }

    for branch in repo.branches(Some(BranchType::Local))? {
        let (branch, _) = branch?;

        let branch_name = match branch.name() {
            Ok(Some(name)) => Some(name),
            Ok(None) => {
                println!("Found a branch with no name.");
                None
            }
            Err(e) => {
                println!("Error: {:?}", e);
                None
            }
        };

        let upstream = match branch.upstream() {
            Ok(u) => Some(u),
            Err(e) => {
                println!("Error: {:?}", e);
                None
            }
        };

        let upstream_name = upstream.and_then(|u| match u.name() {
            Ok(Some(name)) => Some(name.to_string()),
            Ok(None) => {
                println!("Found an upstream branch with no name.");
                None
            }
            Err(e) => {
                println!("Error: {:?}", e);
                None
            }
        });

        match (branch_name, upstream_name) {
            (Some(b), Some(u)) => {
                println!(
                    "\u{25c9}  branch: {}, upstream: {}",
                    b.blue().bold(),
                    u.green().bold()
                );
                println!("\u{ff5c}");
                println!("\u{ff5c}");
                println!("\u{ff5c}");
                println!(
                    "\u{25cb}  branch: {}, upstream: {}",
                    b.blue().bold(),
                    u.green().bold()
                );
                println!("\u{ff5c}");
                println!("\u{ff5c}");
                println!("\u{ff5c}");
                println!(
                    "\u{29bf}  branch: {}, upstream: {}",
                    b.blue().bold(),
                    u.green().bold()
                );
            }
            _ => {
                println!("Skipping branch with no name.");
                continue;
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), git2::Error> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Stack { command } => {
            let repo = match Repository::open(".") {
                Ok(r) => r,
                Err(e) => {
                    if e.code() == git2::ErrorCode::NotFound {
                        println!("Error: Not a git repository.");
                        return Ok(());
                    } else {
                        println!("Error: {:?}", e);
                        return Ok(());
                    }
                }
            };
            match command {
                StackCommands::List => {
                    let res = list_stack(&repo);
                    match res {
                        Ok(_) => {}
                        Err(e) => println!("Error: {:?}", e),
                    }
                }
            }
        }
    }

    Ok(())
}
