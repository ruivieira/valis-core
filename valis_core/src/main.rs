use clap::{App, Arg, SubCommand};
use dirs;

use valis_core::modules::admin::authinfo;
use valis_core::modules::projects::git::github;

fn main() {
    let matches = App::new("my_app")
        .subcommand(
            SubCommand::with_name("projects").subcommand(
                SubCommand::with_name("github")
                    .subcommand(
                        SubCommand::with_name("get-milestones")
                            .arg(Arg::with_name("ORG").required(true))
                            .arg(Arg::with_name("REPO").required(true)),
                    )
                    .subcommand(
                        SubCommand::with_name("list-milestone-issues")
                            .arg(Arg::with_name("ORG").required(true))
                            .arg(Arg::with_name("REPO").required(true))
                            .arg(Arg::with_name("MILESTONE").required(true)),
                    ),
            ),
        )
        .get_matches();

    if let Some(projects) = matches.subcommand_matches("projects") {
        if let Some(github) = projects.subcommand_matches("github") {
            let mut authinfo_file = dirs::home_dir().unwrap();
            authinfo_file.push(".authinfo");
            let authinfos = authinfo::read_auth_file(&authinfo_file).unwrap();
            let github_auth = authinfo::find_auth_info_for_machine("api.github.com", authinfos)
                .get(0)
                .cloned()
                .unwrap();
            let user = github_auth.login.name.to_owned();
            let token = github_auth.password.to_owned();

            if let Some(get_milestones) = github.subcommand_matches("get-milestones") {
                let org = get_milestones.value_of("ORG").unwrap();
                let repo = get_milestones.value_of("REPO").unwrap();

                let milestones = github::github_get_milestones(
                    user.to_owned(),
                    token.to_owned(),
                    org.to_string(),
                    repo.to_string(),
                )
                .unwrap();
                println!("{:?}", milestones);
            }

            if let Some(list_milestone_issues) = github.subcommand_matches("list-milestone-issues")
            {
                let org = list_milestone_issues.value_of("ORG").unwrap();
                let repo = list_milestone_issues.value_of("REPO").unwrap();
                let milestone_number: i32 = list_milestone_issues
                    .value_of("MILESTONE")
                    .unwrap()
                    .parse()
                    .expect("MILESTONE must be a number");

                let issues = github::github_get_milestone_issues(
                    user.to_owned(),
                    token.to_owned(),
                    org.to_string(),
                    repo.to_string(),
                    milestone_number,
                )
                .unwrap();
                println!("{:?}", issues);
            }
        }
    }
}
