//! Exposes a simple DSL for the construction of a data-flow graph for rasen
//!
//! ```
//! # extern crate rasen;
//! # #[macro_use] extern crate rasen_dsl;
//! # use rasen::*;
//! # use rasen::TypedValue::*;
//! # fn main() {
//! let graph = rasen_graph! {
//!     // Keep some values around with bindings
//!     input = Input(0, TypeName::VEC3);
//!     uniform = Uniform(0, TypeName::VEC3);
//!
//!     sum = Add {
//!         uniform
//!         Constant(Vec3(1.0, 0.0, 0.5))
//!     };
//!
//!     // Write to an output
//!     Output(0, TypeName::VEC3) {
//!         Multiply {
//!             input
//!             sum
//!             Input(1, TypeName::VEC3)
//!         }
//!     };
//! };
//! # }
//! ```

#[macro_export]
macro_rules! rasen_graph {
    // Internal resolution
    (@args $graph:expr ; $node:expr ; $id:expr ; $sink:ident ( $( $arg:expr ),* ) { $( $val:tt )* } $( $rest:tt )* ) => {
        let sink = $graph.add_node(Node::$sink( $( $arg ),* ));
        $graph.add_edge(sink, $node, $id);

        rasen_graph!(@args $graph; sink; 0; $( $val )* );

        rasen_graph!(@args $graph; $node; $id + 1; $( $rest )* );
    };

    (@args $graph:expr ; $node:expr ; $id:expr ; $sink:ident ( $( $arg:expr ),* ) $( $rest:tt )* ) => {
        let sink = $graph.add_node(Node::$sink( $( $arg ),* ));
        $graph.add_edge(sink, $node, $id);

        rasen_graph!(@args $graph; $node; $id + 1; $( $rest )* );
    };

    (@args $graph:expr ; $node:expr ; $id:expr ; $sink:ident { $( $val:tt )* } $( $rest:tt )* ) => {
        let sink = $graph.add_node(Node::$sink);
        $graph.add_edge(sink, $node, $id);

        rasen_graph!(@args $graph; sink; 0; $( $val )* );

        rasen_graph!(@args $graph; $node; $id + 1; $( $rest )* );
    };

    (@args $graph:expr ; $node:expr ; $id:expr ; $sink:ident $( $rest:tt )* ) => {
        $graph.add_edge($sink, $node, $id);

        rasen_graph!(@args $graph; $node; $id + 1; $( $rest )* );
    };

    (@args $graph:expr ; $node:expr ; $id:expr ; ) => {};

    // Top level
    (@top $graph:expr ; $bind:ident = $sink:ident { $( $val:tt )* } ; $( $rest:tt )* ) => {
        let $bind = $graph.add_node(Node::$sink);
        rasen_graph!(@args $graph; $bind; 0; $( $val )* );
        rasen_graph!(@top $graph; $( $rest )* );
    };

    (@top $graph:expr; $bind:ident = $sink:ident ( $( $arg:expr ),* ) ; $( $rest:tt )* ) => {
        let $bind = $graph.add_node(Node::$sink( $( $arg ),* ));
        rasen_graph!(@top $graph; $( $rest )* );
    };

    (@top $graph:expr; $sink:ident ( $( $arg:expr ),* ) { $( $val:tt )* } ; $( $rest:tt )* ) => {
        let node = $graph.add_node(Node::$sink( $( $arg ),* ));
        rasen_graph!(@args $graph; node; 0; $( $val )* );

        rasen_graph!(@top $graph; $( $rest )* );
    };

    (@top $graph:expr; ) => {};

    // Context
    ( $( $body:tt )+ ) => {
        {
            let mut graph = ::rasen::graph::Graph::default();
            rasen_graph!(@top graph; $( $body )* );
            graph
        }
    };
}
