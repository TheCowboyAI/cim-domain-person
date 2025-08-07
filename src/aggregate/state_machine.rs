//! State machine framework for aggregates

use cim_domain::{DomainError, DomainResult};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

/// Trait for state machine states
pub trait State: Clone + Debug + Eq + Hash + Send + Sync + 'static {}

/// Trait for state machine commands/triggers
pub trait Command: Clone + Debug + Send + Sync + 'static {}

/// State transition definition
pub struct Transition<S: State, C: Command> {
    pub from_state: S,
    pub to_state: S,
    pub guard: Option<Arc<dyn Fn(&S, &C) -> bool + Send + Sync>>,
    pub action: Option<Arc<dyn Fn(&S, &C) -> DomainResult<()> + Send + Sync>>,
}

/// State machine for aggregates
pub struct StateMachine<S: State, C: Command> {
    initial_state: S,
    transitions: HashMap<(S, std::any::TypeId), Vec<Transition<S, C>>>,
    state_entry_actions: HashMap<S, Box<dyn Fn(&S) -> DomainResult<()> + Send + Sync>>,
    state_exit_actions: HashMap<S, Box<dyn Fn(&S) -> DomainResult<()> + Send + Sync>>,
    _phantom: std::marker::PhantomData<C>,
}

impl<S: State + 'static, C: Command + 'static> StateMachine<S, C> {
    /// Create a new state machine builder
    pub fn builder(initial_state: S) -> StateMachineBuilder<S, C> {
        StateMachineBuilder::new(initial_state)
    }
    
    /// Validate a transition
    pub fn validate_transition(&self, current_state: &S, command: &C) -> DomainResult<S> {
        let command_type_id = std::any::TypeId::of::<C>();
        let key = (current_state.clone(), command_type_id);
        
        if let Some(transitions) = self.transitions.get(&key) {
            for transition in transitions {
                // If there's a guard, use it to determine if this transition matches
                // If no guard, the transition always matches
                let matches = if let Some(guard) = &transition.guard {
                    guard(current_state, command)
                } else {
                    true
                };
                
                if matches {
                    // Execute exit action for current state
                    if let Some(exit_action) = self.state_exit_actions.get(current_state) {
                        exit_action(current_state)?;
                    }
                    
                    // Execute transition action if present
                    if let Some(action) = &transition.action {
                        action(current_state, command)?;
                    }
                    
                    // Execute entry action for new state
                    if let Some(entry_action) = self.state_entry_actions.get(&transition.to_state) {
                        entry_action(&transition.to_state)?;
                    }
                    
                    return Ok(transition.to_state.clone());
                }
            }
        }
        
        Err(DomainError::ValidationError(
            format!("Invalid transition from {:?} with command", current_state)
        ))
    }
    
    /// Get all valid transitions from a given state
    pub fn valid_transitions(&self, state: &S) -> Vec<&S> {
        let mut to_states = Vec::new();
        
        for ((from_state, _), transitions) in &self.transitions {
            if from_state == state {
                for transition in transitions {
                    to_states.push(&transition.to_state);
                }
            }
        }
        
        to_states
    }
}

/// Builder for state machines
pub struct StateMachineBuilder<S: State, C: Command> {
    initial_state: S,
    transitions: Vec<Transition<S, C>>,
    state_entry_actions: HashMap<S, Box<dyn Fn(&S) -> DomainResult<()> + Send + Sync>>,
    state_exit_actions: HashMap<S, Box<dyn Fn(&S) -> DomainResult<()> + Send + Sync>>,
}

impl<S: State + 'static, C: Command + 'static> StateMachineBuilder<S, C> {
    pub fn new(initial_state: S) -> Self {
        Self {
            initial_state,
            transitions: Vec::new(),
            state_entry_actions: HashMap::new(),
            state_exit_actions: HashMap::new(),
        }
    }
    
    /// Add a simple transition without action
    pub fn transition(mut self, from: S, _command: C, to: S) -> Self {
        self.transitions.push(Transition {
            from_state: from,
            to_state: to,
            guard: None,
            action: None,
        });
        self
    }
    
