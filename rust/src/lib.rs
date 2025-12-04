//! TKET extension crate.
//
// TODO: These docs appear in the landing page of the crate documentation on docs.rs.
// Make sure to update them to reflect the details of your extension.

use hugr_core::hugr::internal::HugrMutInternals;
use hugr_core::ops::constant::CustomConst;
use hugr_core::ops::{Const, Value};
use portgraph::Direction;
use rsgridsynth::config::config_from_theta_epsilon;
use rsgridsynth::gridsynth::gridsynth_gates;
use tket::extension::rotation::ConstRotation;
// use tket::hugr::ops::handle::NodeHandle;
use tket::{Hugr, hugr, op_matches};
use tket::hugr::builder::{Container, DFGBuilder, BuildHandle,Dataflow, HugrBuilder};
use tket::hugr::extension::prelude::{qb_t};
use tket::hugr::HugrView;
use tket::hugr::hugr::hugrmut::HugrMut;
use tket::hugr::{Node, Port};
use tket::hugr::types::Signature;
use tket::TketOp;

/// Find the FuncDefn node for the Rz gate.
fn find_rz(hugr: &mut Hugr) -> Option<tket::hugr::Node> {
    for node in hugr.nodes() {
        let op_type = HugrView::get_optype(hugr, node);
        if op_matches(op_type, TketOp::Rz) {
            return Some(node);
        }
    }
    None
}
// TO DO: extend this function to find all RZ gates

fn find_linked_incoming_ports(hugr: &mut Hugr, node: Node, port_idx: usize) -> Vec<(Node, Port)> {
    let ports = hugr.node_ports(node, Direction::Incoming);
    let collected_ports: Vec<_> = ports.collect();
    let linked_ports = hugr.
        linked_ports(node, collected_ports[port_idx]);
    let linked_ports: Vec<(Node, Port)> = linked_ports.collect();
    linked_ports
}

fn find_linked_outgoing_ports(hugr: &mut Hugr, node: Node, port_idx: usize) -> Vec<(Node, Port)> {
    let ports = hugr.node_ports(node, Direction::Outgoing);
    let collected_ports: Vec<_> = ports.collect();
    let linked_ports = hugr.
        linked_ports(node, collected_ports[port_idx]);
    let linked_ports: Vec<(Node, Port)> = linked_ports.collect();
    linked_ports
}

// fn check_if_angle_node(hugr: &mut Hugr, node: Node) {
//     let op_type = HugrView::get_optype(hugr, node);
//     if op_type.is_const() & op_type.value() == 
// }



// fn follow_path_to_angle

/// Find the constant node containing the angle to be inputted to the Rz gate
fn find_angle_node(hugr: &mut Hugr, rz_node: Node) -> Node {
    // find linked ports to the rz port where the angle will be inputted
    // the port offset of the angle is known to be 1 for the rz gate.
    let linked_ports = find_linked_incoming_ports(hugr, rz_node, 1);
    let mut prev_node = linked_ports[0].0;

    // SHORTCUT: specialise to simple hugrs with LoadConst preceded by const node
    // TO DO: generalise the following
    let linked_ports = find_linked_incoming_ports(hugr, prev_node, 0);
    let angle_node = linked_ports[0].0;
    angle_node

    // recursively follow the path(s) from node to node until a constant node containing
    // the angle is found. 0 is the most likely port to be used, so we'll start with this
    // loop {

    // }

    // for pair in linked_ports {
    //     println!("{}, {}", pair.0.index(), pair.1.index());
    //     }
}

fn find_angle(hugr: &mut Hugr) -> f64 {
    let rz_node = find_rz(hugr).unwrap();
    let angle_node = find_angle_node(hugr, rz_node);
    let op_type = hugr.get_optype(angle_node);
    let angle_const = op_type.as_const().unwrap();
    let angle_val = &angle_const.value;
    let rot: &ConstRotation = angle_val.get_custom_value().unwrap();
    let angle = rot.to_radians();
    angle
}

fn apply_gridsynth(hugr: &mut Hugr) -> String {
    let theta = find_angle(hugr);
    // The following parameters could be made user-specifiable. For simplicity, I fix them, for now
    let epsilon = 1e-10;
    let seed = 1234;
    let verbose = false;
    let mut gridsynth_config = config_from_theta_epsilon(theta, epsilon, seed, verbose);
    let gates = gridsynth_gates(&mut gridsynth_config);
    gates    
}


fn gridsynth_output_to_hugr(gates: &str) -> Hugr {
    let qb_row = vec![qb_t(); 1]; // TO CHECK: will it cause issues to insert new qubit wire?
    let mut h = DFGBuilder::new(Signature::new(qb_row.clone(), qb_row)).unwrap();
    let [q_in] = h.input_wires_arr();

    let mut prev_op = h.input(); 
    for gate in gates.chars() {
        if gate == 'H' {
            prev_op = h.add_dataflow_op(TketOp::H, prev_op.outputs()).unwrap();
        }
        else if gate == 'S' {
            prev_op = h.add_dataflow_op(TketOp::S, prev_op.outputs()).unwrap();
        }
        else if gate == 'T' {
            prev_op = h.add_dataflow_op(TketOp::T, prev_op.outputs()).unwrap();
        }
        else if gate == 'W' {
            break; // Ignoring global phases for now.
        }
    }
    h.set_outputs(prev_op.outputs());
    let mut hugr = h.finish_hugr().unwrap();
    hugr.validate().unwrap_or_else(|e| panic!("{e}"));
    hugr
}

