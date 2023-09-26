use serde::{Deserialize, Serialize};

// ###########################################################################
// User Register Response.
// ###########################################################################

#[derive(Serialize, Deserialize)]
pub struct RegisterResponse {
    pub url: String
}

// ###########################################################################
// User Register Response.
// ###########################################################################

#[derive(Serialize, Deserialize)]
pub struct Register {
    pub user_id: u16
}