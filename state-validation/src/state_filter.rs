use crate::Condition;

pub trait StateFilter<State, Input> {
    type ValidOutput;
    type Error: std::error::Error;
    fn filter(state: &State, value: Input) -> Result<Self::ValidOutput, Self::Error>;
}
impl<State, Input> StateFilter<State, Input> for () {
    type ValidOutput = Input;
    type Error = std::convert::Infallible;
    fn filter(_state: &State, input: Input) -> Result<Self::ValidOutput, Self::Error> {
        Ok(input)
    }
}
impl<
    State,
    InitialInput,
    Input,
    F: StateFilter<State, Input>,
> StateFilter<State, InitialInput> for Condition<Input, F>
where
    InitialInput: StateFilterInputConversion<Input>,
    <InitialInput as StateFilterInputConversion<Input>>::Remainder:
        StateFilterInputCombination<F::ValidOutput>,
{
    type ValidOutput =
        <<InitialInput as StateFilterInputConversion<Input>>::Remainder as StateFilterInputCombination<
            F::ValidOutput,
        >>::Combined;
    type Error = F::Error;
    fn filter(state: &State, value: InitialInput) -> Result<Self::ValidOutput, Self::Error> {
        let (input, remainder) = value.split_take();
        F::filter(state, input).map(|v| remainder.combine(v))
    }
}
impl<
    State,
    InitialInput,
    Input0,
    Input1,
    F0: StateFilter<State, Input0>,
    F1: StateFilter<State, Input1>,
> StateFilter<State, InitialInput> for (Condition<Input0, F0>, Condition<Input1, F1>)
where
    InitialInput: StateFilterInputConversion<Input0>,
    <InitialInput as StateFilterInputConversion<Input0>>::Remainder:
        StateFilterInputCombination<F0::ValidOutput>,
    <<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined: StateFilterInputConversion<Input1>,
    <<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder:
        StateFilterInputCombination<F1::ValidOutput>,
{
    type ValidOutput = <<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
        >>::Combined as StateFilterInputConversion<Input1>>::Remainder as
        StateFilterInputCombination<F1::ValidOutput>>::Combined;
    type Error = StateFilterTwoChainError<F0::Error, F1::Error>;
    fn filter(state: &State, value: InitialInput) -> Result<Self::ValidOutput, Self::Error> {
        let (input, remainder) = value.split_take();
        F0::filter(state, input)
            .map(|v| remainder.combine(v))
            .map_err(|e| StateFilterTwoChainError::Filter0(e))
            .and_then(|v| {
                let (input, remainder) = v.split_take();
                F1::filter(state, input).map(|v| remainder.combine(v))
            .map_err(|e| StateFilterTwoChainError::Filter1(e))
            })
    }
}
#[derive(thiserror::Error, Debug)]
pub enum StateFilterTwoChainError<E0: std::error::Error, E1: std::error::Error> {
    #[error(transparent)]
    Filter0(E0),
    #[error(transparent)]
    Filter1(E1),
}
impl<
    State,
    InitialInput,
    Input0,
    Input1,
    Input2,
    F0: StateFilter<State, Input0>,
    F1: StateFilter<State, Input1>,
    F2: StateFilter<State, Input2>,
> StateFilter<State, InitialInput>
    for (
        Condition<Input0, F0>,
        Condition<Input1, F1>,
        Condition<Input2, F2>,
    )
