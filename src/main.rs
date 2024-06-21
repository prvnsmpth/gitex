use git2::{BranchType, Repository};
use clap::{Parser, Subcommand};
use colored::Colorize;

/// gx - git xtended
#[derive(Parser, Debug)]
struct Cli {
   #[command(subcommand)]
   command: Commands
}

#[derive(Subcommand, Debug)]
enum Commands {
   /// Create and manage stacked PRs and commits
   Stack {
       #[command(subcommand)]
       command: StackCommands
   } 
}

#[derive(Subcommand, Debug)]
enum StackCommands {
    /// List all commits in the current stack
    List,
}

fn list_stack(repo: &Repository) -> Result<(), git2::Error> {
    let head = repo.head()?;
    if !head.is_branch() {
        println!("Error: HEAD is not currently pointing to a local branch. Switch to a local branch to list the stack.");
        return Ok(());
    }
    
    let mut curr = head.peel_to_commit();
    while let Ok(commit) = curr {
        let commit_hash = String::from(commit.id().as_bytes());
        println!("* {} - {} ({}) <{}>", commit.id().as_bytes());
        println!("Author: {}", commit.author().name().unwrap_or("Unknown"));
        println!("Date: {}", commit.time().seconds());
        println!("\n    {}", commit.summary().unwrap_or("No message"));
        println!("\n");
        
        if commit.parent_count() > 1 {
            let hash = commit.id();
            println!("Error: Commit {hash} has more than one parent. Stacked PRs are not supported.");
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
            },
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
        
        let upstream_name = upstream.and_then(|u| {
            match u.name() {
                Ok(Some(name)) => Some(name.to_string()),
                Ok(None) => {
                    println!("Found an upstream branch with no name.");
                    None
                },
                Err(e) => {
                    println!("Error: {:?}", e);
                    None
                }
            }
        });
        
        match (branch_name, upstream_name) {
            (Some(b), Some(u)) => {
                println!("\u{25c9}  branch: {}, upstream: {}", b.blue().bold(), u.green().bold());
                println!("\u{ff5c}");
                println!("\u{ff5c}");
                println!("\u{ff5c}");
                println!("\u{25cb}  branch: {}, upstream: {}", b.blue().bold(), u.green().bold());
                println!("\u{ff5c}");
                println!("\u{ff5c}");
                println!("\u{ff5c}");
                println!("\u{29bf}  branch: {}, upstream: {}", b.blue().bold(), u.green().bold());
            },
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
                        Ok(_) => {},
                        Err(e) => println!("Error: {:?}", e)
                    }
                }
            }
        }
    }
    
    Ok(())
}
