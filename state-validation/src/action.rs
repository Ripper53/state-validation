use crate::{StateFilter, StateFilterInput};

pub trait ValidAction<State, Input: StateFilterInput> {
    type Filter: StateFilter<State, Input>;
    type Output;
    fn with_valid_input(
        self,
        state: State,
        valid: <Self::Filter as StateFilter<State, Input>>::ValidOutput,
    ) -> Self::Output;
}
