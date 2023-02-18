/// Build a new [Node](crate::graph::Node) struct.
///
/// The general case is:
/// ```rust
/// # use node_rs::{graph::Node, node};
/// node! ( (id: name) (input1, input2) -> (output1, output2) );
/// ```
///
/// Simplier form are available:
///
/// - Build a node without input or output:
/// ```rust
/// # use node_rs::{graph::Node, node};
/// node! ( id );
/// ```
/// - Build a node with just input:
/// ```rust
/// # use node_rs::{graph::Node, node};
/// node! ( id (input1, input2));
/// ```
/// - Build a node with just outputs:.
/// ```rust
/// # use node_rs::{graph::Node, node};
/// node! ( id -> (output1, output2));
/// ```
#[macro_export]
macro_rules! node {
    (@id $node_id:ident) => {stringify!($node_id).into()};
    (@name $node_id:ident) => {stringify!($node_id).into()};

    (@id ($node_id:ident : $node_name:tt)) => {stringify!($node_id).into()};
    (@name ($node_id:ident : $node_name:ident)) => {stringify!($node_name).into()};
    (@name ($node_id:ident : $node_name:literal)) => {$node_name.into()};

    ($node_id_and_name:tt) => {
	Node {
	    id: node!(@id $node_id_and_name),
	    name: node!(@name $node_id_and_name),
	    input: Vec::new(),
	    output: Vec::new(),
	}
    };
    ($node_id_and_name:tt ($($input:ident),+)) => {
	Node {
	    id: node!(@id $node_id_and_name),
	    name: node!(@name $node_id_and_name),
	    input: vec![$(stringify!($input).into()),+],
	    output: Vec::new(),
	}
    };
    ($node_id_and_name:tt -> ($($output:ident),+)) => {
	Node {
	    id: node!(@id $node_id_and_name),
	    name: node!(@name $node_id_and_name),
	    input: Vec::new(),
	    output: vec![$(stringify!($output).into()),+],
	}
    };
    ($node_id_and_name:tt ($($input:ident),+) -> ($($output:ident),+)) => {
	Node {
	    id: node!(@id $node_id_and_name),
	    name: node!(@name $node_id_and_name),
	    input: vec![$(stringify!($input).into()),+],
	    output: vec![$(stringify!($output).into()),+],
	}
    };}

/// Build a [Vec] of [Node](crate::graph::Node).
///
///
/// - Build a vec of empty nodes:.
/// ```rust
/// # use node_rs::{graph::Node, nodes, node};
/// nodes! ( node1, node2);
/// ```
/// - Build a vec of nodes:.
/// ```rust
/// # use node_rs::{graph::Node, nodes, node};
/// nodes! ( ((node1:name1)(i1,i1) -> (o1,o2)), ((node2:name2)(i1,i2) -> (o1,o2)));
/// ```
///
#[macro_export]
macro_rules! nodes {
    () => {
	Vec::new()
    };
    (@a ($($node_fragment:tt)+ )) => {
	node!($($node_fragment)+)
    };
    (@a $($node_fragment:tt)+ ) => {
	node!($($node_fragment)+)
    };

    ($($args:tt),*) => {
	{
	    let mut nodes_vec = Vec::new();
	    $(
		nodes_vec.push(nodes!(@a $args));
	    )*
		nodes_vec
	}
    }
}

/// Build a [node](crate::graph::Node) [relation](crate::graph::Relation).
///
/// The syntax is
/// ```rust
/// # use node_rs::{relation, graph::{Relation, OutputRef, InputRef}};
/// relation!(output_node(output_slot) -> (input_slot)input_node);
/// ```
#[macro_export]
macro_rules! relation {
  ($output_node:ident($output:ident) -> ($input:ident)$input_node:ident) => {
    Relation {
      from: OutputRef {
        node: stringify!($output_node).into(),
        output: stringify!($output).into(),
      },
      to: InputRef {
        node: stringify!($input_node).into(),
        input: stringify!($input).into(),
      },
    }
  };
}

/// Build a Vec<[relation](crate::graph::Relation)>.
///
/// The syntax is
/// ```rust
/// # use node_rs::{relations, graph::{Relation, OutputRef, InputRef}};
/// relations![output_node(output_slot) -> (input_slot)input_node,
/// 	   /* ... , */
/// 	   output_node_n(output_slot_n) -> (input_slot_n)input_node_n];
/// ```
#[macro_export]
macro_rules! relations {
    ($($output_node:ident($output:ident) -> ($input:ident)$input_node:ident),* ) => {
	vec![
	    $(Relation {
		from: OutputRef {
		    node: stringify!($output_node).into(),
		    output: stringify!($output).into(),
		},
		to: InputRef {
		    node: stringify!($input_node).into(),
		    input: stringify!($input).into(),
		},
	    },)*
	]
    };
}

#[cfg(test)]
mod tests {
  use crate::graph::{InputRef, Node, OutputRef, Relation};

  // #SPC-graph.tst-id
  #[test]
  fn node_isolated_node() {
    let node = node!(a);
    assert_eq!(
      node,
      Node {
        id: "a".into(),
        name: "a".into(),
        input: vec![],
        output: vec![]
      }
    );

    let node = node![a];
    assert_eq!(
      node,
      Node {
        id: "a".into(),
        name: "a".into(),
        input: vec![],
        output: vec![]
      }
    );
  }