where
    InitialInput: StateFilterInputConversion<Input0>,
    <InitialInput as StateFilterInputConversion<Input0>>::Remainder:
        StateFilterInputCombination<F0::ValidOutput>,
    <<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined: StateFilterInputConversion<Input1>,
    <<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder:
        StateFilterInputCombination<F1::ValidOutput>,
    <<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined: StateFilterInputConversion<Input2>,

    <<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder:
        StateFilterInputCombination<F2::ValidOutput>,
{
    type ValidOutput = 
    <<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined
    ;
    type Error = StateFilterThreeChainError<F0::Error, F1::Error, F2::Error>;
    fn filter(state: &State, value: InitialInput) -> Result<Self::ValidOutput, Self::Error> {
        let (input, remainder) = value.split_take();
        F0::filter(state, input)
            .map(|v| remainder.combine(v))
            .map_err(|e| StateFilterThreeChainError::Filter0(e))
            .and_then(|v| {
                let (input, remainder) = v.split_take();
                F1::filter(state, input).map(|v| remainder.combine(v))
            .map_err(|e| StateFilterThreeChainError::Filter1(e))
            })
            .and_then(|v| {
                let (input, remainder) = v.split_take();
                F2::filter(state, input).map(|v| remainder.combine(v))
            .map_err(|e| StateFilterThreeChainError::Filter2(e))
            })
    }
}
#[derive(thiserror::Error, Debug)]
pub enum StateFilterThreeChainError<E0: std::error::Error, E1: std::error::Error, E2: std::error::Error> {
    #[error(transparent)]
    Filter0(E0),
    #[error(transparent)]
    Filter1(E1),
    #[error(transparent)]
    Filter2(E2),
}
impl<
    State,
    InitialInput,
    Input0,
    Input1,
    Input2,
    Input3,
    F0: StateFilter<State, Input0>,
    F1: StateFilter<State, Input1>,
    F2: StateFilter<State, Input2>,
    F3: StateFilter<State, Input3>,
> StateFilter<State, InitialInput>
    for (
        Condition<Input0, F0>,
        Condition<Input1, F1>,
        Condition<Input2, F2>,
        Condition<Input3, F3>,
    )
where
    InitialInput: StateFilterInputConversion<Input0>,
    <InitialInput as StateFilterInputConversion<Input0>>::Remainder:
        StateFilterInputCombination<F0::ValidOutput>,
    <<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined: StateFilterInputConversion<Input1>,
    <<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder:
        StateFilterInputCombination<F1::ValidOutput>,
    <<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined: StateFilterInputConversion<Input2>,

    <<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder:
        StateFilterInputCombination<F2::ValidOutput>,
        
        
    <<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined: StateFilterInputConversion<Input3>,

    <<<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined as StateFilterInputConversion<Input3>>::Remainder: StateFilterInputCombination<F3::ValidOutput>,
{
    type ValidOutput = 
    <<<<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined as StateFilterInputConversion<Input3>>::Remainder as StateFilterInputCombination<F3::ValidOutput>>::Combined
    ;
    type Error = StateFilterFourChainError<F0::Error, F1::Error, F2::Error, F3::Error>;
    fn filter(state: &State, value: InitialInput) -> Result<Self::ValidOutput, Self::Error> {
        let (input, remainder) = value.split_take();
        F0::filter(state, input)
            .map(|v| remainder.combine(v))
            .map_err(|e| StateFilterFourChainError::Filter0(e))
            .and_then(|v| {
                let (input, remainder) = v.split_take();
                F1::filter(state, input).map(|v| remainder.combine(v))
            .map_err(|e| StateFilterFourChainError::Filter1(e))
            })
            .and_then(|v| {
                let (input, remainder) = v.split_take();
                F2::filter(state, input).map(|v| remainder.combine(v))
            .map_err(|e| StateFilterFourChainError::Filter2(e))
            })
            .and_then(|v| {
                let (input, remainder) = v.split_take();
                F3::filter(state, input).map(|v| remainder.combine(v))
            .map_err(|e| StateFilterFourChainError::Filter3(e))
            })
    }
}
#[derive(thiserror::Error, Debug)]
pub enum StateFilterFourChainError<E0: std::error::Error, E1: std::error::Error, E2: std::error::Error, E3: std::error::Error> {
    #[error(transparent)]
    Filter0(E0),
    #[error(transparent)]
    Filter1(E1),
    #[error(transparent)]
    Filter2(E2),
    #[error(transparent)]
    Filter3(E3),
}
impl<
    State,
    InitialInput,
    Input0,
    Input1,
    Input2,
    Input3,
    Input4,
    F0: StateFilter<State, Input0>,
    F1: StateFilter<State, Input1>,
    F2: StateFilter<State, Input2>,
    F3: StateFilter<State, Input3>,
    F4: StateFilter<State, Input4>,
