#[macro_export]
macro_rules! rasen_graph {
    (@edge $graph:expr, $index:expr, $dest:ident, $origin:ident ) => {
        $graph.add_edge($origin, $dest, $index);
    };

    (@edge $graph:expr, $index:expr, $dest:ident, $origin:ident, $( $rest:ident ),* ) => {
        rasen_graph!(@edge $graph, $index, $dest, $origin);
        rasen_graph!(@edge $graph, $index + 1, $dest, $( $rest ),* );
    };

    ( nodes { $( $id:ident = $val:expr ),* } edges { $( $to:ident ( $( $from:ident ),* ) ),* } ) => {
        {
            let mut graph = $crate::graph::Graph::new();

            $(
              let $id = graph.add_node($val);
            )*

            $(
                rasen_graph!(@edge graph, 0, $to, $( $from ),* );
            )*

            graph
        }
    };
}
