#[derive(Debug, Clone)]
pub enum Action {
    AgentAddressChange(String),
    AgentTokenChange(String),
    AgentSaveDirChange(String),

    SaveSettings,
}
