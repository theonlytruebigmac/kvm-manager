use std::sync::Arc;
use virt::connect::Connect;

/// Supplies a live libvirt handle captured for a single operation.
/// Implementations must not look up mutable connection selection while a command is running.
pub trait ConnectionProvider {
    fn get_connection(&self) -> &Connect;
}

impl ConnectionProvider for Connect {
    fn get_connection(&self) -> &Connect {
        self
    }
}

impl<T: ConnectionProvider + ?Sized> ConnectionProvider for Arc<T> {
    fn get_connection(&self) -> &Connect {
        self.as_ref().get_connection()
    }
}
