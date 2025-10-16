use crate::StateFilter;

pub struct CollectedInputs<State, Inputs: Iterator> {
    inputs: Inputs,
    _m: std::marker::PhantomData<State>,
}

impl<State, Inputs: Iterator> CollectedInputs<State, Inputs> {
    pub fn new(inputs: Inputs) -> Self {
        CollectedInputs {
            inputs,
            _m: std::marker::PhantomData::default(),
        }
    }
    /// Do all the inputs pass the filter without error?
    pub fn fits_all<F: StateFilter<State, Inputs::Item>>(self, state: &State) -> bool {
        self.inputs
            .into_iter()
            .all(|input| F::filter(state, input).is_ok())
    }
    /// Do any of the inputs pass the filter without error?
    pub fn fits_any<F: StateFilter<State, Inputs::Item>>(self, state: &State) -> bool {
        self.inputs
            .into_iter()
            .any(|input| F::filter(state, input).is_ok())
    }
    /// Iterator for the ouputs of the inputs that pass the filter without error.
    pub fn fits_iter<F: StateFilter<State, Inputs::Item>>(
        self,
        state: &State,
    ) -> impl Iterator<Item = F::ValidOutput> {
        self.inputs
            .into_iter()
            .filter_map(|input| F::filter(state, input).ok())
    }
}

pub trait InputCollector<State, Input> {
    fn collect_inputs(state: &State) -> CollectedInputs<State, impl Iterator<Item = Input>>;
}
