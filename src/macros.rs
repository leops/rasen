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

    (@args $graph:expr ; $node:expr ; $id:expr ; $sink:ident ) => {
        $graph.add_edge($sink, $node, $id);
    };

    (@args $graph:expr ; $node:expr ; $id:expr ; ) => {};

    // Top level
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
            let mut graph = $crate::graph::Graph::new();
            rasen_graph!(@top graph; $( $body )* );
            graph
        }
    };
}
