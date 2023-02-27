mod steps;

use std::collections::HashMap;
use std::env::temp_dir;
use std::fmt::Debug;
use std::sync::Arc;

use cache::state_resource::ResourceFactory;
use database::DefaultDatabaseStore;
use cache::state::YummyState;
use uuid::Uuid;
use database::create_database;
use general::websocket::WebsocketTestClient;
use general::web::json_error_handler;
use general::meta::MetaType;
use testing::model::*;

use manager::conn::ConnectionManager;
use actix_web::web::Data;
use crate::api::websocket::websocket_endpoint;

use manager::auth::AuthManager;
use manager::user::UserManager;
use manager::room::RoomManager;
use manager::plugin::PluginExecuter;

use async_trait::async_trait;

use actix::Actor;

use actix_test::{TestServer, TestServerConfig};
use actix_web::web::ServiceConfig;
use actix_web::App;
use actix_web::web::JsonConfig;
use actix_web::web::QueryConfig;
use actix_web::error::InternalError;
use actix_web::HttpResponse;
use actix_web::web::get;

use cucumber::*;
use database::create_connection;
use general::config::YummyConfig;

#[derive(Default)]
pub struct CustomWriter {
    passed_steps: usize,
    skipped_steps: usize,
    failed_steps: usize,
    parsing_errors: usize,
    hook_errors: usize,
}

#[async_trait(?Send)]
impl<W: 'static> cucumber::Writer<W> for CustomWriter {
    type Cli = cli::Empty; // we provide no CLI options

    async fn handle_event(&mut self, ev: parser::Result<Event<event::Cucumber<W>>>, _: &Self::Cli) {
        match ev {
            Ok(Event { value, .. }) => match value {
                event::Cucumber::Feature(feature, ev) => match ev {
                    event::Feature::Started => {
                        println!("{}: {}", feature.keyword, feature.name)
                    }
                    event::Feature::Scenario(scenario, ev) => match ev.event {
                        event::Scenario::Started => {
                            println!("\r\n{}: {}", scenario.keyword, scenario.name)
                        }
                        event::Scenario::Step(step, ev) => match ev {
                            event::Step::Started => {
                                println!("  {} {}...", step.keyword, step.value)
                            }
                            event::Step::Passed(_, _) => {
                                self.passed_steps += 1;
                            }
                            event::Step::Skipped => {
                                self.skipped_steps += 1;
                                println!("Skipped: {:?} {:?}", feature.path, step)
                            }
                            event::Step::Failed(_, _, _, err) => {
                                self.failed_steps += 1;
                                println!("failed: {err} {:?}", step)
                            }
                        },
                        _ => {}
                    },
                    _ => {}
                },
                _ => {}
            },
            Err(e) => {
                self.parsing_errors += 1;
                println!("failed: {}", e)
            }
        }
    }
}

#[async_trait(?Send)]
impl<W: 'static> cucumber::StatsWriter<W> for CustomWriter {
    fn passed_steps(&self) -> usize {
        self.passed_steps
    }

    fn skipped_steps(&self) -> usize {
        self.skipped_steps
    }

    fn retried_steps(&self) -> usize { 0 }

    fn failed_steps(&self) -> usize {
        self.failed_steps
    }

    fn parsing_errors(&self) -> usize {
        self.parsing_errors
    }
    
    fn hook_errors(&self) -> usize {
        self.hook_errors
    }
}

