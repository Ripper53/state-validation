use crate::StateFilter;

pub struct DynStateFilter<State, Input, ValidOutput>(
    for<'a> fn(&State, Input) -> Result<ValidOutput, Box<dyn std::error::Error>>,
);

impl<State, Input, ValidOutput> DynStateFilter<State, Input, ValidOutput> {
    pub fn new<T: StateFilter<State, Input>>() -> Self
    where
        T::ValidOutput: Into<ValidOutput>,
        T::Error: 'static,
    {
        DynStateFilter(|state, input| match T::filter(state, input) {
            Ok(v) => Ok(v.into()),
            Err(e) => Err(Box::new(e)),
        })
    }
    pub fn filter(
        &self,
        state: &State,
        input: Input,
    ) -> Result<ValidOutput, Box<dyn std::error::Error>> {
        (self.0)(state, input)
    }
}
impl<State, Input> DynStateFilter<State, Input, Box<dyn std::any::Any>> {
    pub fn new_with_any_output<T: StateFilter<State, Input>>() -> Self
    where
        T::ValidOutput: 'static,
        T::Error: 'static,
    {
        DynStateFilter(|state, input| match T::filter(state, input) {
            Ok(v) => Ok(Box::new(v)),
            Err(e) => Err(Box::new(e)),
        })
    }
}

impl<State, Input, ValidOutput> Clone for DynStateFilter<State, Input, ValidOutput> {
    fn clone(&self) -> Self {
        DynStateFilter(self.0)
    }
}
