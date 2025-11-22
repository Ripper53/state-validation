//! ## State Validation
//! `state-validation` lets you validate an input for a given state. Then, run an action using the validated output.
//!
//! Ex. You want to remove an admin from `UserStorage`, given a `UserID`, you want to retrieve the `User` who maps onto the `UserID` and validate they are an existing user whose privilege level is admin.
//! The state is `UserStorage`, the input is a `UserID`, and the valid output is an `AdminUser`.
//!
//! Here is our input `UserID`:
//! ```
//! # use std::collections::{HashSet, HashMap};
//! # use state_validation::{Validator, ValidAction, StateFilter, Condition};
//! #[derive(Hash, PartialEq, Eq, Clone, Copy)]
//! struct UserID(usize);
//! ```
//! Our `User` which holds its `UserID` and `username`:
//! ```
//! # use std::collections::{HashSet, HashMap};
//! # use state_validation::{Validator, ValidAction, StateFilter, Condition};
//! # #[derive(Hash, PartialEq, Eq, Clone, Copy)]
//! # struct UserID(usize);
//! #[derive(Clone)]
//! struct User {
//!     id: UserID,
//!     username: String,
//! }
//! ```
//! Our state will be `UserStorage`:
//! ```
//! # use std::collections::{HashSet, HashMap};
//! # use state_validation::{Validator, ValidAction, StateFilter, Condition};
//! # #[derive(Hash, PartialEq, Eq, Clone, Copy)]
//! # struct UserID(usize);
//! # #[derive(Clone)]
//! # struct User {
//! #     id: UserID,
//! #     username: String,
//! # }
//! #[derive(Default)]
//! struct UserStorage {
//!     maps: HashMap<UserID, User>,
//! }
//! ```
//! We will create a newtype `AdminUser`, which only users with admin privilege will be wrapped in.
//! This will also be the output of our filter which checks if a user is admin:
//! ```
//! # use std::collections::{HashSet, HashMap};
//! # use state_validation::{Validator, ValidAction, StateFilter, Condition};
//! # #[derive(Hash, PartialEq, Eq, Clone, Copy)]
//! # struct UserID(usize);
//! # #[derive(Clone)]
//! # struct User {
//! #     id: UserID,
//! #     username: String,
//! # }
//! # #[derive(Default)]
//! # struct UserStorage {
//! #     maps: HashMap<UserID, User>,
//! # }
//! struct AdminUser(User);
//! ```
//! Our first filter will check if a `User` exists given a `UserID`:
//! ```
//! # use std::collections::{HashSet, HashMap};
//! # use state_validation::{Validator, ValidAction, StateFilter, Condition};
//! # #[derive(Hash, PartialEq, Eq, Clone, Copy)]
//! # struct UserID(usize);
//! # #[derive(Clone)]
//! # struct User {
//! #     id: UserID,
//! #     username: String,
//! # }
//! # #[derive(Default)]
//! # struct UserStorage {
//! #     maps: HashMap<UserID, User>,
//! # }
//! # struct AdminUser(User);
//! struct UserExists;
//! # #[derive(Debug)]
//! # struct UserDoesNotExistError;
//! # impl std::error::Error for UserDoesNotExistError {}
//! # impl std::fmt::Display for UserDoesNotExistError {
//! #    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//! #        write!(f, "user does not exist")
//! #     }
//! # }
//! impl StateFilter<UserStorage, UserID> for UserExists {
//!     type ValidOutput = User;
//!     type Error = UserDoesNotExistError;
//!     fn filter(state: &UserStorage, user_id: UserID) -> Result<Self::ValidOutput, Self::Error> {
//!         if let Some(user) = state.maps.get(&user_id) {
//!             Ok(user.clone())
//!         } else {
//!             Err(UserDoesNotExistError)
//!         }
//!     }
//! }
//! ```
//! Our second filter will check if a `User` is admin:
//! ```
//! # use std::collections::{HashSet, HashMap};
//! # use state_validation::{Validator, ValidAction, StateFilter, Condition};
//! # #[derive(Hash, PartialEq, Eq, Clone, Copy)]
//! # struct UserID(usize);
//! # #[derive(Clone)]
//! # struct User {
//! #     id: UserID,
//! #     username: String,
//! # }
//! # #[derive(Default)]
//! # struct UserStorage {
//! #     maps: HashMap<UserID, User>,
//! # }
//! # struct AdminUser(User);
//! #
//! struct UserIsAdmin;
//! # #[derive(Debug)]
//! # struct UserIsNotAdminError;
//! # impl std::error::Error for UserIsNotAdminError {}
//! # impl std::fmt::Display for UserIsNotAdminError {
//! #    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//! #        write!(f, "user is not an admin")
//! #     }
//! # }
//! impl<State> StateFilter<State, User> for UserIsAdmin {
//!     type ValidOutput = AdminUser;
//!     type Error = UserIsNotAdminError;
//!     fn filter(state: &State, user: User) -> Result<Self::ValidOutput, Self::Error> {
//!         if user.username == "ADMIN" {
//!             Ok(AdminUser(user))
//!         } else {
//!             Err(UserIsNotAdminError)
//!         }
//!     }
//! }
//! ```
//! Note: in the above code, we don't care about the `state` so it is a generic.
//!
//! Now, we can finally implement an action that removes the admin from user storage:
//! ```
//! # use std::collections::{HashSet, HashMap};
//! # use state_validation::{Validator, ValidAction, StateFilter, Condition};
//! # #[derive(Hash, PartialEq, Eq, Clone, Copy)]
//! # struct UserID(usize);
//! # #[derive(Clone)]
//! # struct User {
//! #     id: UserID,
//! #     username: String,
//! # }
//! # #[derive(Default)]
//! # struct UserStorage {
//! #     maps: HashMap<UserID, User>,
//! # }
//! # struct AdminUser(User);
//! # struct UserExists;
//! # #[derive(Debug)]
//! # struct UserDoesNotExistError;
//! # impl std::error::Error for UserDoesNotExistError {}
//! # impl std::fmt::Display for UserDoesNotExistError {
//! #    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//! #        write!(f, "user does not exist")
//! #     }
//! # }
//! # impl StateFilter<UserStorage, UserID> for UserExists {
//! #     type ValidOutput = User;
//! #     type Error = UserDoesNotExistError;
//! #     fn filter(state: &UserStorage, user_id: UserID) -> Result<Self::ValidOutput, Self::Error> {
//! #         if let Some(user) = state.maps.get(&user_id) {
//! #             Ok(user.clone())
//! #         } else {
//! #             Err(UserDoesNotExistError)
//! #         }
//! #     }
//! # }
//! # struct UserIsAdmin;
//! # #[derive(Debug)]
//! # struct UserIsNotAdminError;
//! # impl std::error::Error for UserIsNotAdminError {}
//! # impl std::fmt::Display for UserIsNotAdminError {
//! #    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//! #        write!(f, "user is not an admin")
//! #     }
//! # }
//! # impl<State> StateFilter<State, User> for UserIsAdmin {
//! #     type ValidOutput = AdminUser;
//! #     type Error = UserIsNotAdminError;
//! #     fn filter(state: &State, user: User) -> Result<Self::ValidOutput, Self::Error> {
//! #         if user.username == "ADMIN" {
//! #             Ok(AdminUser(user))
//! #         } else {
//! #             Err(UserIsNotAdminError)
//! #         }
//! #     }
//! # }
//! #
//! struct RemoveAdmin;
//! impl ValidAction<UserStorage, UserID> for RemoveAdmin {
//!     // To chain filters, use `Condition`.
//!     type Filter = (
//!         //       <Input,  Filter>
//!         Condition<UserID, UserExists>,
//!         // Previous filter outputs a `User`,
//!         // so next filter can take `User` as input.
//!         Condition<User, UserIsAdmin>,
//!     );
//!     type Output = UserStorage;
//!     fn with_valid_input(
//!         self,
//!         mut state: UserStorage,
//!         // The final output from the filters is an `AdminUser`.
//!         admin_user: <Self::Filter as StateFilter<UserStorage, UserID>>::ValidOutput,
//!     ) -> Self::Output {
//!         let _ = state.maps.remove(&admin_user.0.id).unwrap();
//!         state
//!     }
//! }
//! ```
//! Now, let's put it all together. We create the state `UserStorage`,
//! and then use [`Validator::try_new`] to run our filters. An error is returned if any of the filters fail,
//! otherwise we get a validator that we can run an action on:
//! ```
//! # use std::collections::{HashSet, HashMap};
//! # use state_validation::{Validator, ValidAction, StateFilter, Condition};
//! # #[derive(Hash, PartialEq, Eq, Clone, Copy)]
//! # struct UserID(usize);
//! # #[derive(Clone)]
//! # struct User {
//! #     id: UserID,
//! #     username: String,
//! # }
//! # #[derive(Default)]
//! # struct UserStorage {
//! #     maps: HashMap<UserID, User>,
//! # }
//! # struct AdminUser(User);
//! # struct UserExists;
//! # #[derive(Debug)]
//! # struct UserDoesNotExistError;
//! # impl std::error::Error for UserDoesNotExistError {}
//! # impl std::fmt::Display for UserDoesNotExistError {
//! #    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//! #        write!(f, "user does not exist")
//! #     }
//! # }
//! # impl StateFilter<UserStorage, UserID> for UserExists {
//! #     type ValidOutput = User;
//! #     type Error = UserDoesNotExistError;
//! #     fn filter(state: &UserStorage, user_id: UserID) -> Result<Self::ValidOutput, Self::Error> {
//! #         if let Some(user) = state.maps.get(&user_id) {
//! #             Ok(user.clone())
//! #         } else {
//! #             Err(UserDoesNotExistError)
//! #         }
//! #     }
//! # }
//! # struct UserIsAdmin;
//! # #[derive(Debug)]
//! # struct UserIsNotAdminError;
//! # impl std::error::Error for UserIsNotAdminError {}
//! # impl std::fmt::Display for UserIsNotAdminError {
//! #    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//! #        write!(f, "user is not an admin")
//! #     }
//! # }
//! # impl<State> StateFilter<State, User> for UserIsAdmin {
//! #     type ValidOutput = AdminUser;
//! #     type Error = UserIsNotAdminError;
//! #     fn filter(state: &State, user: User) -> Result<Self::ValidOutput, Self::Error> {
//! #         if user.username == "ADMIN" {
//! #             Ok(AdminUser(user))
//! #         } else {
//! #             Err(UserIsNotAdminError)
//! #         }
//! #     }
//! # }
//! #
//! # struct RemoveAdmin;
//! # impl ValidAction<UserStorage, UserID> for RemoveAdmin {
//! #     type Filter = (
//! #         Condition<UserID, UserExists>,
//! #         Condition<User, UserIsAdmin>,
//! #     );
//! #     type Output = UserStorage;
//! #     fn with_valid_input(
//! #         self,
//! #         mut state: UserStorage,
//! #         admin_user: <Self::Filter as StateFilter<UserStorage, UserID>>::ValidOutput,
//! #     ) -> Self::Output {
//! #         let _ = state.maps.remove(&admin_user.0.id).unwrap();
//! #         state
//! #     }
//! # }
//! #
//! // State setup
//! let mut user_storage = UserStorage::default();
//! user_storage.maps.insert(UserID(0), User {
//!     id: UserID(0),
//!     username: "ADMIN".to_string(),
//! });
//!
//! // Create validator which will validate the input.
//! // No error is returned if validation succeeds.
//! let validator = Validator::try_new(user_storage, UserID(0)).expect("admin user did not exist");
//!
//! // Execute an action which requires the state and input above.
//! let user_storage = validator.execute(RemoveAdmin);
//!
//! assert!(user_storage.maps.is_empty());
//! ```
//! ---
//! Another example using `UserExists` filter, and `UserIsAdmin` filter in the body of the [`ValidAction`]:
//! ```
//! # use std::collections::{HashSet, HashMap};
//! # use state_validation::{Validator, ValidAction, StateFilter, Condition};
//! # #[derive(Hash, PartialEq, Eq, Clone, Copy)]
//! # struct UserID(usize);
//! #
//! # #[derive(Default)]
//! # struct UserStorage {
//! #     maps: HashMap<UserID, User>,
//! # }
//! # #[derive(Clone)]
//! # struct User {
//! #     id: UserID,
//! #     username: String,
//! # }
//! #
//! # struct AdminUser(User);
//! #
//! # struct UserExists;
//! # #[derive(Debug)]
//! # struct UserDoesNotExistError;
//! # impl std::error::Error for UserDoesNotExistError {}
//! # impl std::fmt::Display for UserDoesNotExistError {
//! #    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//! #        write!(f, "user does not exist")
//! #     }
//! # }
//! # impl StateFilter<UserStorage, UserID> for UserExists {
//! #     type ValidOutput = User;
//! #     type Error = UserDoesNotExistError;
//! #     fn filter(state: &UserStorage, user_id: UserID) -> Result<Self::ValidOutput, Self::Error> {
//! #         if let Some(user) = state.maps.get(&user_id) {
//! #             Ok(user.clone())
//! #         } else {
//! #             Err(UserDoesNotExistError)
//! #         }
//! #     }
//! # }
//! #
//! # struct UserIsAdmin;
//! # #[derive(Debug)]
//! # struct UserIsNotAdminError;
//! # impl std::error::Error for UserIsNotAdminError {}
//! # impl std::fmt::Display for UserIsNotAdminError {
//! #    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//! #        write!(f, "user is not an admin")
//! #     }
//! # }
//! # impl StateFilter<UserStorage, User> for UserIsAdmin {
//! #     type ValidOutput = AdminUser;
//! #     type Error = UserIsNotAdminError;
//! #     fn filter(state: &UserStorage, user: User) -> Result<Self::ValidOutput, Self::Error> {
//! #         if user.username == "ADMIN" {
//! #             Ok(AdminUser(user))
//! #         } else {
//! #             Err(UserIsNotAdminError)
//! #         }
//! #     }
//! # }
//! #
//! enum Privilege {
//!     None,
//!     Admin,
//! }
//!
//! struct UpdateUserPrivilege(Privilege);
//! impl ValidAction<UserStorage, UserID> for UpdateUserPrivilege {
//!     type Filter = UserExists;
//!     type Output = UserStorage;
//!     fn with_valid_input(
//!         self,
//!         mut state: UserStorage,
//!         mut user: <Self::Filter as StateFilter<UserStorage, UserID>>::ValidOutput,
//!     ) -> Self::Output {
//!         match self.0 {
//!             Privilege::None => {
//!                 if let Ok(AdminUser(mut user)) = UserIsAdmin::filter(&state, user) {
//!                     user.username = "NOT_ADMIN".to_string();
//!                     let _ = state.maps.insert(user.id, user);
//!                 }
//!             }
//!             Privilege::Admin => {
//!                 user.username = "ADMIN".to_string();
//!                 let _ = state.maps.insert(user.id, user);
//!             }
//!         }
//!         state
//!     }
//! }
//!
//! # let mut user_storage = UserStorage::default();
//! # user_storage.maps.insert(UserID(0), User {
//! #     id: UserID(0),
//! #     username: "ADMIN".to_string(),
//! # });
//! # assert_eq!(user_storage.maps.len(), 1);
//! # let user = user_storage.maps.get(&UserID(0));
//! # assert!(user.is_some());
//! # assert_eq!(user.unwrap().username, "ADMIN");
//!
//! // This `Validator` has its generic `Filter` parameter implicitly changed
//! // based on what action we call. On a type level, this is a different type
//! // than the validator in the previous example even though we write the same code
//! // due to its `Filter` generic parameter being different.
//! let validator = Validator::try_new(user_storage, UserID(0)).expect("user did not exist");
//!
//! let user_storage = validator.execute(UpdateUserPrivilege(Privilege::None));
//!
//! # assert_eq!(user_storage.maps.len(), 1);
//! let user = user_storage.maps.get(&UserID(0));
//! # assert!(user.is_some());
//! assert_eq!(user.unwrap().username, "NOT_ADMIN");
//! ```
//!
//! ## [`StateFilterInputConversion`] & [`StateFilterInputCombination`]
//! Automatic implementations of [`StateFilterInputConversion`] and [`StateFilterInputCombination`]
//! are generated with the [`StateFilterConversion`] derive macro.
//! Example:
//! ```
//! # use state_validation::StateFilterConversion;
//! # struct UserID(usize);
//! # struct User;
//! #[derive(StateFilterConversion)]
//! struct UserWithUserName {
//!     #[conversion(User)]
//!     user_id: UserID,
//!     username: String,
//! }
//! ```
//! Now, `UserWithUsername` can be broken down into `User`, `UserID`, and `String`.
//! Take advantage of the newtype pattern to breakdown the input further.
//! For example, instead of having username as a `String`, use:
//! ```
//! struct Username(String);
//! ```
//! This way, the compiler can differentiate between a `String` and a `Username`.
//!
//!
//! The [`StateFilterInputConversion`] and [`StateFilterInputCombination`] traits work together
//! to allow splitting the input down into its parts and then back together.
//!
//! The usefulness of this is, if a [`StateFilter`] only requires a part of the input,
//! [`StateFilterInputConversion`] can split it down to just that part, and leave the rest in [`StateFilterInputConversion::Remainder`]
//! which will be combined with the output of the filter. And, each consecutive filter can split
//! whatever input they desire and combine their output with the remainder they did not touch.
//!
//! Here is an example with manual implementations: Assume we wanted to change the username of a user,
//! if its `UserID` and current username matched that of the input.
//! First, let's create its input:
//! ```
//! # struct UserID(usize);
//! struct UserWithUsername {
//!     user_id: UserID,
//!     username: String,
//! }
//! ```
//!
//! We want to check if a user with `UserID` exists, and their current username is `username`, and then update their username.
//! We already have a filter that will check if a user exists. It is called: `UserExists`.
//! But, `UserExists` does not take `UserWithUsername` as an input. It only takes `UserID` as input.
//! `UserWithUsername` does contain a `UserID`. So, we should be able to retrieve the `UserID`,
//! pass it into `UserExists`, then reconstruct the output of `UserExists` with the leftover `username`.
//! The first part of solving this issue is input conversion into `UserID`, so we implement [`StateFilterInputConversion`]:
//! ```
//! # use state_validation::StateFilterInputConversion;
//! # struct UserID(usize);
//! # struct UserWithUsername {
//! #     user_id: UserID,
//! #     username: String,
//! # }
//! struct UsernameForUserID(String);
//! impl StateFilterInputConversion<UserID> for UserWithUsername {
//!     type Remainder = UsernameForUserID;
//!     fn split_take(self) -> (UserID, Self::Remainder) {
//!         (self.user_id, UsernameForUserID(self.username))
//!     }
//! }
//! ```
//! Notice in the above code, within `split_take`, the first element of the tuple is the input our `UserExists` filter cares about.
//! However, for the `Remainder`, we have a newtype which stores the leftover to combine later with the output of `UserExists`.
//! ```
//! # use state_validation::StateFilterInputCombination;
//! # struct User;
//! # struct UserID(usize);
//! # struct UsernameForUserID(String);
//! struct UsernameForUser {
//!     user: User,
//!     username: String,
//! }
//! impl StateFilterInputCombination<User> for UsernameForUserID {
//!     type Combined = UsernameForUser;
//!     fn combine(self, user: User) -> Self::Combined {
//!         UsernameForUser {
//!             user,
//!             username: self.0,
//!         }
//!     }
//! }
//! ```
//! Now, the combination of `UserExists`'s output (which is a `User`) and the leftover `username`,
//! results in a new struct called `UsernameForUser`.
//!
//! Now, we need a new filter that checks if the username of the `User` is equal to `username`.
//! ```
//! # use state_validation::StateFilter;
//! # struct User {
//! #     username: String,
//! # }
//! # struct UsernameForUser {
//! #     user: User,
//! #     username: String,
//! # }
//! struct UsernameEquals;
//! # #[derive(Debug)]
//! # struct UsernameIsNotEqual;
//! # impl std::error::Error for UsernameIsNotEqual {}
//! # impl std::fmt::Display for UsernameIsNotEqual {
//! #    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//! #        write!(f, "username is not equal")
//! #     }
//! # }
//! impl<State> StateFilter<State, UsernameForUser> for UsernameEquals {
//!     type ValidOutput = User;
//!     type Error = UsernameIsNotEqual;
//!     fn filter(state: &State, value: UsernameForUser) -> Result<Self::ValidOutput, Self::Error> {
//!         if value.user.username == value.username {
//!             Ok(value.user)
//!         } else {
//!             Err(UsernameIsNotEqual)
//!         }
//!     }
//! }
//! ```
//!
//! Finally, we can run our action to update the user's username:
//! ```
//! # use std::collections::HashMap;
//! # use state_validation::{ValidAction, Condition, StateFilter, StateFilterInputConversion, StateFilterInputCombination};
//! # #[derive(Hash, PartialEq, Eq, Clone, Copy)]
//! # struct UserID(usize);
//! # #[derive(Clone)]
//! # struct User {
//! #     id: UserID,
//! #     username: String,
//! # }
//! # #[derive(Default)]
//! # struct UserStorage {
//! #     maps: HashMap<UserID, User>,
//! # }
//! # struct UserExists;
//! # #[derive(Debug)]
//! # struct UserDoesNotExistError;
//! # impl std::error::Error for UserDoesNotExistError {}
//! # impl std::fmt::Display for UserDoesNotExistError {
//! #    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//! #        write!(f, "user does not exist")
//! #     }
//! # }
//! # impl StateFilter<UserStorage, UserID> for UserExists {
//! #     type ValidOutput = User;
//! #     type Error = UserDoesNotExistError;
//! #     fn filter(state: &UserStorage, user_id: UserID) -> Result<Self::ValidOutput, Self::Error> {
//! #         if let Some(user) = state.maps.get(&user_id) {
//! #             Ok(user.clone())
//! #         } else {
//! #             Err(UserDoesNotExistError)
//! #         }
//! #     }
//! # }
//! # struct UserWithUsername {
//! #     user_id: UserID,
//! #     username: String,
//! # }
//! # struct UsernameForUserID(String);
//! # impl StateFilterInputConversion<UserID> for UserWithUsername {
//! #     type Remainder = UsernameForUserID;
//! #     fn split_take(self) -> (UserID, Self::Remainder) {
//! #         (self.user_id, UsernameForUserID(self.username))
//! #     }
//! # }
//! # struct UsernameForUser {
//! #     user: User,
//! #     username: String,
//! # }
//! # impl StateFilterInputCombination<User> for UsernameForUserID {
//! #     type Combined = UsernameForUser;
//! #     fn combine(self, user: User) -> Self::Combined {
//! #         UsernameForUser {
//! #             user,
//! #             username: self.0,
//! #         }
//! #     }
//! # }
//! # struct UsernameEquals;
//! # #[derive(Debug)]
//! # struct UsernameIsNotEqual;
//! # impl std::error::Error for UsernameIsNotEqual {}
//! # impl std::fmt::Display for UsernameIsNotEqual {
//! #    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//! #        write!(f, "username is not equal")
//! #     }
//! # }
//! # impl<State> StateFilter<State, UsernameForUser> for UsernameEquals {
//! #     type ValidOutput = User;
//! #     type Error = UsernameIsNotEqual;
//! #     fn filter(state: &State, value: UsernameForUser) -> Result<Self::ValidOutput, Self::Error> {
//! #         if value.user.username == value.username {
//! #             Ok(value.user)
//! #         } else {
//! #             Err(UsernameIsNotEqual)
//! #         }
//! #     }
//! # }
//! struct UpdateUsername {
//!     new_username: String,
//! }
//! impl ValidAction<UserStorage, UserWithUsername> for UpdateUsername {
//!     type Filter = (
//!         Condition<UserID, UserExists>,
//!         Condition<UsernameForUser, UsernameEquals>,
//!     );
//!     type Output = UserStorage;
//!     fn with_valid_input(
//!         self,
//!         mut state: UserStorage,
//!         mut user: <Self::Filter as StateFilter<UserStorage, UserWithUsername>>::ValidOutput,
//!     ) -> Self::Output {
//!         user.username = self.new_username;
//!         let _ = state.maps.insert(user.id, user);
//!         state
//!     }
//! }
//! ```
//!
//! ## Soundness Rules
//! [`Validator::try_new`] takes ownership of the `state` to disallow consecutive
//! [`Validator::execute`] calls because an action is assumed to mutate the `state`.
//! Since an action is assumed to mutate the `state`, any validators using the same `state`
//! cannot be created.
//!
//! It is up to you to make sure the filters properly validate what they promise.
//!
//! ## Limitations
//! Currently, the amount of filters that can be chained is eight.
//! The reason for this is because of variadics not being supported as of Rust 2024.
//! Having no more than eight implementations is arbitrary because having more than eight filters is unlikely.
//! There is no reason not to implement more in the future, if more than eight filters are required.

mod action;
mod condition;
#[cfg(feature = "input_collector")]
mod input_collector;
mod state_filter;
pub use action::*;
pub use condition::*;
#[cfg(feature = "input_collector")]
pub use input_collector::*;
pub use state_filter::*;
#[cfg(feature = "derive")]
pub use state_validation_derive::*;

pub struct Validator<State, Input, Filter: StateFilter<State, Input>> {
    state: State,
    value: Filter::ValidOutput,
    _p: std::marker::PhantomData<(Input, Filter)>,
}

impl<State, Input, Filter: StateFilter<State, Input>> Validator<State, Input, Filter> {
    pub fn try_new(state: State, input: Input) -> Result<Self, Filter::Error> {
        let value = Filter::filter(&state, input)?;
        Ok(Validator {
            state,
            value,
            _p: std::marker::PhantomData::default(),
        })
    }
    pub fn valid_output(&self) -> &Filter::ValidOutput {
        &self.value
    }
    pub fn execute<Action: ValidAction<State, Input, Filter = Filter>>(
        self,
        valid_action: Action,
    ) -> Action::Output {
        valid_action.with_valid_input(self.state, self.value)
    }
}