  #[test]
  fn node_isolated_node_with_name() {
    let node = node!((a: name));
    assert_eq!(
      node,
      Node {
        id: "a".into(),
        name: "name".into(),
        input: vec![],
        output: vec![]
      }
    );

    let node = node![(a: name)];
    assert_eq!(
      node,
      Node {
        id: "a".into(),
        name: "name".into(),
        input: vec![],
        output: vec![]
      }
    );

    let node = node!((a: "a name"));
    assert_eq!(
      node,
      Node {
        id: "a".into(),
        name: "a name".into(),
        input: vec![],
        output: vec![]
      }
    );
  }

  #[test]
  fn node_input() {
    let node = node!(with_input(i1));
    assert_eq!(
      node,
      Node {
        id: "with_input".into(),
        name: "with_input".into(),
        input: vec!["i1".into()],
        output: vec![]
      }
    );
  }

  #[test]
  fn node_inputs() {
    let node = node!(with_inputs(i1, i2, i3));
    assert_eq!(
      node,
      Node {
        id: "with_inputs".into(),
        name: "with_inputs".into(),
        input: vec!["i1".into(), "i2".into(), "i3".into()],
        output: vec![]
      }
    );
  }

  #[test]
  fn node_output() {
    let node = node!(with_output -> (o1));
    assert_eq!(
      node,
      Node {
        id: "with_output".into(),
        name: "with_output".into(),
        input: vec![],
        output: vec!["o1".into()],
      }
    );
  }

  #[test]
  fn node_outputs() {
    let node = node!(with_outputs -> (o1, o2, o3));
    assert_eq!(
      node,
      Node {
        id: "with_outputs".into(),
        name: "with_outputs".into(),
        input: vec![],
        output: vec!["o1".into(), "o2".into(), "o3".into()],
      }
    );
  }

  #[test]
  fn node_full() {
    let node = node!((with_input_outputs:name)(i1, i2, i3) -> (o1, o2, o3));
    assert_eq!(
      node,
      Node {
        id: "with_input_outputs".into(),
        name: "name".into(),
        input: vec!["i1".into(), "i2".into(), "i3".into()],
        output: vec!["o1".into(), "o2".into(), "o3".into()],
      }
    );
  }

  #[test]
  fn nodes_empty() {
    let nodes: Vec<Node> = nodes!();
    assert_eq!(nodes, vec![]);
  }

  #[test]
  fn nodes_one() {
    let nodes: Vec<Node> = nodes!(a);
    assert_eq!(
      nodes,
      vec![Node {
        id: "a".into(),
        name: "a".into(),
        input: vec![],
        output: vec![],
      }]
    );

    let nodes: Vec<Node> = nodes!((a));
    assert_eq!(
      nodes,
      vec![Node {
        id: "a".into(),
        name: "a".into(),
        input: vec![],
        output: vec![],
      }]
    );

    let nodes: Vec<Node> = nodes!((a(i1, i2) -> (o1, o2)));
    assert_eq!(
      nodes,
      vec![Node {
        id: "a".into(),
        name: "a".into(),
        input: vec!["i1".into(), "i2".into()],
        output: vec!["o1".into(), "o2".into()],
      }]
    );

    let nodes: Vec<Node> = nodes!(((a:b)(i1, i2) -> (o1, o2)));
    assert_eq!(
      nodes,
      vec![Node {
        id: "a".into(),
        name: "b".into(),
        input: vec!["i1".into(), "i2".into()],
        output: vec!["o1".into(), "o2".into()],
      }]
    );
  }

  #[test]
  fn relation() {
    let relation = relation!(fact(r) -> (i)join);
    assert_eq!(
      relation,
      Relation {
        from: OutputRef {
          node: "fact".to_string(),
          output: "r".to_string(),
        },
        to: InputRef {
          node: "join".to_string(),
          input: "i".to_string(),
        },
      },
    );
  }

  #[test]
  fn relations() {
    let relations: Vec<Relation> = relations!();
    assert_eq!(relations, vec![],);

    let relations = relations!(fact(r) -> (i)join);
    assert_eq!(
      relations,
      vec![Relation {
        from: OutputRef {
          node: "fact".to_string(),
          output: "r".to_string(),
        },
        to: InputRef {
          node: "join".to_string(),
          input: "i".to_string(),
        },
      }],
    );

    let relations = relations![fact(r) -> (i)join,
			     a(i)->(b)d,
			     t(y)->(f)e];
    assert_eq!(
      relations,
      vec![
        Relation {
          from: OutputRef {
            node: "fact".to_string(),
            output: "r".to_string(),
          },
          to: InputRef {
            node: "join".to_string(),
            input: "i".to_string(),
          },
        },
        Relation {
          from: OutputRef {
            node: "a".to_string(),
            output: "i".to_string()
          },
          to: InputRef {
            node: "d".to_string(),
            input: "b".to_string()
          }
        },
        Relation {
          from: OutputRef {
            node: "t".to_string(),
            output: "y".to_string()
          },
          to: InputRef {
            node: "e".to_string(),
            input: "f".to_string()
          }
        }
      ],
    );
  }
}
