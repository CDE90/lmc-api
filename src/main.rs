use actix_cors::Cors;
use actix_web::{get, middleware::Logger, post, web, App, HttpResponse, HttpServer, Responder};
use env_logger::Env;
use lmc_assembly::{self, ExecutionState, Output};
use serde::{Deserialize, Serialize};

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[derive(Serialize, Deserialize)]
struct ExecutionRequest {
    state: ExecutionState,
    input: Vec<i16>,
}

#[derive(Serialize, Deserialize)]
struct ExecutionResponse {
    state: ExecutionState,
    output: Vec<Output>,
    input_success: Option<bool>,
    next_requires_input: bool,
}

#[post("/assemble")]
async fn assemble(req_body: String) -> impl Responder {
    let parsed = match lmc_assembly::parse(req_body.as_str(), false) {
        Ok(parsed) => parsed,
        Err(e) => {
            return HttpResponse::BadRequest().body(e);
        }
    };
    let assembled = match lmc_assembly::assemble(parsed) {
        Ok(assembled) => assembled,
        Err(e) => {
            return HttpResponse::BadRequest().body(e);
        }
    };

    let mut assembled_string = String::from('[');
    for item in &assembled {
        assembled_string.push_str(&item.to_string());
        assembled_string.push(',');
    }
    assembled_string.push(']');

    let state = ExecutionState {
        pc: 0,
        acc: 0,
        cir: 0,
        mar: 0,
        mdr: 0,
        ram: assembled,
    };

    let response = ExecutionResponse {
        state,
        output: Vec::new(),
        input_success: None,
        next_requires_input: false,
    };
    HttpResponse::Ok().json(response)
}

struct IOHandler {
    input: Vec<i16>,
    input_success: Option<bool>,
    output: Vec<Output>,
}

impl lmc_assembly::LMCIO for IOHandler {
    fn get_input(&mut self) -> i16 {
        if !self.input.is_empty() {
            self.input_success = Some(true);
            self.input.pop().unwrap()
        } else {
            self.input_success = Some(false);
            0
        }
    }
    fn print_output(&mut self, val: lmc_assembly::Output) {
        self.output.push(val);
    }
}

#[post("/step")]
async fn step_execution(request: web::Json<ExecutionRequest>) -> impl Responder {
    let state = request.into_inner();

    let mut execution_state = state.state;
    let mut io_handler = IOHandler {
        input: state.input,
        input_success: None,
        output: Vec::new(),
    };

    match execution_state.step(&mut io_handler) {
        Ok(_) => {}
        Err(e) => {
            return HttpResponse::BadRequest().body(e);
        }
    }

    let input_success = io_handler.input_success;

    match input_success {
        Some(true) => {
            let response = ExecutionResponse {
                next_requires_input: execution_state.ram.get(execution_state.pc as usize)
                    == Some(&901),
                state: execution_state,
                output: io_handler.output,
                input_success: Some(true),
            };
            HttpResponse::Ok().json(response)
        }
        Some(false) => {
            let response = ExecutionResponse {
                next_requires_input: execution_state.ram.get(execution_state.pc as usize)
                    == Some(&901),
                state: execution_state,
                output: vec![],
                input_success: Some(false),
            };
            HttpResponse::BadRequest().json(response)
        }
        None => {
            let response = ExecutionResponse {
                next_requires_input: execution_state.ram.get(execution_state.pc as usize)
                    == Some(&901),
                state: execution_state,
                output: io_handler.output,
                input_success: None,
            };
            HttpResponse::Ok().json(response)
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .service(assemble)
            .service(step_execution)
            .service(index)
    })
    .bind(("0.0.0.0", 5001))?
    .run()
    .await
}