> StateFilter<State, InitialInput>
    for (
        Condition<Input0, F0>,
        Condition<Input1, F1>,
        Condition<Input2, F2>,
        Condition<Input3, F3>,
        Condition<Input4, F4>,
    )
where
    InitialInput: StateFilterInputConversion<Input0>,
    <InitialInput as StateFilterInputConversion<Input0>>::Remainder:
        StateFilterInputCombination<F0::ValidOutput>,
    <<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined: StateFilterInputConversion<Input1>,
    <<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder:
        StateFilterInputCombination<F1::ValidOutput>,
    <<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined: StateFilterInputConversion<Input2>,

    <<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder:
        StateFilterInputCombination<F2::ValidOutput>,
        
        
    <<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined: StateFilterInputConversion<Input3>,

    <<<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined as StateFilterInputConversion<Input3>>::Remainder: StateFilterInputCombination<F3::ValidOutput>,

    <<<<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined as StateFilterInputConversion<Input3>>::Remainder as StateFilterInputCombination<F3::ValidOutput>>::Combined: StateFilterInputConversion<Input4>,

    <<<<<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined as StateFilterInputConversion<Input3>>::Remainder as StateFilterInputCombination<F3::ValidOutput>>::Combined as StateFilterInputConversion<Input4>>::Remainder: StateFilterInputCombination<F4::ValidOutput>,
{
    type ValidOutput =
    <<<<<<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined as StateFilterInputConversion<Input3>>::Remainder as StateFilterInputCombination<F3::ValidOutput>>::Combined as StateFilterInputConversion<Input4>>::Remainder as StateFilterInputCombination<F4::ValidOutput>>::Combined
    ;
    type Error = StateFilterFiveChainError<F0::Error, F1::Error, F2::Error, F3::Error, F4::Error>;
    fn filter(state: &State, value: InitialInput) -> Result<Self::ValidOutput, Self::Error> {
        let (input, remainder) = value.split_take();
        F0::filter(state, input)
            .map(|v| remainder.combine(v))
            .map_err(|e| StateFilterFiveChainError::Filter0(e))
            .and_then(|v| {
                let (input, remainder) = v.split_take();
                F1::filter(state, input).map(|v| remainder.combine(v))
            .map_err(|e| StateFilterFiveChainError::Filter1(e))
            })
            .and_then(|v| {
                let (input, remainder) = v.split_take();
                F2::filter(state, input).map(|v| remainder.combine(v))
            .map_err(|e| StateFilterFiveChainError::Filter2(e))
            })
            .and_then(|v| {
                let (input, remainder) = v.split_take();
                F3::filter(state, input).map(|v| remainder.combine(v))
            .map_err(|e| StateFilterFiveChainError::Filter3(e))
            })
            .and_then(|v| {
                let (input, remainder) = v.split_take();
                F4::filter(state, input).map(|v| remainder.combine(v))
            .map_err(|e| StateFilterFiveChainError::Filter4(e))
            })
    }
}
#[derive(thiserror::Error, Debug)]
pub enum StateFilterFiveChainError<E0: std::error::Error, E1: std::error::Error, E2: std::error::Error, E3: std::error::Error, E4: std::error::Error> {
    #[error(transparent)]
    Filter0(E0),
    #[error(transparent)]
    Filter1(E1),
    #[error(transparent)]
    Filter2(E2),
    #[error(transparent)]
    Filter3(E3),
    #[error(transparent)]
    Filter4(E4),
}
impl<
    State,
    InitialInput,
    Input0,
    Input1,
    Input2,
    Input3,
    Input4,
    Input5,
    F0: StateFilter<State, Input0>,
    F1: StateFilter<State, Input1>,
    F2: StateFilter<State, Input2>,
    F3: StateFilter<State, Input3>,
    F4: StateFilter<State, Input4>,
    F5: StateFilter<State, Input5>,