fn destroy_path_to_angle_node(hugr: &mut Hugr, rz_node: Node)  {
    // find linked ports to the rz port where the angle will be inputted
    // the port offset of the angle is known to be 1 for the rz gate.
    let linked_ports = find_linked_incoming_ports(hugr, rz_node, 1);
    let load_const_node = linked_ports[0].0;

    // SHORTCUT: specialise to simple hugrs with LoadConst preceded by const node
    // TO DO: generalise the following
    let linked_ports = find_linked_incoming_ports(hugr, load_const_node, 0);
    let angle_node = linked_ports[0].0;

    hugr.remove_node(load_const_node);
    hugr.remove_node(angle_node);
    // println!("{}", hugr.mermaid_string());
}

/// get previous node that provided qubit to Rz gate and the Rz gate
fn find_qubit_source(hugr: &mut Hugr, rz_node: Node) -> Node {
    let linked_ports = find_linked_incoming_ports(hugr, rz_node, 0);
    let prev_node = linked_ports[0].0;
    prev_node
}

/// Add a gridsynth gate to some previous node, which may or may not be a gridsynth gate, 
/// and connect
fn add_gate_and_connect(hugr: &mut Hugr, prev_node: Node, op: hugr::ops::OpType) -> Node {
    let current_node =  hugr.add_node_after(prev_node, op);
    hugr.add_ports(prev_node, Direction::Outgoing, 1);
    let ports:  Vec<_> = hugr.node_outputs(prev_node).collect();
    // Assuming there were no outgoing ports to begin with when deciding port offset
    let src_port = ports[0];
    let ports:  Vec<_> = hugr.node_inputs(current_node).collect();
    let dst_port = ports[0];
    hugr.connect(prev_node, src_port, current_node, dst_port);
    let prev_node = current_node;
    prev_node
}

fn replace_rz_with_gridsynth_output(hugr: &mut Hugr, rz_node: Node, gates: &str) {
    // getting node that gave qubit to Rz gate
    let mut prev_node = find_qubit_source(hugr, rz_node);

    hugr.remove_node(rz_node);

    println!("Before for loop");
    // recursively adding next gate in gates to prev_node
    for gate in gates.chars() {
        if gate == 'H' {
            prev_node = add_gate_and_connect(hugr, prev_node, TketOp::H.into());
        }
        else if gate == 'S' {
            prev_node = add_gate_and_connect(hugr, prev_node, TketOp::S.into());
            // let prev_node = hugr.add_node_after(prev_node,TketOp::S);
        }
        else if gate == 'T' {
            prev_node = add_gate_and_connect(hugr, prev_node, TketOp::T.into());
            // let prev_node = hugr.add_node_after(prev_node,TketOp::T);
        }
        else if gate == 'W' {
            break; // Ignoring global phases for now.
        }
    }
    println!("{}", hugr.mermaid_string());
    // TO DO: connect nodes and validate
} 
// TO DO: FINISH

/// Replace an Rz gate with the corresponding gates outputted by gridsynth
pub fn apply_gridsynth_pass(hugr: &mut Hugr) {
    let rz_node = find_rz(hugr).unwrap();
    let gates = apply_gridsynth(hugr);
    destroy_path_to_angle_node(hugr, rz_node);
    replace_rz_with_gridsynth_output(hugr, rz_node, &gates);
}


// }
// TO DO: make compatible with Guppy hugrs. Right now, it will only work for simple hugrs not like the 
// ones that guppy produces

// pub fn gridsynth_pass(hugr: &mut Hugr) {

// }

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

    use hugr_core::PortIndex;
    use tket::Hugr;
    use tket::hugr::NodeIndex;
    use tket::hugr::ops::FuncDefn;
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
        // TO DO: rename test once its usage is settled
        let qb_row = vec![qb_t(); 1];
        let mut h = DFGBuilder::new(Signature::new(qb_row.clone(), qb_row)).unwrap();
        let [q_in] = h.input_wires_arr();

        let constant = h.add_constant(Value::extension(ConstRotation::PI_4));
        let loaded_const = h.load_const(&constant);
        let rz = h.add_dataflow_op(TketOp::Rz, [q_in, loaded_const]).unwrap();
        let _ = h.set_outputs(rz.outputs());
        let mut circ = h.finish_hugr().unwrap(); //(rz.outputs()).unwrap().into();
        // println!("{}", circ.mermaid_string());
        let rz_node = find_rz(&mut circ).unwrap();

        // for figuring out how to access ports

        // let rz_ports = circ
        //     .linked_ports(rz_node, circ.node_ports(rz_node, Direction.Incoming));
        // for port in rz_ports
        //     println!("{}", port);
        assert_eq!(rz_node.index(), 9); // index 9 gleaned from manual inspection of hugr

        // testing that I can find prev port
        let linked_ports = find_linked_incoming_ports(&mut circ, rz_node, 1);
        let mut prev_node = linked_ports[0].0;
        println!("{}", prev_node.index());
        // let linked_ports = find_linked_incoming_ports(&mut circ, rz_node);
        // for tup in linked_ports {
        //     println!("{}, {}", tup.0.index(), tup.1.index());
        // }
        let angle = find_angle(&mut circ);
        println!("The angle is: {}", angle);

        let gates = apply_gridsynth(&mut circ);
        println!("{}", &gates);        
        apply_gridsynth_pass(&mut circ);
    }
}
