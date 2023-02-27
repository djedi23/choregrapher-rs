use crate::{graph::Relation, App};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
  collections::HashMap,
  fmt::Debug,
  sync::{Arc, Mutex},
};

pub type ContextDict = HashMap<String, Value>;

/// User context. You can put transient data in it. It will be passed in the messages.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FlowContext {
  /// Labels are use as filter in some nodes (map, join)
  pub labels: Arc<Mutex<ContextDict>>,
  /// user variables
  pub context: Arc<Mutex<ContextDict>>,
}

impl Default for FlowContext {
  fn default() -> Self {
    Self {
      labels: Arc::new(Mutex::new(HashMap::new())),
      context: Arc::new(Mutex::new(HashMap::new())),
    }
  }
}

/// The context used by the node's action.
/// It contains some metadata and some shared variables
#[derive(Clone, Serialize)]
pub struct Context {
  pub process_id: String,
  pub relation: Relation,
  /// The context that will travel with the messages. You can read and modify it.
  pub context: FlowContext,
  /// The application context (collections, channels)
  // It's an option to be able to test the Context without settings up a real mongo and rabbmitmq
  #[serde(skip)]
  pub app: Option<Arc<App>>,
}

impl Debug for Context {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Context")
      .field("process_id", &self.process_id)
      .field("relation", &self.relation)
      .field("context", &self.context)
      .finish()
  }
}

impl Context {
  /// Get the value associated to a label
  pub fn get_label(&self, key: String) -> Option<Value> {
    let label = &self.context.labels.lock().unwrap();
    label.get(&key).cloned()
  }

  /// Set a value associated to a label
  /// # Examples
  ///
  /// ```
  /// use serde_json::{json, to_string};
  /// use node_rs::context::{Context, ContextDict, FlowContext};
  /// use node_rs::graph::{InputRef, OutputRef, Relation};
  ///
  /// let context = Context {
  ///      process_id: "id".into(),
  ///      relation: Relation {
  ///        from: OutputRef {
  ///          node: "a".to_string(),
  ///          output: "o".to_string(),
  ///        },
  ///        to: InputRef {
  ///          node: "n".to_string(),
  ///          input: "i".to_string(),
  ///        },
  ///      },
  ///      context: FlowContext::default(),
  ///      app: None,
  /// };
  ///
  ///  context.insert_label("key".into(), json!(5));
  ///  assert_eq!(to_string(&context).unwrap(), "{\"process_id\":\"id\",\"relation\":{\"from\":{\"node\":\"a\",\"output\":\"o\"},\"to\":{\"node\":\"n\",\"input\":\"i\"}},\"context\":{\"labels\":{\"key\":5},\"context\":{}}}" );
  /// ```
  pub fn insert_label(&self, key: String, value: Value) -> Option<Value> {
    let mut labels = self.context.labels.lock().unwrap();
    labels.insert(key, value)
  }

  pub fn remove_label(&self, key: String) -> Option<Value> {
    let mut labels = self.context.labels.lock().unwrap();
    labels.remove(&key)
  }

  /// Get the value associated to a context
  pub fn get_context(&self, key: String) -> Option<Value> {
    let context = &self.context.context.lock().unwrap();
    context.get(&key).cloned()
  }

