pub mod models;
pub mod schema;

use diesel::prelude::*;
use fuzzy_matcher::FuzzyMatcher;

use std::{collections::HashMap, env, fs, sync::Arc};

use diesel::r2d2::ConnectionManager;
use r2d2::Pool;

use abi_stable::std_types::{ROption, RString, RVec};
use anyrun_plugin::*;
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use serde::Deserialize;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[derive(Deserialize)]
pub struct Config {
    prefix: Option<String>,
    ha_language: Option<String>,
    ha_url: url::Url,
    ha_token: String,
}

pub struct State {
    config: Config,
    connection_pool: Option<Arc<Pool<ConnectionManager<SqliteConnection>>>>,
    responses: HashMap<String, Response>,
}

#[init]
fn init(config_dir: RString) -> State {
    let content =
        fs::read_to_string(format!("{}/ha-assist.ron", config_dir)).unwrap_or_else(|why| {
            panic!(
                "Error reading anyrun-ha-assist config file ({}/ha-assist.ron).\n{}",
                config_dir, why
            )
        });
    let mut cfg: Config = ron::from_str(&content).unwrap();

    cfg.ha_url.set_path("api/conversation/process");

    let database_url = env::var("XDG_CACHE_HOME")
        .and_then(|cache_dir| Ok(cache_dir))
        .or_else(|_| {
            env::var("HOME")
                .map(|home| format!("{}/.cache", home))
                .map_err(|_| "Unable to determine cache directory")
        })
        .map(|cache_dir| format!("{}/anyrun-ha-assist.sqlite3", cache_dir));

    let connection_pool = match database_url {
        Ok(database_url) => {
            let manager = ConnectionManager::<SqliteConnection>::new(&database_url);

            Some(Arc::new(
                Pool::builder()
                    .max_size(1)
                    .build(manager)
                    .expect("Failed to create connection pool."),
            ))
        }
        Err(why) => {
            eprintln!("Failed to create sqlite3 database for anyrun-ha-assist. Functionality will be gowno.\n{:#?}", why);
            None
        }
    };

    if let Some(pool) = &connection_pool {
        if let Err(why) = pool.get().unwrap().run_pending_migrations(MIGRATIONS) {
            eprintln!("Failed to create sqlite3 database for anyrun-ha-assist. Functionality will be decreased.\n{:#?}", why);
        }
    };

    State {
        config: cfg,
        responses: HashMap::new(),
        connection_pool,
    }
}

#[info]
fn info() -> PluginInfo {
    PluginInfo {
        name: "HA Assist".into(),
        icon: "go-home".into(),
    }
}

#[get_matches]
fn get_matches(input: RString, state: &mut State) -> RVec<Match> {
    let input = if let Some(input) =
        input.strip_prefix(&state.config.prefix.clone().unwrap_or(":ha".to_string()))
    {
        input.trim()
    } else {
        return RVec::new();
    };

    let mut matches = vec![];

    if !input.is_empty() {
        let response = state.responses.remove(input);

        let description = response.as_ref().map_or(ROption::RNone, |r| {
            ROption::RSome(RString::from(r.speech.plain.speech.clone()))
        });

        let icon = response.map_or(ROption::RNone, |res| match res.response_type {
            ResponseType::ActionDone | ResponseType::QueryAnswer => {
                ROption::RSome("emblem-success".into())
            }
            ResponseType::Error => ROption::RSome("emblem-error".into()),
        });

        matches.push(Match {
            title: input.into(),
            icon,
            use_pango: false,
            description,
            id: ROption::RNone,
        });
    }

    if let Some(pool) = &mut state.connection_pool {
        use crate::schema::history;

        let result = history::table
            .group_by(history::query)
            .select((
                diesel::dsl::sql::<diesel::sql_types::BigInt>("COUNT(query)"),
                history::query,
            ))
            .order(diesel::dsl::sql::<diesel::sql_types::BigInt>("COUNT(query)").desc())
            .load::<(i64, String)>(&mut pool.get().unwrap());

        if let Ok(history) = result {
            let matcher = fuzzy_matcher::skim::SkimMatcherV2::default().smart_case();

            let mut entries = history
                .iter()
                .filter_map(|(_count, query)| {
                    let score = matcher.fuzzy_match(&query, &input).unwrap_or(0);

                    if score > 0 {
                        Some((score, query))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            entries.sort_by(|a, b| b.0.cmp(&a.0));

            for entry in entries {
                let response = state.responses.remove(&entry.1.clone());

                let description = response.as_ref().map_or(ROption::RNone, |r| {
                    ROption::RSome(RString::from(r.speech.plain.speech.clone()))
                });

                let icon = response.map_or(ROption::RNone, |res| match res.response_type {
                    ResponseType::ActionDone | ResponseType::QueryAnswer => {
                        ROption::RSome("emblem-success".into())
                    }
                    ResponseType::Error => ROption::RSome("emblem-error".into()),
                });

                matches.push(Match {
                    title: entry.1.clone().into(),
                    description,
                    icon,
                    id: ROption::RNone,
                    use_pango: false,
                });
            }
        }
    }

    matches.into()
}

#[derive(Deserialize, Debug)]
pub struct PlainSpeech {
    speech: String,
}

#[derive(Deserialize, Debug)]
pub struct Speech {
    plain: PlainSpeech,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ResponseType {
    ActionDone,
    QueryAnswer,
    Error,
}

#[derive(Deserialize, Debug)]
pub struct Response {
    response_type: ResponseType,
    speech: Speech,
}

#[derive(Deserialize, Debug)]
pub struct ConversationResponse {
    response: Response,
}

#[handler]
fn handler(selection: Match, state: &mut State) -> HandleResult {
    let client = reqwest::blocking::Client::new();

    let response = client
        .post(state.config.ha_url.clone())
        .json(&serde_json::json!({
            "language": state.config.ha_language.clone().unwrap_or("en".to_string()),
            "text": selection.title,
        }))
        .bearer_auth(&state.config.ha_token)
        .send();

    match response {
        Ok(response) => {
            let data = response.json::<ConversationResponse>();

            if let Ok(response) = data {
                use crate::schema::history;
                use ResponseType::*;

                match response.response.response_type {
                    Error => (),
                    ActionDone | QueryAnswer => {
                        if let Some(pool) = &mut state.connection_pool {
                            let new_history = crate::models::NewHistory {
                                query: &selection.title,
                            };

                            diesel::insert_into(history::table)
                                .values(&new_history)
                                .execute(&mut pool.get().unwrap())
                                .ok();
                        }
                    }
                };

                state
                    .responses
                    .insert(selection.title.to_string(), response.response);
            }
        }
        Err(why) => {
            eprintln!("Assist request failed: {}", why);
        }
    };

    HandleResult::Refresh(true)
}
