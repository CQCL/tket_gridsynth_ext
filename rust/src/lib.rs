//! TKET extension crate.
//
// TODO: These docs appear in the landing page of the crate documentation on docs.rs.
// Make sure to update them to reflect the details of your extension.

// use tket::hugr::ops::handle::NodeHandle;
use tket::{Hugr, op_matches};
use tket::hugr::HugrView;
use tket::hugr::hugr::hugrmut::HugrMut;
use tket::TketOp;

/// Find the FuncDefn node for the Rz gate
pub fn find_rz_defn(hugr: &mut Hugr) -> Option<tket::hugr::Node> {
    for node in hugr.nodes() {
        let op_type = HugrView::get_optype(hugr, node);
        if op_matches(op_type, TketOp::Rz) {
            return Some(node);
        }
    }
    None
}


/// Example function.
///
/// Takes a Hugr and removes every node from it, except the module root.
pub fn example_remove_contents(hugr: &mut Hugr) -> Result<(), ExampleError> {
    if hugr.num_nodes() == 1 {
        return Err(ExampleError {
            message: "Hugr is already empty".to_string(),
        });
    }
    hugr.set_entrypoint(hugr.module_root());

    while let Some(node) = hugr.first_child(hugr.module_root()) {
        hugr.remove_subtree(node);
    }

    hugr.validate().unwrap_or_else(|e| panic!("{e}"));

    Ok(())
}

/// Example error.
#[derive(Debug, derive_more::Display, derive_more::Error)]
#[display("Example error: {message}")]
pub struct ExampleError {
    message: String,
}

// Test of example function
#[cfg(test)]
mod tests {
    use super::*;

    use tket::Hugr;
    use tket::hugr::NodeIndex;
    use tket::hugr::builder::{Container, DFGBuilder, Dataflow, HugrBuilder};
    use tket::hugr::extension::prelude::{qb_t};
    use tket::hugr::ops::FuncDefn;
    use tket::hugr::types::Signature;
    use tket::hugr::ops::Value;
    use tket::extension::rotation::ConstRotation;

    #[test]
    fn it_works() {
        let mut hugr =
            Hugr::new_with_entrypoint(FuncDefn::new("test", Signature::new(vec![], vec![])))
                .unwrap();

        example_remove_contents(&mut hugr).unwrap();
    }

    #[test]
    fn finds_rz_func_defn() {
        let qb_row = vec![qb_t(); 1];
        let mut h = DFGBuilder::new(Signature::new(qb_row.clone(), qb_row)).unwrap();
        let [q_in] = h.input_wires_arr();

        let constant = h.add_constant(Value::extension(ConstRotation::PI_2));
        let loaded_const = h.load_const(&constant);
        let rz = h.add_dataflow_op(TketOp::Rz, [q_in, loaded_const]).unwrap();
        let _ = h.set_outputs(rz.outputs());
        let mut circ = h.finish_hugr().unwrap(); //(rz.outputs()).unwrap().into();
        println!("{}", circ.mermaid_string());
        let defn_node = find_rz_defn(&mut circ).unwrap();
        assert_eq!(defn_node.index(), 9); // index 9 gleaned from manual inspection of hugr


        // let mut dfg_builder = DFGBuilder::new(inout_sig(
        //     vec![qb_t()],
        //     vec![qb_t(), bool_t()]
        // )).unwrap();
        // // Get the wire  
        // let [wire] = dfg_builder.input_wires_arr();

        // let constant = dfg_builder.add_constant(Value::extension(ConstRotation::PI_2));
        // let loaded_const = dfg_builder.load_const(&constant);
        // let rz = dfg_builder.add_dataflow_op(TketOp::Rz, [wire, loaded_const]).unwrap();
        // let circ = dfg_builder.finish_hugr_with_outputs(rz.outputs()).unwrap().into();
        
    }
}