  /// Set a value associated to a label
  /// # Examples
  ///
  /// ```
  /// use serde_json::{json, to_string};
  /// use node_rs::context::{Context, ContextDict, FlowContext};
  /// use node_rs::graph::{InputRef, OutputRef, Relation};
  ///
  /// let context = Context {
  ///      process_id: "id".into(),
  ///      relation: Relation {
  ///        from: OutputRef {
  ///          node: "a".to_string(),
  ///          output: "o".to_string(),
  ///        },
  ///        to: InputRef {
  ///          node: "n".to_string(),
  ///          input: "i".to_string(),
  ///        },
  ///      },
  ///     context: FlowContext::default(),
  ///     app: None
  ///  };
  ///
  ///  context.insert_context("key".into(), json!(5));
  ///  assert_eq!(to_string(&context).unwrap(), "{\"process_id\":\"id\",\"relation\":{\"from\":{\"node\":\"a\",\"output\":\"o\"},\"to\":{\"node\":\"n\",\"input\":\"i\"}},\"context\":{\"labels\":{},\"context\":{\"key\":5}}}" );
  /// ```
  pub fn insert_context(&self, key: String, value: Value) -> Option<Value> {
    let mut context = self.context.context.lock().unwrap();
    context.insert(key, value)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::graph::{InputRef, OutputRef};
  use serde_json::json;

  // #SPC-processing.tst-context_serialize_context_1
  #[test]
  fn serialize_context_1() {
    if let Ok(json) = serde_json::to_string(&Context {
      process_id: "id".into(),
      relation: Relation {
        from: OutputRef {
          node: "a".to_string(),
          output: "o".to_string(),
        },
        to: InputRef {
          node: "n".to_string(),
          input: "i".to_string(),
        },
      },
      context: FlowContext {
        labels: Arc::new(Mutex::new(HashMap::new())),
        context: Arc::new(Mutex::new(HashMap::new())),
      },
      app: None,
    }) {
      assert_eq!(
       json,
       "{\"process_id\":\"id\",\"relation\":{\"from\":{\"node\":\"a\",\"output\":\"o\"},\"to\":{\"node\":\"n\",\"input\":\"i\"}},\"context\":{\"labels\":{},\"context\":{}}}"
     )
    };
  }

  // #SPC-processing.tst-context_serialize_context_2
  #[test]
  fn serialize_context_2() {
    let mut labels = HashMap::new();
    labels.insert("key".into(), json!(5));

    match serde_json::to_string(&Context {
    process_id: "id".into(),
    relation: Relation {
      from: OutputRef {
        node: "a".to_string(),
        output: "o".to_string(),
      },
      to: InputRef {
        node: "n".to_string(),
        input: "i".to_string(),
      },
    },
    context: FlowContext {
      labels: Arc::new(Mutex::new(labels)),
      context: Arc::new(Mutex::new(HashMap::new())),
    }, app:None
  }) {
    Ok(json) => assert_eq!(
      json,
      "{\"process_id\":\"id\",\"relation\":{\"from\":{\"node\":\"a\",\"output\":\"o\"},\"to\":{\"node\":\"n\",\"input\":\"i\"}},\"context\":{\"labels\":{\"key\":5},\"context\":{}}}"
    ),
    _ => unreachable!( "Error during the Context serialization."),
  };
  }

  // #SPC-processing.tst-context_serialize_context_3
  #[test]
  fn serialize_context_3() {
    let mut context = HashMap::new();
    context.insert("key".into(), json!(5));

    match serde_json::to_string(&Context {
    process_id: "id".into(),
    relation: Relation {
      from: OutputRef {
        node: "a".to_string(),
        output: "o".to_string(),
      },
      to: InputRef {
        node: "n".to_string(),
        input: "i".to_string(),
      },
    },
    context: FlowContext {
      labels: Arc::new(Mutex::new(HashMap::new())),
      context: Arc::new(Mutex::new(context)),
    }, app:None
  }) {
    Ok(json) => assert_eq!(
      json,
      "{\"process_id\":\"id\",\"relation\":{\"from\":{\"node\":\"a\",\"output\":\"o\"},\"to\":{\"node\":\"n\",\"input\":\"i\"}},\"context\":{\"labels\":{},\"context\":{\"key\":5}}}"
    ),
    _ => unreachable!( "Error during the Context serialization."),
  };
  }

  #[test]
  fn labels_setter() {
    let context = Context {
      process_id: "id".into(),
      relation: Relation {
        from: OutputRef {
          node: "a".to_string(),
          output: "o".to_string(),
        },
        to: InputRef {
          node: "n".to_string(),
          input: "i".to_string(),
        },
      },
      context: FlowContext::default(),
      app: None,
    };

    context.insert_label("key".into(), json!(5));

    match serde_json::to_string(&context) {
    Ok(json) => assert_eq!(
      json,
      "{\"process_id\":\"id\",\"relation\":{\"from\":{\"node\":\"a\",\"output\":\"o\"},\"to\":{\"node\":\"n\",\"input\":\"i\"}},\"context\":{\"labels\":{\"key\":5},\"context\":{}}}"
    ),
    _ => unreachable!( "Error during the Context serialization."),
  };
  }

  #[test]
  fn labels_setter2() {
    let context = Context {
      process_id: "id".into(),
      relation: Relation {
        from: OutputRef {
          node: "a".to_string(),
          output: "o".to_string(),
        },
        to: InputRef {
          node: "n".to_string(),
          input: "i".to_string(),
        },
      },
      context: FlowContext::default(),
      app: None,
    };

    context.insert_label("key".into(), json!(1));
    context.insert_label("key1".into(), json!("text"));
    context.insert_label("key2".into(), json!({"array":[9,8,"e"]}));
    context.insert_label("key".into(), json!(4));

    let label = context.context.labels.lock().unwrap();

    assert_eq!(label.get("key").unwrap(), &json!(4));
    assert_eq!(label.get("key1").unwrap(), &json!("text"));
    assert_eq!(label.get("key2").unwrap(), &json!({"array":[9,8,"e"]}));
  }

  #[test]
  fn label_getter() {
    let mut labels = HashMap::new();
    labels.insert("key".into(), json!(5));

    let context = Context {
      process_id: "id".into(),
      relation: Relation {
        from: OutputRef {
          node: "a".to_string(),
          output: "o".to_string(),
        },
        to: InputRef {
          node: "n".to_string(),
          input: "i".to_string(),
        },
      },
      context: FlowContext {
        labels: Arc::new(Mutex::new(labels)),
        context: Arc::new(Mutex::new(HashMap::new())),
      },
      app: None,
    };

    assert_eq!(context.get_label("key".into()).unwrap(), json!(5));
  }

  #[test]
  fn context_setter() {
    let context = Context {
      process_id: "id".into(),
      relation: Relation {
        from: OutputRef {
          node: "a".to_string(),
          output: "o".to_string(),
        },
        to: InputRef {
          node: "n".to_string(),
          input: "i".to_string(),
        },
      },
      context: FlowContext::default(),
      app: None,
    };

    context.insert_context("key".into(), json!(5));

    match serde_json::to_string(&context) {
    Ok(json) => assert_eq!(
      json,
      "{\"process_id\":\"id\",\"relation\":{\"from\":{\"node\":\"a\",\"output\":\"o\"},\"to\":{\"node\":\"n\",\"input\":\"i\"}},\"context\":{\"labels\":{},\"context\":{\"key\":5}}}"
    ),
    _ => unreachable!( "Error during the Context serialization."),
  };
  }

  #[test]
  fn context_setter2() {
    let context = Context {
      process_id: "id".into(),
      relation: Relation {
        from: OutputRef {
          node: "a".to_string(),
          output: "o".to_string(),
        },
        to: InputRef {
          node: "n".to_string(),
          input: "i".to_string(),
        },
      },
      context: FlowContext::default(),
      app: None,
    };

    context.insert_context("key".into(), json!(1));
    context.insert_context("key1".into(), json!("text"));
    context.insert_context("key2".into(), json!({"array":[9,8,"e"]}));
    context.insert_context("key".into(), json!(4));

    let context = context.context.context.lock().unwrap();

    assert_eq!(context.get("key").unwrap(), &json!(4));
    assert_eq!(context.get("key1").unwrap(), &json!("text"));
    assert_eq!(context.get("key2").unwrap(), &json!({"array":[9,8,"e"]}));
  }

  #[test]
  fn context_getter() {
    let mut context_context = HashMap::new();
    context_context.insert("key".into(), json!(5));

    let context = Context {
      process_id: "id".into(),
      relation: Relation {
        from: OutputRef {
          node: "a".to_string(),
          output: "o".to_string(),
        },
        to: InputRef {
          node: "n".to_string(),
          input: "i".to_string(),
        },
      },
      context: FlowContext {
        labels: Arc::new(Mutex::new(HashMap::new())),
        context: Arc::new(Mutex::new(context_context)),
      },
      app: None,
    };

    assert_eq!(context.get_context("key".into()).unwrap(), json!(5));
  }
}
