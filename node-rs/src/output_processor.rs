use crate::context::Context;
use std::rc::Rc;

pub trait OutputProcessing {
  type INPUT;
  type OUTPUT;
  /// Transform an output to an array of outputs.
  fn process(&self, output: Self::INPUT, context: Rc<Context>) -> Vec<(Self::OUTPUT, Rc<Context>)>;
}

type OutputProcessorFunction<T, U> = Box<dyn Fn(T, Rc<Context>) -> Vec<(U, Rc<Context>)>>;
pub struct OutputProcessor<T, U> {
  process_fn: OutputProcessorFunction<T, U>,
}

impl<T, U> OutputProcessor<T, U> {
  pub fn new(func: OutputProcessorFunction<T, U>) -> OutputProcessor<T, U> {
    OutputProcessor { process_fn: func }
  }
}
impl<T, U: From<T>> Default for OutputProcessor<T, U> {
  fn default() -> OutputProcessor<T, U> {
    OutputProcessor::new(Box::new(
      |out: T, context: Rc<Context>| -> Vec<(U, Rc<Context>)> { vec![(out.into(), context)] },
    ))
  }
}

impl<T, U> OutputProcessing for OutputProcessor<T, U> {
  type INPUT = T;
  type OUTPUT = U;

  fn process(&self, output: T, context: Rc<Context>) -> Vec<(U, Rc<Context>)> {
    (self.process_fn)(output, context)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    context::FlowContext,
    graph::{InputRef, OutputRef, Relation},
  };
  use std::cell::RefCell;
  use std::collections::HashMap;

  #[test]
  fn default() {
    let op: OutputProcessor<i32, i32> = OutputProcessor::default();
    let context = Rc::new(Context {
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
        labels: Rc::new(RefCell::new(HashMap::new())),
        context: Rc::new(RefCell::new(HashMap::new())),
      },
    });

    let res = op.process(4, context);

    assert_eq!(res[0].0, 4);
  }

  #[test]
  fn new() {
    let op: OutputProcessor<i32, i32> = OutputProcessor::new(Box::new(
      |out: i32, context: Rc<Context>| -> Vec<(i32, Rc<Context>)> { vec![(out, context)] },
    ));

    let context = Rc::new(Context {
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
        labels: Rc::new(RefCell::new(HashMap::new())),
        context: Rc::new(RefCell::new(HashMap::new())),
      },
    });

    let res = op.process(4, context);

    assert_eq!(res[0].0, 4);
  }
}
