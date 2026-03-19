package dev.nucleus;

import java.util.List;
import java.util.Map;

public record NucleusClaims(
    String userId, String projectId, String email,
    String firstName, String lastName, String avatarUrl,
    Boolean emailVerified, Map<String, Object> metadata,
    String orgId, String orgSlug, String orgRole,
    List<String> orgPermissions
) {
    public String getUserId() { return userId; }
    public String getOrgId() { return orgId; }
    public String getOrgRole() { return orgRole; }
    public List<String> getPermissions() { return orgPermissions != null ? orgPermissions : List.of(); }
}
