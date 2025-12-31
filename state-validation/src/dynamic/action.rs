use std::any::Any;

use crate::{StateFilter, ValidAction, dynamic::DynStateFilter};

pub struct DynValidAction<State, Input, Output> {
    filter: DynStateFilter<State, Input, Box<dyn Any>>,
    valid_action: Box<dyn DynAnyClone>,
    action: for<'a> fn(Box<dyn Any>, State, Box<dyn Any>) -> Output,
}
impl<State, Input, Output> std::fmt::Debug for DynValidAction<State, Input, Output> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynValidAction").finish_non_exhaustive()
    }
}

impl<State, Input, Output> DynValidAction<State, Input, Output> {
    pub fn new<T: ValidAction<State, Input, Output = Output> + Clone + 'static>(
        valid_action: T,
    ) -> Self
    where
        <T::Filter as StateFilter<State, Input>>::ValidOutput: 'static,
        <T::Filter as StateFilter<State, Input>>::Error: 'static,
    {
        DynValidAction {
            filter: DynStateFilter::new_with_any_output::<T::Filter>(),
            valid_action: Box::new(valid_action),
            action: |valid_action, state, valid| {
                T::with_valid_input(
                    *valid_action.downcast().unwrap(),
                    state,
                    *valid.downcast().unwrap(),
                )
            },
        }
    }
    pub fn filter(&self) -> &DynStateFilter<State, Input, Box<dyn Any>> {
        &self.filter
    }
    pub fn execute_with_filter(
        self,
        state: State,
        input: Input,
    ) -> Result<Output, DynValidActionExecutionError<State>> {
        match self.filter.filter(&state, input) {
            Ok(v) => Ok((self.action)(self.valid_action, state, v)),
            Err(error) => Err(DynValidActionExecutionError { state, error }),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub struct DynValidActionExecutionError<State> {
    pub state: State,
    #[source]
    pub error: Box<dyn std::error::Error>,
}

impl<State, Input, Output> ValidAction<State, Input> for DynValidAction<State, Input, Output> {
    type Filter = ();
    type Output = Result<Output, DynValidActionExecutionError<State>>;
    fn with_valid_input(self, state: State, input: Input) -> Self::Output {
        self.execute_with_filter(state, input)
    }
}

impl<State, Input, Output> Clone for DynValidAction<State, Input, Output> {
    fn clone(&self) -> Self {
        DynValidAction {
            filter: self.filter.clone(),
            valid_action: (*self.valid_action).any_clone(),
            action: self.action,
        }
    }
}
trait DynAnyClone: Any {
    fn any_clone(&self) -> Box<dyn DynAnyClone>;
}
impl<T: Clone + 'static> DynAnyClone for T {
    fn any_clone(&self) -> Box<dyn DynAnyClone> {
        Box::new(self.clone())
    }
}
