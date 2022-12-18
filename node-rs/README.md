# Choregrapher Node for Rust

Choregrapher is a library to easily build distributed computing graph. 

## Features

- Choreographies are described using rust structs or using JSON object.
It use Mongo DB as database for the choreographies, storing the results of the processes or as object storages. The choreographies can be stored and loaded in the database.
- Graph nodes can:
  - have multiple inputs and multiple outputs
- all processes steps are saved in database

## Examples
### Running the examples 
Launch the database and the bus:
```shell
docker compose up -d
```

Run an example
```shell
RUST_LOG=debug cargo run --example fact
```

### Factorial

Let's code a factorial. The complete code is in `./examples/fact.rs`.

![factorial](./examples/fact.png)
We are coding a recursive version of the factorial.

The input `i` is an struct describing the state of the computation.

``` rust
#[derive(Serialize, Deserialize, Debug, Clone)]
struct FactInput<T: Sub + Mul> {
  i: Fact<T>,
}
```
The state `Fact` can be:
- `N(n,acc)` where `n` is the current _n_ and `acc` the current value of hte factorial.
- any other alternative useful for the computation. We can imagine a `FACT(acc)` where `acc` is the final value of the computation

``` rust
#[derive(Serialize, Deserialize, Debug, Clone, FieldAccessor)]
enum Fact<T: Sub + Mul> {
  #[serde(rename = "n")]
  N(T, T),
}
```

The output type is
- `O(state)` if the computation is not complete;
- `R(T)` is the computation is complete.

``` rust
#[derive(Serialize, Deserialize, Debug, Clone, FieldAccessor)]
enum FactResult<T: Sub + Mul> {
  #[serde(rename = "o")]
  O(Fact<T>),
  #[serde(rename = "r")]
  R(T),
}
```

Now, we can create the factorial function:

``` rust
async fn fact_action(
  data: FactInput<u64>,
  _context: Rc<Context>,
) -> (Option<FactResult<u64>>, Rc<Context>) {
  debug!("fact action {:?}", data);
  match data.i {
    Fact::N(1, r) => {
      info!("Fact: {:?}", r);
      (Some(FactResult::R(r)), _context)
    }
    Fact::N(n, r) => (Some(FactResult::O(Fact::N(n - 1, r * n))), _context),
  }
}
```
If we ignore some details (`async`, `context`,...), the function is a near straigthforward implementation of the tail recursive factorial. The first parameter of this function is the node's inputs and a context. The function returns the node's outputs and the context.

As we have the code, we need to declare the graph.
The main components of the graph are the `nodes` and the `relations` betweens the nodes.
The nodes declare the inputs and the outputs. The relations declare the links from a node's output to a node's input.

``` rust
let graph = Graph {
  id: String::from("fact"),
  nodes: vec![Node {
    id: "fact".to_string(),
    name: String::from("Factorial"),
    input: vec!["i".to_string()],
    output: vec!["o".to_string(), "r".to_string()],
  }],
  edges: vec![
    Relation {
      from: OutputRef {
        node: "start".to_string(),
        output: "o".to_string(),
      },
      to: InputRef {
        node: "fact".to_string(),
        input: "i".to_string(),
      },
    },
    Relation {
      from: OutputRef {
        node: "fact".to_string(),
        output: "o".to_string(),
      },
      to: InputRef {
        node: "fact".to_string(),
        input: "i".to_string(),
      },
    },
  ],
};
```

Finaly, let's associate the graph and the function in the system. After this line, the choregrapher will listen events for this node.
```rust
    graph.register_node_action(
      "fact",
      fact_action,
      None::<Box<OutputProcessor<FactResult<u64>, FactResult<u64>>>>,
    ).await;
```

Inject some data in the graph:
```rust
    let init = FactResult::O(Fact::N(20, 1));
    start_process(&channel_out, graph, &init, &None).await;
```


## Configuration

Configuration can be set in three points:
- `config/default.toml` file;
- `config/development.toml` file;
- `config/local.toml` file;
- environment variables. Fields are separated by a double underscore. `database:url` --> `DATABASE__URL`

| config                      | description                                   | Default                               |
|-----------------------------|-----------------------------------------------|---------------------------------------|
| debug                       |                                               |                                       |
| starting_node_id            |                                               |                                       |
| database:url                | URL of the database                           | mongodb://localhost:27017             |
| database:database           | name of the database                          | test                                  |
| database:collection         | Collection where the processes are saved.runs |                                       |
| database:events_collection  | Collection where the events are saved.        | events                                |
| database:graphes_collection | Collection where the graphes are saved.       | graph                                 |
| database:files_bucket       | bucket where the files are saved.             | choregrapher                          |
| rabbitmq:url                | Rabbitmq url.                                 | amqp://admin:admin@127.0.0.1:5672/%2f |
| rabbitmq:prefetch_count     | Number of event to prefetch.                  | 8                                     |
| rabbitmq:exchange_name      | Prefix of the exchange.                       | flow                                  |
| rabbitmq:routing_key        | Routing key prefix                            | flow                                  |
| rabbitmq:queues_name        | Queues prefix                                 | flow_queue                            |