> StateFilter<State, InitialInput>
    for (
        Condition<Input0, F0>,
        Condition<Input1, F1>,
        Condition<Input2, F2>,
        Condition<Input3, F3>,
        Condition<Input4, F4>,
        Condition<Input5, F5>,
    )
where
    InitialInput: StateFilterInputConversion<Input0>,
    <InitialInput as StateFilterInputConversion<Input0>>::Remainder:
        StateFilterInputCombination<F0::ValidOutput>,
    <<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined: StateFilterInputConversion<Input1>,
    <<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder:
        StateFilterInputCombination<F1::ValidOutput>,
    <<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined: StateFilterInputConversion<Input2>,

    <<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder:
        StateFilterInputCombination<F2::ValidOutput>,
        
        
    <<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined: StateFilterInputConversion<Input3>,

    <<<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined as StateFilterInputConversion<Input3>>::Remainder: StateFilterInputCombination<F3::ValidOutput>,

    <<<<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined as StateFilterInputConversion<Input3>>::Remainder as StateFilterInputCombination<F3::ValidOutput>>::Combined: StateFilterInputConversion<Input4>,

    <<<<<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined as StateFilterInputConversion<Input3>>::Remainder as StateFilterInputCombination<F3::ValidOutput>>::Combined as StateFilterInputConversion<Input4>>::Remainder: StateFilterInputCombination<F4::ValidOutput>,

    <<<<<<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined as StateFilterInputConversion<Input3>>::Remainder as StateFilterInputCombination<F3::ValidOutput>>::Combined as StateFilterInputConversion<Input4>>::Remainder as StateFilterInputCombination<F4::ValidOutput>>::Combined: StateFilterInputConversion<Input5>,

    <<<<<<<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined as StateFilterInputConversion<Input3>>::Remainder as StateFilterInputCombination<F3::ValidOutput>>::Combined as StateFilterInputConversion<Input4>>::Remainder as StateFilterInputCombination<F4::ValidOutput>>::Combined as StateFilterInputConversion<Input5>>::Remainder: StateFilterInputCombination<F5::ValidOutput>,
{
    type ValidOutput = 
    <<<<<<<<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined as StateFilterInputConversion<Input3>>::Remainder as StateFilterInputCombination<F3::ValidOutput>>::Combined as StateFilterInputConversion<Input4>>::Remainder as StateFilterInputCombination<F4::ValidOutput>>::Combined as StateFilterInputConversion<Input5>>::Remainder as StateFilterInputCombination<F5::ValidOutput>>::Combined
    ;
    type Error = StateFilterSixChainError<F0::Error, F1::Error, F2::Error, F3::Error, F4::Error, F5::Error>;
    fn filter(state: &State, value: InitialInput) -> Result<Self::ValidOutput, Self::Error> {
        let (input, remainder) = value.split_take();
        F0::filter(state, input)
            .map(|v| remainder.combine(v))
            .map_err(|e| StateFilterSixChainError::Filter0(e))
            .and_then(|v| {
                let (input, remainder) = v.split_take();
                F1::filter(state, input).map(|v| remainder.combine(v))
            .map_err(|e| StateFilterSixChainError::Filter1(e))
            })
            .and_then(|v| {
                let (input, remainder) = v.split_take();
                F2::filter(state, input).map(|v| remainder.combine(v))
            .map_err(|e| StateFilterSixChainError::Filter2(e))
            })
            .and_then(|v| {
                let (input, remainder) = v.split_take();
                F3::filter(state, input).map(|v| remainder.combine(v))
            .map_err(|e| StateFilterSixChainError::Filter3(e))
            })
            .and_then(|v| {
                let (input, remainder) = v.split_take();
                F4::filter(state, input).map(|v| remainder.combine(v))
            .map_err(|e| StateFilterSixChainError::Filter4(e))
            })
            .and_then(|v| {
                let (input, remainder) = v.split_take();
                F5::filter(state, input).map(|v| remainder.combine(v))
            .map_err(|e| StateFilterSixChainError::Filter5(e))
            })
    }
}
#[derive(thiserror::Error, Debug)]
pub enum StateFilterSixChainError<E0: std::error::Error, E1: std::error::Error, E2: std::error::Error, E3: std::error::Error, E4: std::error::Error, E5: std::error::Error> {
    #[error(transparent)]
    Filter0(E0),
    #[error(transparent)]
    Filter1(E1),
    #[error(transparent)]
    Filter2(E2),
    #[error(transparent)]
    Filter3(E3),
    #[error(transparent)]
    Filter4(E4),
    #[error(transparent)]
    Filter5(E5),
}
impl<
    State,
    InitialInput,
    Input0,
    Input1,
    Input2,
    Input3,
    Input4,
    Input5,
    Input6,
    F0: StateFilter<State, Input0>,
    F1: StateFilter<State, Input1>,
    F2: StateFilter<State, Input2>,
    F3: StateFilter<State, Input3>,
    F4: StateFilter<State, Input4>,
    F5: StateFilter<State, Input5>,
    F6: StateFilter<State, Input6>,
