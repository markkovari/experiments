use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrgMember {
    pub user_id: ObjectId,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Organization {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub owner_id: ObjectId,
    pub members: Vec<OrgMember>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Invitation {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub org_id: ObjectId,
    pub inviter_id: ObjectId,
    pub invitee_email: String,
    pub status: InvitationStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Rejected,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrgRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct InviteRequest {
    pub email: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRoleRequest {
    pub role: String,
}