pub fn create_websocket_server_with_config(config: Arc<YummyConfig>, test_server_config: TestServerConfig) -> TestServer {
    let config = config.clone();
    
    actix_test::start_with(test_server_config, move || {
        let mut db_location = temp_dir();
        db_location.push(format!("{}.db", Uuid::new_v4()));

        let connection = create_connection(db_location.to_str().unwrap()).unwrap();
        create_database(&mut connection.clone().get().unwrap()).unwrap();
        
        #[cfg(feature = "stateless")]
        let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();

        let resource_factory = ResourceFactory::<DefaultDatabaseStore>::new(config.clone(), Arc::new(connection.clone()));
        let states = YummyState::new(config.clone(), Box::new(resource_factory), #[cfg(feature = "stateless")] conn.clone());
        let executer = Arc::new(PluginExecuter::default());

        ConnectionManager::new(config.clone(), states.clone(), executer.clone(), #[cfg(feature = "stateless")] conn.clone()).start();

        let auth_manager = Data::new(AuthManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection.clone()), executer.clone()).start());
        let user_manager = Data::new(UserManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection.clone()), executer.clone()).start());
        let room_manager = Data::new(RoomManager::<database::SqliteStore>::new(config.clone(), states, Arc::new(connection), executer.clone()).start());

        let query_cfg = QueryConfig::default()
            .error_handler(|err, _| {
                log::error!("{:?}", err);
                InternalError::from_response(err, HttpResponse::Conflict().finish()).into()
            });

        App::new()
            .app_data(auth_manager)
            .app_data(user_manager)
            .app_data(room_manager)
            .app_data(query_cfg)
            .app_data(JsonConfig::default().error_handler(json_error_handler))
            .app_data(Data::new(config.clone()))
            .route("/v1/socket", get().to(websocket_endpoint::<database::SqliteStore>))
    })
}

pub struct TestServerCapsule {
    pub server: TestServer,
}

impl TestServerCapsule {
    pub fn new() -> Self {
        Self { server: create_websocket_server_with_config(::general::config::get_configuration(), TestServerConfig::default()) }
    }

    pub fn url(&self, uri: &str) -> String {
        self.server.url(uri)
    }
}

impl Debug for TestServerCapsule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TestServerCapsule")
    }
}

pub struct ClientInfo {
    socket: WebsocketTestClient<String, String>,
    last_message: Option<String>,
    last_error: Option<String>,
    room_id: Option<usize>,
    name: Option<String>,
    message: String,
    token: String,
    memory: HashMap<String, String>,
    rooms: HashMap<String, String>
}

#[derive(World)]
pub struct YummyWorld {
    ws_server: TestServerCapsule,
    ws_clients: HashMap<String, ClientInfo>,
    rooms: HashMap<String, String>
}

impl Default for YummyWorld {
    fn default() -> Self {
        Self {
            ws_clients: HashMap::new(),
            ws_server: TestServerCapsule::new(),
            rooms: HashMap::default()
        }
    }
}

impl Debug for YummyWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "YummyWorld")
    }
}

pub fn config(cfg: &mut ServiceConfig) {
    let config = ::general::config::get_configuration();
    let mut db_location = temp_dir();
    db_location.push(format!("{}.db", Uuid::new_v4()));

    let connection = create_connection(db_location.to_str().unwrap()).unwrap();
    create_database(&mut connection.clone().get().unwrap()).unwrap();
    
    #[cfg(feature = "stateless")]
    let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();

    let resource_factory = ResourceFactory::<DefaultDatabaseStore>::new(config.clone(), Arc::new(connection.clone()));
    let states = YummyState::new(config.clone(), Box::new(resource_factory), #[cfg(feature = "stateless")] conn.clone());
    let executer = Arc::new(PluginExecuter::default());

    ConnectionManager::new(config.clone(), states.clone(), executer.clone(), #[cfg(feature = "stateless")] conn.clone()).start();

    let auth_manager = Data::new(AuthManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection.clone()), executer.clone()).start());
    let user_manager = Data::new(UserManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection.clone()), executer.clone()).start());
    let room_manager = Data::new(RoomManager::<database::SqliteStore>::new(config.clone(), states, Arc::new(connection), executer.clone()).start());

    let query_cfg = QueryConfig::default()
        .error_handler(|err, _| {
            log::error!("{:?}", err);
            InternalError::from_response(err, HttpResponse::Conflict().finish()).into()
        });

    cfg
        .app_data(auth_manager)
        .app_data(user_manager)
        .app_data(room_manager)
        .app_data(query_cfg)
        .app_data(JsonConfig::default().error_handler(json_error_handler))
        .app_data(Data::new(config.clone()))
        .route("/v1/socket", get().to(websocket_endpoint::<database::SqliteStore>));
}

#[actix_web::test]
async fn cucumber_test() {
    use actix_web::test::init_service;
    use cucumber::WriterExt;
    let writer = CustomWriter::default();

    init_service(App::new().configure(config)).await;
    YummyWorld::cucumber()
        .with_writer(writer.normalized())
        .run_and_exit("./tests/").await;
}
