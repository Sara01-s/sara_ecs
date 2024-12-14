use thiserror::Error;

#[derive(Debug, Error)]
pub enum ECSError {
    #[error("Attempted to add to an entity without calling create entity first.")]
    CreateComponentNeverCalled,

    #[error("Attempted to reference a component that was not registered.")]
    ComponentNotRegistered,

    #[error("Attempted to reference an entity that does not exist.")]
    EntityDoesNotExist,

    #[error("Attempted to reference component data that does not exist.")]
    ComponentDoesNotExist,

    #[error("Attempted to downcast to the wrong type.")]
    DowncastToWrongType,
}