> StateFilter<State, InitialInput>
    for (
        Condition<Input0, F0>,
        Condition<Input1, F1>,
        Condition<Input2, F2>,
        Condition<Input3, F3>,
        Condition<Input4, F4>,
        Condition<Input5, F5>,
        Condition<Input6, F6>,
    )
where
    InitialInput: StateFilterInputConversion<Input0>,
    <InitialInput as StateFilterInputConversion<Input0>>::Remainder:
        StateFilterInputCombination<F0::ValidOutput>,
    <<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined: StateFilterInputConversion<Input1>,
    <<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder:
        StateFilterInputCombination<F1::ValidOutput>,
    <<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined: StateFilterInputConversion<Input2>,

    <<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder:
        StateFilterInputCombination<F2::ValidOutput>,
        
        
    <<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined: StateFilterInputConversion<Input3>,

    <<<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined as StateFilterInputConversion<Input3>>::Remainder: StateFilterInputCombination<F3::ValidOutput>,

    <<<<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined as StateFilterInputConversion<Input3>>::Remainder as StateFilterInputCombination<F3::ValidOutput>>::Combined: StateFilterInputConversion<Input4>,

    <<<<<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined as StateFilterInputConversion<Input3>>::Remainder as StateFilterInputCombination<F3::ValidOutput>>::Combined as StateFilterInputConversion<Input4>>::Remainder: StateFilterInputCombination<F4::ValidOutput>,

    <<<<<<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined as StateFilterInputConversion<Input3>>::Remainder as StateFilterInputCombination<F3::ValidOutput>>::Combined as StateFilterInputConversion<Input4>>::Remainder as StateFilterInputCombination<F4::ValidOutput>>::Combined: StateFilterInputConversion<Input5>,

    <<<<<<<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined as StateFilterInputConversion<Input3>>::Remainder as StateFilterInputCombination<F3::ValidOutput>>::Combined as StateFilterInputConversion<Input4>>::Remainder as StateFilterInputCombination<F4::ValidOutput>>::Combined as StateFilterInputConversion<Input5>>::Remainder: StateFilterInputCombination<F5::ValidOutput>,

    <<<<<<<<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined as StateFilterInputConversion<Input3>>::Remainder as StateFilterInputCombination<F3::ValidOutput>>::Combined as StateFilterInputConversion<Input4>>::Remainder as StateFilterInputCombination<F4::ValidOutput>>::Combined as StateFilterInputConversion<Input5>>::Remainder as StateFilterInputCombination<F5::ValidOutput>>::Combined: StateFilterInputConversion<Input6>,

    <<<<<<<<<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined as StateFilterInputConversion<Input3>>::Remainder as StateFilterInputCombination<F3::ValidOutput>>::Combined as StateFilterInputConversion<Input4>>::Remainder as StateFilterInputCombination<F4::ValidOutput>>::Combined as StateFilterInputConversion<Input5>>::Remainder as StateFilterInputCombination<F5::ValidOutput>>::Combined as StateFilterInputConversion<Input6>>::Remainder: StateFilterInputCombination<F6::ValidOutput>,
{
    type ValidOutput =
    <<<<<<<<<<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined as StateFilterInputConversion<Input3>>::Remainder as StateFilterInputCombination<F3::ValidOutput>>::Combined as StateFilterInputConversion<Input4>>::Remainder as StateFilterInputCombination<F4::ValidOutput>>::Combined as StateFilterInputConversion<Input5>>::Remainder as StateFilterInputCombination<F5::ValidOutput>>::Combined as StateFilterInputConversion<Input6>>::Remainder as StateFilterInputCombination<F6::ValidOutput>>::Combined
    ;
    type Error = StateFilterSevenChainError<F0::Error, F1::Error, F2::Error, F3::Error, F4::Error, F5::Error, F6::Error>;
    fn filter(state: &State, value: InitialInput) -> Result<Self::ValidOutput, Self::Error> {
        let (input, remainder) = value.split_take();
        F0::filter(state, input)
            .map(|v| remainder.combine(v))
            .map_err(|e| StateFilterSevenChainError::Filter0(e))
            .and_then(|v| {
                let (input, remainder) = v.split_take();
                F1::filter(state, input).map(|v| remainder.combine(v))
            .map_err(|e| StateFilterSevenChainError::Filter1(e))
            })
            .and_then(|v| {
                let (input, remainder) = v.split_take();
                F2::filter(state, input).map(|v| remainder.combine(v))
            .map_err(|e| StateFilterSevenChainError::Filter2(e))
            })
            .and_then(|v| {
                let (input, remainder) = v.split_take();
                F3::filter(state, input).map(|v| remainder.combine(v))
            .map_err(|e| StateFilterSevenChainError::Filter3(e))
            })
            .and_then(|v| {
                let (input, remainder) = v.split_take();
                F4::filter(state, input).map(|v| remainder.combine(v))
            .map_err(|e| StateFilterSevenChainError::Filter4(e))
            })
            .and_then(|v| {
                let (input, remainder) = v.split_take();
                F5::filter(state, input).map(|v| remainder.combine(v))
            .map_err(|e| StateFilterSevenChainError::Filter5(e))
            })
            .and_then(|v| {
                let (input, remainder) = v.split_take();
                F6::filter(state, input).map(|v| remainder.combine(v))
            .map_err(|e| StateFilterSevenChainError::Filter6(e))
            })
    }
}
#[derive(thiserror::Error, Debug)]
pub enum StateFilterSevenChainError<E0: std::error::Error, E1: std::error::Error, E2: std::error::Error, E3: std::error::Error, E4: std::error::Error, E5: std::error::Error, E6: std::error::Error> {
    #[error(transparent)]
    Filter0(E0),
    #[error(transparent)]
    Filter1(E1),
    #[error(transparent)]
    Filter2(E2),
    #[error(transparent)]
    Filter3(E3),
    #[error(transparent)]
    Filter4(E4),
    #[error(transparent)]
    Filter5(E5),
    #[error(transparent)]
    Filter6(E6),
}
impl<
    State,
    InitialInput,
    Input0,
    Input1,
    Input2,
    Input3,
    Input4,
    Input5,
    Input6,
    Input7,
    F0: StateFilter<State, Input0>,
    F1: StateFilter<State, Input1>,
    F2: StateFilter<State, Input2>,
    F3: StateFilter<State, Input3>,
    F4: StateFilter<State, Input4>,
    F5: StateFilter<State, Input5>,
    F6: StateFilter<State, Input6>,
    F7: StateFilter<State, Input7>,
