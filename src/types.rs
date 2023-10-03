use serde::{Deserialize, Serialize};

// ###########################################################################
// User Register Response.
// ###########################################################################

#[derive(Serialize, Deserialize)]
pub struct RegisterResponse {
    pub url: String,
}

// ###########################################################################
// User Register Response.
// ###########################################################################

#[derive(Serialize, Deserialize)]
pub struct Register {
    pub user_id: u16,
}

// ###########################################################################
// Webex types.
// ###########################################################################

#[derive(Debug, Deserialize)]
pub struct MessageEventResponse {
    pub id: String,
    #[serde(alias = "roomId")]
    pub room_id: String,
    #[serde(alias = "roomType")]
    pub room_type: String,
    #[serde(alias = "personId")]
    pub person_id: String,
    #[serde(alias = "personEmail")]
    pub person_email: String,
    #[serde(alias = "mentionedPeople")]
    pub mentioned_people: Box<[String]>,
    pub created: String,
}

#[derive(Deserialize, Debug)]
pub struct Response<T> {
    pub id: String,
    pub name: String,
    #[serde(alias = "targetUrl")]
    pub target_url: String,
    pub resource: String,
    pub event: String,
    pub created: String,
    #[serde(alias = "actorId")]
    pub actor_id: String,
    pub data: T,
}
