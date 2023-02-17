use crate::context::Context;
use std::{marker::PhantomData, sync::Arc};

pub trait OutputProcessing {
  type INPUT;
  type OUTPUT;
  /// Transform an output to an array of outputs.
  fn process(&self, output: Self::INPUT, context: Arc<Context>) -> Vec<(Self::OUTPUT, Arc<Context>)>;
}

// type OutputProcessorFunction<T, U> =
//   Box<dyn Fn(T, Arc<Context>) -> Vec<(U, Arc<Context>)> + Clone + Sync + Send>;
/// Default output processor just convert I to [I]
#[derive(Clone)]
pub struct DefaultOutputProcessor<I, O> {
  input: PhantomData<I>,
  output: PhantomData<O>,
}

impl<T, U: From<T>> Default for DefaultOutputProcessor<T, U> {
  fn default() -> DefaultOutputProcessor<T, U> {
    DefaultOutputProcessor {
      input: PhantomData,
      output: PhantomData,
    }
  }
}

impl<I: Clone, O: From<I> + Clone> OutputProcessing for DefaultOutputProcessor<I, O> {
  type INPUT = I;
  type OUTPUT = O;

  /// Default output processor just convert `output` to `[output]`.
  fn process(&self, output: Self::INPUT, context: Arc<Context>) -> Vec<(Self::OUTPUT, Arc<Context>)> {
    vec![(output.into(), context)]
  }
}

// #[cfg(test)]
// mod tests {
//   use super::*;
//   use crate::{
//     context::FlowContext,
//     graph::{InputRef, OutputRef, Relation},
//   };
//   use std::{collections::HashMap, sync::Mutex};

//   #[test]
//   fn default() {
//     let op: DefaultOutputProcessor<i32, i32> = DefaultOutputProcessor::default();
//     let context = Arc::new(Context {
//       process_id: "id".into(),
//       relation: Relation {
//         from: OutputRef {
//           node: "a".to_string(),
//           output: "o".to_string(),
//         },
//         to: InputRef {
//           node: "n".to_string(),
//           input: "i".to_string(),
//         },
//       },
//       context: FlowContext {
//         labels: Arc::new(Mutex::new(HashMap::new())),
//         context: Arc::new(Mutex::new(HashMap::new())),
//       },
//     });

//     let res = op.process(4, context);

//     assert_eq!(res[0].0, 4);
//   }
// }