> StateFilter<State, InitialInput>
    for (
        Condition<Input0, F0>,
        Condition<Input1, F1>,
        Condition<Input2, F2>,
        Condition<Input3, F3>,
        Condition<Input4, F4>,
        Condition<Input5, F5>,
        Condition<Input6, F6>,
        Condition<Input7, F7>,
    )
where
    InitialInput: StateFilterInputConversion<Input0>,
    <InitialInput as StateFilterInputConversion<Input0>>::Remainder:
        StateFilterInputCombination<F0::ValidOutput>,
    <<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined: StateFilterInputConversion<Input1>,
    <<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder:
        StateFilterInputCombination<F1::ValidOutput>,
    <<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined: StateFilterInputConversion<Input2>,

    <<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder:
        StateFilterInputCombination<F2::ValidOutput>,
        
        
    <<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined: StateFilterInputConversion<Input3>,

    <<<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined as StateFilterInputConversion<Input3>>::Remainder: StateFilterInputCombination<F3::ValidOutput>,

    <<<<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined as StateFilterInputConversion<Input3>>::Remainder as StateFilterInputCombination<F3::ValidOutput>>::Combined: StateFilterInputConversion<Input4>,

    <<<<<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined as StateFilterInputConversion<Input3>>::Remainder as StateFilterInputCombination<F3::ValidOutput>>::Combined as StateFilterInputConversion<Input4>>::Remainder: StateFilterInputCombination<F4::ValidOutput>,

    <<<<<<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined as StateFilterInputConversion<Input3>>::Remainder as StateFilterInputCombination<F3::ValidOutput>>::Combined as StateFilterInputConversion<Input4>>::Remainder as StateFilterInputCombination<F4::ValidOutput>>::Combined: StateFilterInputConversion<Input5>,

    <<<<<<<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined as StateFilterInputConversion<Input3>>::Remainder as StateFilterInputCombination<F3::ValidOutput>>::Combined as StateFilterInputConversion<Input4>>::Remainder as StateFilterInputCombination<F4::ValidOutput>>::Combined as StateFilterInputConversion<Input5>>::Remainder: StateFilterInputCombination<F5::ValidOutput>,

    <<<<<<<<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined as StateFilterInputConversion<Input3>>::Remainder as StateFilterInputCombination<F3::ValidOutput>>::Combined as StateFilterInputConversion<Input4>>::Remainder as StateFilterInputCombination<F4::ValidOutput>>::Combined as StateFilterInputConversion<Input5>>::Remainder as StateFilterInputCombination<F5::ValidOutput>>::Combined: StateFilterInputConversion<Input6>,

    <<<<<<<<<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined as StateFilterInputConversion<Input3>>::Remainder as StateFilterInputCombination<F3::ValidOutput>>::Combined as StateFilterInputConversion<Input4>>::Remainder as StateFilterInputCombination<F4::ValidOutput>>::Combined as StateFilterInputConversion<Input5>>::Remainder as StateFilterInputCombination<F5::ValidOutput>>::Combined as StateFilterInputConversion<Input6>>::Remainder: StateFilterInputCombination<F6::ValidOutput>,

    <<<<<<<<<<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined as StateFilterInputConversion<Input3>>::Remainder as StateFilterInputCombination<F3::ValidOutput>>::Combined as StateFilterInputConversion<Input4>>::Remainder as StateFilterInputCombination<F4::ValidOutput>>::Combined as StateFilterInputConversion<Input5>>::Remainder as StateFilterInputCombination<F5::ValidOutput>>::Combined as StateFilterInputConversion<Input6>>::Remainder as StateFilterInputCombination<F6::ValidOutput>>::Combined: StateFilterInputConversion<Input7>,

    <<<<<<<<<<<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined as StateFilterInputConversion<Input3>>::Remainder as StateFilterInputCombination<F3::ValidOutput>>::Combined as StateFilterInputConversion<Input4>>::Remainder as StateFilterInputCombination<F4::ValidOutput>>::Combined as StateFilterInputConversion<Input5>>::Remainder as StateFilterInputCombination<F5::ValidOutput>>::Combined as StateFilterInputConversion<Input6>>::Remainder as StateFilterInputCombination<F6::ValidOutput>>::Combined as StateFilterInputConversion<Input7>>::Remainder: StateFilterInputCombination<F7::ValidOutput>,
{
    type ValidOutput=
    <<<<<<<<<<<<<<<<InitialInput as StateFilterInputConversion<Input0>>::Remainder as StateFilterInputCombination<
        F0::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input1>>::Remainder as StateFilterInputCombination<
        F1::ValidOutput,
    >>::Combined as StateFilterInputConversion<Input2>>::Remainder as
        StateFilterInputCombination<F2::ValidOutput>>::Combined as StateFilterInputConversion<Input3>>::Remainder as StateFilterInputCombination<F3::ValidOutput>>::Combined as StateFilterInputConversion<Input4>>::Remainder as StateFilterInputCombination<F4::ValidOutput>>::Combined as StateFilterInputConversion<Input5>>::Remainder as StateFilterInputCombination<F5::ValidOutput>>::Combined as StateFilterInputConversion<Input6>>::Remainder as StateFilterInputCombination<F6::ValidOutput>>::Combined as StateFilterInputConversion<Input7>>::Remainder as StateFilterInputCombination<F7::ValidOutput>>::Combined
        ;
    type Error = StateFilterEightChainError<F0::Error, F1::Error, F2::Error, F3::Error, F4::Error, F5::Error, F6::Error, F7::Error>;
    fn filter(state: &State, value: InitialInput) -> Result<Self::ValidOutput, Self::Error> {
        let (input, remainder) = value.split_take();
        F0::filter(state, input)
            .map(|v| remainder.combine(v))
            .map_err(|e| StateFilterEightChainError::Filter0(e))
            .and_then(|v| {
                let (input, remainder) = v.split_take();
                F1::filter(state, input).map(|v| remainder.combine(v))
            .map_err(|e| StateFilterEightChainError::Filter1(e))
            })
            .and_then(|v| {
                let (input, remainder) = v.split_take();
                F2::filter(state, input).map(|v| remainder.combine(v))
            .map_err(|e| StateFilterEightChainError::Filter2(e))
            })
            .and_then(|v| {
                let (input, remainder) = v.split_take();
                F3::filter(state, input).map(|v| remainder.combine(v))
            .map_err(|e| StateFilterEightChainError::Filter3(e))
            })
            .and_then(|v| {
                let (input, remainder) = v.split_take();
                F4::filter(state, input).map(|v| remainder.combine(v))
            .map_err(|e| StateFilterEightChainError::Filter4(e))
            })
            .and_then(|v| {
                let (input, remainder) = v.split_take();
                F5::filter(state, input).map(|v| remainder.combine(v))
            .map_err(|e| StateFilterEightChainError::Filter5(e))
            })
            .and_then(|v| {
                let (input, remainder) = v.split_take();
                F6::filter(state, input).map(|v| remainder.combine(v))
            .map_err(|e| StateFilterEightChainError::Filter6(e))
            })
            .and_then(|v| {
                let (input, remainder) = v.split_take();
                F7::filter(state, input).map(|v| remainder.combine(v))
            .map_err(|e| StateFilterEightChainError::Filter7(e))
            })
    }
}
#[derive(thiserror::Error, Debug)]
pub enum StateFilterEightChainError<E0: std::error::Error, E1: std::error::Error, E2: std::error::Error, E3: std::error::Error, E4: std::error::Error, E5: std::error::Error, E6: std::error::Error, E7: std::error::Error> {
    #[error(transparent)]
    Filter0(E0),
    #[error(transparent)]
    Filter1(E1),
    #[error(transparent)]
    Filter2(E2),
    #[error(transparent)]
    Filter3(E3),
    #[error(transparent)]
    Filter4(E4),
    #[error(transparent)]
    Filter5(E5),
    #[error(transparent)]
    Filter6(E6),
    #[error(transparent)]
    Filter7(E7),
}

pub trait StateFilterInputConversion<T> {
    type Remainder;
    fn split_take(self) -> (T, Self::Remainder);
}

pub trait StateFilterInputCombination<T> {
    type Combined;
    fn combine(self, value: T) -> Self::Combined;
}

impl<T> StateFilterInputConversion<T> for T {
    type Remainder = ();
    fn split_take(self) -> (T, Self::Remainder) {
        (self, ())
    }
}

impl<T> StateFilterInputCombination<T> for () {
    type Combined = T;
    fn combine(self, value: T) -> Self::Combined {
        value
    }
}
