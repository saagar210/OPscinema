use opscinema_types::StepModel;
use schemars::schema_for;

fn main() {
    let schema = schema_for!(StepModel);
    let json = serde_json::to_string_pretty(&schema).expect("schema json");
    println!("{}", json);
}
