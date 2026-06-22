/// APNs push notification sender.
/// Credentials are configured per-deployment; self-hosted instances
/// without APNs credentials will skip push silently.
pub struct ApnsConfig {
    pub team_id: String,
    pub key_id: String,
    pub private_key_pem: String,
    pub bundle_id: String,
    pub sandbox: bool,
}
