use std::{collections::HashMap, rc::Rc};

use halo2_proofs::{circuit::Layouter, halo2curves::FieldExt, plonk::Error};
use ndarray::{Array, IxDyn};

use crate::gadgets::{
  gadget::{Gadget, GadgetConfig, GadgetType},
  max::MaxChip,
};

use super::layer::{AssignedTensor, CellRc, GadgetConsumer, Layer, LayerConfig};

pub struct MaxPool2DChip<F: FieldExt> {
  pub marker: std::marker::PhantomData<F>,
}

impl<F: FieldExt> MaxPool2DChip<F> {
  pub fn splat(
    &self,
    inp: &AssignedTensor<F>,
    layer_config: &LayerConfig,
  ) -> Result<Vec<Vec<CellRc<F>>>, Error> {
    let params = &layer_config.layer_params;
    let (fx, fy) = (params[0], params[1]);
    let (fx, fy) = (fx as usize, fy as usize);
    let (sx, sy) = (params[2], params[3]);
    let (sx, sy) = (sx as usize, sy as usize);

    // Only support batch size 1 for now
    assert_eq!(inp.shape()[0], 1);

    let mut splat = vec![];
    for i in 0..inp.shape()[1] / sx {
      for j in 0..inp.shape()[2] / sy {
        for k in 0..inp.shape()[3] {
          let mut tmp = vec![];
          for x in 0..fx {
            for y in 0..fy {
              let x = i * sx + x;
              let y = j * sy + y;
              if x < inp.shape()[1] && y < inp.shape()[2] {
                tmp.push(inp[[0, x, y, k]].clone());
              }
            }
          }
          splat.push(tmp);
        }
      }
    }

    Ok(splat)
  }
}

impl<F: FieldExt> Layer<F> for MaxPool2DChip<F> {
  fn forward(
    &self,
    mut layouter: impl Layouter<F>,
    tensors: &Vec<AssignedTensor<F>>,
    _constants: &HashMap<i64, CellRc<F>>,
    gadget_config: Rc<GadgetConfig>,
    layer_config: &LayerConfig,
  ) -> Result<Vec<AssignedTensor<F>>, Error> {
    let inp = &tensors[0];
    let splat = self.splat(inp, layer_config).unwrap();

    let max_chip = MaxChip::<F>::construct(gadget_config.clone());
    let mut out = vec![];
    for i in 0..splat.len() {
      let inps = &splat[i];
      let inps = inps.iter().map(|x| x.as_ref()).collect();
      let max = max_chip
        .forward(
          layouter.namespace(|| format!("max {}", i)),
          &vec![inps],
          &vec![],
        )
        .unwrap();
      out.push(max[0].clone());
    }
    let out = out.into_iter().map(|x| Rc::new(x)).collect();

    // TODO: refactor this
    let (sx, sy) = (layer_config.layer_params[2], layer_config.layer_params[3]);
    let (sx, sy) = (sx as usize, sy as usize);
    let out_shape = vec![1, inp.shape()[1] / sx, inp.shape()[2] / sy, inp.shape()[3]];

    let out = Array::from_shape_vec(IxDyn(&out_shape), out).unwrap();

    Ok(vec![out])
  }
}

impl<F: FieldExt> GadgetConsumer for MaxPool2DChip<F> {
  fn used_gadgets(&self) -> Vec<GadgetType> {
    vec![GadgetType::Max]
  }
}