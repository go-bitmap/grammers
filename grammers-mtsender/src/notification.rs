/// Connection disconnection notification.
#[derive(Debug, Clone)]
pub struct DisconnectionNotification {
    /// Datacenter ID.
    pub dc_id: i32,
    /// Error message.
    pub error: String,
}
