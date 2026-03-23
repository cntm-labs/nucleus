package dev.nucleus.model;

import org.json.JSONArray;
import org.json.JSONObject;
import java.util.ArrayList;
import java.util.List;

public class NucleusMember {
    private final String id, userId, orgId, role, email;
    private final List<String> permissions;
    private final String firstName, lastName;

    public NucleusMember(String id, String userId, String orgId, String role, String email,
                         List<String> permissions, String firstName, String lastName) {
        this.id = id; this.userId = userId; this.orgId = orgId; this.role = role;
        this.email = email; this.permissions = permissions;
        this.firstName = firstName; this.lastName = lastName;
    }

    public static NucleusMember fromJson(JSONObject json) {
        List<String> perms = new ArrayList<>();
        JSONArray arr = json.optJSONArray("permissions");
        if (arr != null) { for (int i = 0; i < arr.length(); i++) perms.add(arr.optString(i)); }
        return new NucleusMember(
            json.optString("id"), json.optString("user_id"), json.optString("org_id"),
            json.optString("role"), json.optString("email"), perms,
            json.optString("first_name", null), json.optString("last_name", null)
        );
    }

    public String getId() { return id; }
    public String getUserId() { return userId; }
    public String getOrgId() { return orgId; }
    public String getRole() { return role; }
    public String getEmail() { return email; }
    public List<String> getPermissions() { return permissions; }
    public String getFirstName() { return firstName; }
    public String getLastName() { return lastName; }
}
