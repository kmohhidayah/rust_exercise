use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;

// Definisikan struct untuk response JSON
#[derive(Debug, Deserialize, Serialize)]
struct Todo {
    id: i32,
    title: String,
    completed: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Buat client
    let client = reqwest::Client::new();

    // GET request
    let todos: Vec<Todo> = client
        .get("https://jsonplaceholder.typicode.com/todos")
        .send()
        .await?
        .json()
        .await?;

    println!("Todos: {:?}", todos);

    // POST request dengan data
    let new_todo = Todo {
        id: 1,
        title: String::from("Belajar Rust"),
        completed: false,
    };

    let response = client
        .post("https://jsonplaceholder.typicode.com/todos")
        .json(&new_todo)
        .send()
        .await?;

    println!("Status: {}", response.status());

    Ok(())
}
