use rlua::{Context, Lua, Result, ToLua, UserData};
use rlua::Error as LuaError;
use rlua::FromLua;
use rlua::Table;
use uuid::Uuid;

use crate::modules::db;
use crate::modules::db::DatabaseOperations;
use crate::modules::db::serializers::SerializableDateTime;
use crate::modules::projects::agile;
use crate::modules::projects::agile::core::Project;

pub fn agile_create_project(ctx: &Context) {
    let f = ctx
        .create_function(|ctx, (name, description, path): (String, String, String)| {
            db::init_db(&path).ok().unwrap();
            let project = Project {
                name,
                description,
                ..Default::default()
            };
            project.save(&path).ok().unwrap();
            let project_table = ctx.create_table().ok().unwrap();
            project_table
                .set("id", project.id.to_string())
                .ok()
                .unwrap();
            project_table.set("name", project.name).ok().unwrap();
            project_table
                .set("description", project.description)
                .ok()
                .unwrap();
            project_table
                .set("created_at", project.created_at.to_string())
                .ok()
                .unwrap();
            project_table
                .set("update_at", project.updated_at.to_string())
                .ok()
                .unwrap();

            Ok(project_table)
        })
        .unwrap();
    ctx.globals()
        .set("agile_create_project", f)
        .unwrap();
}

pub fn agile_create_sprint(ctx: &Context) {
    let f = ctx
        .create_function(
            |ctx, (project_id, name, start_date, path): (String, String, String, String)| {
                db::init_db(&path).ok().unwrap();
                let sprint = agile::core::Sprint {
                    project_id: Uuid::parse_str(&project_id).unwrap(),
                    name,
                    start_date: SerializableDateTime::from_str(&start_date).unwrap(),
                    end_date: SerializableDateTime::from_str(&start_date)
                        .unwrap()
                        .add_weeks(3),
                    ..Default::default()
                };
                sprint.save(&path).ok().unwrap();
                let sprint_table = ctx.create_table().ok().unwrap();
                sprint_table.set("id", sprint.id.to_string()).ok().unwrap();
                sprint_table
                    .set("project_id", sprint.project_id.to_string())
                    .ok()
                    .unwrap();
                sprint_table.set("name", sprint.name).ok().unwrap();
                sprint_table
                    .set("start_date", sprint.start_date.to_string())
                    .ok()
                    .unwrap();
                sprint_table
                    .set("end_date", sprint.end_date.to_string())
                    .ok()
                    .unwrap();
                sprint_table
                    .set("created_at", sprint.created_at.to_string())
                    .ok()
                    .unwrap();
                sprint_table
                    .set("update_at", sprint.updated_at.to_string())
                    .ok()
                    .unwrap();
                Ok(sprint_table)
            },
        )
        .unwrap();
    ctx.globals()
        .set("agile_create_sprint", f)
        .unwrap();
}