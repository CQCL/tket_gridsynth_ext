use hugr_core::ops::Value;
use tket::extension::rotation::ConstRotation;
use tket::hugr::builder::{Container, DFGBuilder, Dataflow, HugrBuilder};
use tket::hugr::extension::prelude::qb_t;
use tket::hugr::HugrView;
use tket::hugr::types::Signature;
use tket::TketOp;

use tket_gridsynth_ext::apply_gridsynth_pass;
// use tket_gridsynth_ext::lib::apply_gridsynth_pass;

fn main() {
    // Build simple demo hugr containing only Rz gate
    let qb_row = vec![qb_t(); 1];
    let mut h = DFGBuilder::new(Signature::new(qb_row.clone(), qb_row)).unwrap();
    let [q_in] = h.input_wires_arr();
    let constant = h.add_constant(Value::extension(ConstRotation::PI_4));
    let loaded_const = h.load_const(&constant);
    let rz = h.add_dataflow_op(TketOp::Rz, [q_in, loaded_const]).unwrap();
    let _ = h.set_outputs(rz.outputs());
    let mut hugr = h.finish_hugr().unwrap(); 
    hugr.validate().unwrap_or_else(|e| panic!("{e}"));

    // Print result before the gridsynth pass. Paste into a mermaid visualiser to see
    // image of HUGR
    println!("First mermaid string is: ");
    println!("{}", hugr.mermaid_string());

    // Apply the gridsynth pass
    let epsilon = 1e-6;
    apply_gridsynth_pass(&mut hugr, epsilon);

    // Print result after the gridsynth pass. Paste into a mermaid visualiser to see
    // image of HUGR
    println!("Second mermaid string is: ");
    println!("{}", hugr.mermaid_string());
}