package dev.nucleus.model;

import org.json.JSONObject;

public class NucleusInvitation {
    private final String id, orgId, email, role, status, createdAt;

    public NucleusInvitation(String id, String orgId, String email, String role, String status, String createdAt) {
        this.id = id; this.orgId = orgId; this.email = email;
        this.role = role; this.status = status; this.createdAt = createdAt;
    }

    public static NucleusInvitation fromJson(JSONObject json) {
        return new NucleusInvitation(
            json.optString("id"), json.optString("org_id"), json.optString("email"),
            json.optString("role"), json.optString("status"), json.optString("created_at")
        );
    }

    public String getId() { return id; }
    public String getOrgId() { return orgId; }
    public String getEmail() { return email; }
    public String getRole() { return role; }
    public String getStatus() { return status; }
    public String getCreatedAt() { return createdAt; }
}