    /// Add a transition with action
    pub fn add_transition(
        mut self,
        from: S,
        to: S,
        action: impl Fn(&S, &C) -> DomainResult<()> + Send + Sync + 'static,
    ) -> Self {
        self.transitions.push(Transition {
            from_state: from,
            to_state: to,
            guard: None,
            action: Some(Arc::new(action)),
        });
        self
    }
    
    /// Add a transition with guard
    pub fn transition_with_guard<F>(mut self, from: S, to: S, guard: F) -> Self
    where
        F: Fn(&S, &C) -> bool + Send + Sync + 'static,
    {
        self.transitions.push(Transition {
            from_state: from,
            to_state: to,
            guard: Some(Arc::new(guard)),
            action: None,
        });
        self
    }
    
    /// Add a transition with action
    pub fn transition_with_action<F>(mut self, from: S, to: S, action: F) -> Self
    where
        F: Fn(&S, &C) -> DomainResult<()> + Send + Sync + 'static,
    {
        self.transitions.push(Transition {
            from_state: from,
            to_state: to,
            guard: None,
            action: Some(Arc::new(action)),
        });
        self
    }
    
    /// Add state entry action
    pub fn on_entry<F>(mut self, state: S, action: F) -> Self
    where
        F: Fn(&S) -> DomainResult<()> + Send + Sync + 'static,
    {
        self.state_entry_actions.insert(state, Box::new(action));
        self
    }
    
    /// Add state exit action
    pub fn on_exit<F>(mut self, state: S, action: F) -> Self
    where
        F: Fn(&S) -> DomainResult<()> + Send + Sync + 'static,
    {
        self.state_exit_actions.insert(state, Box::new(action));
        self
    }
    
    /// Build the state machine
    pub fn build(self) -> StateMachine<S, C> {
        let mut transitions_map = HashMap::new();
        
        for transition in self.transitions {
            let key = (transition.from_state.clone(), std::any::TypeId::of::<C>());
            transitions_map
                .entry(key)
                .or_insert_with(Vec::new)
                .push(transition);
        }
        
        StateMachine {
            initial_state: self.initial_state,
            transitions: transitions_map,
            state_entry_actions: self.state_entry_actions,
            state_exit_actions: self.state_exit_actions,
            _phantom: std::marker::PhantomData,
        }
    }
}

/// Trait for aggregates with state machines
pub trait StateMachineAggregate {
    type State: State;
    type Command: Command;
    
    /// Get the current state
    fn current_state(&self) -> &Self::State;
    
    /// Set the current state
    fn set_state(&mut self, state: Self::State);
    
    /// Get the state machine definition
    fn state_machine() -> StateMachine<Self::State, Self::Command>;
    
    /// Handle a command using the state machine
    fn handle_command_with_state_machine(
        &mut self,
        command: Self::Command,
    ) -> DomainResult<Self::State> {
        let state_machine = Self::state_machine();
        let new_state = state_machine.validate_transition(self.current_state(), &command)?;
        self.set_state(new_state.clone());
        Ok(new_state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    enum TestState {
        Initial,
        Active,
        Completed,
    }
    
    impl State for TestState {}
    
    #[derive(Clone, Debug)]
    enum TestCommand {
        Start,
        Complete,
    }
    
    impl Command for TestCommand {}
    
    #[test]
    fn test_state_machine_builder() {
        let sm = StateMachine::builder(TestState::Initial)
            .transition(TestState::Initial, TestCommand::Start, TestState::Active)
            .transition(TestState::Active, TestCommand::Complete, TestState::Completed)
            .build();
        
        // Test valid transition
        let result = sm.validate_transition(&TestState::Initial, &TestCommand::Start);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TestState::Active);
        
        // Test invalid transition
        let result = sm.validate_transition(&TestState::Initial, &TestCommand::Complete);
        assert!(result.is_err());
    }
}