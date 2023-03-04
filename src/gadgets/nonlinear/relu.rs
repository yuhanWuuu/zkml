use std::{collections::HashMap, marker::PhantomData, rc::Rc};

use halo2_proofs::{
  circuit::{AssignedCell, Layouter, Region},
  halo2curves::FieldExt,
  plonk::{ConstraintSystem, Error},
};

use super::{
  super::gadget::{Gadget, GadgetConfig, GadgetType},
  non_linearity::NonLinearGadget,
};

pub struct ReluChip<F: FieldExt> {
  config: Rc<GadgetConfig>,
  _marker: PhantomData<F>,
}

impl<F: FieldExt> ReluChip<F> {
  pub fn construct(config: Rc<GadgetConfig>) -> Self {
    Self {
      config,
      _marker: PhantomData,
    }
  }

  pub fn configure(meta: &mut ConstraintSystem<F>, gadget_config: GadgetConfig) -> GadgetConfig {
    <ReluChip<F> as NonLinearGadget<F>>::configure(meta, gadget_config, GadgetType::Relu)
  }
}

impl<F: FieldExt> NonLinearGadget<F> for ReluChip<F> {
  fn generate_map(_scale_factor: u64, min_val: i64, max_val: i64) -> HashMap<i64, i64> {
    let range = max_val - min_val;

    let mut map = HashMap::new();
    for i in 0..range {
      let shifted = i + min_val;
      let relu = shifted.max(0);
      map.insert(i as i64, relu);
    }

    map
  }

  fn get_map(&self) -> &HashMap<i64, i64> {
    &self.config.maps.get(&GadgetType::Relu).unwrap()[0]
  }

  fn get_selector(&self) -> halo2_proofs::plonk::Selector {
    self.config.selectors.get(&GadgetType::Relu).unwrap()[0]
  }
}

impl<F: FieldExt> Gadget<F> for ReluChip<F> {
  fn name(&self) -> String {
    "Relu".to_string()
  }

  fn num_cols_per_op(&self) -> usize {
    <ReluChip<F> as NonLinearGadget<F>>::num_cols_per_op()
  }

  fn num_inputs_per_row(&self) -> usize {
    self.config.columns.len() / self.num_cols_per_op()
  }

  fn num_outputs_per_row(&self) -> usize {
    self.config.columns.len() / self.num_cols_per_op()
  }

  fn load_lookups(&self, layouter: impl Layouter<F>) -> Result<(), Error> {
    NonLinearGadget::load_lookups(self, layouter, self.config.clone(), GadgetType::Relu)?;
    Ok(())
  }

  fn op_row_region(
    &self,
    region: &mut Region<F>,
    row_offset: usize,
    vec_inputs: &Vec<Vec<&AssignedCell<F, F>>>,
    single_inputs: &Vec<&AssignedCell<F, F>>,
  ) -> Result<Vec<AssignedCell<F, F>>, Error> {
    NonLinearGadget::op_row_region(
      self,
      region,
      row_offset,
      vec_inputs,
      single_inputs,
      self.config.clone(),
    )
  }
}