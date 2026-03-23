package dev.nucleus;

import org.json.JSONObject;

public class NucleusUser {
    private final String id;
    private final String email;
    private final boolean emailVerified;
    private final String firstName;
    private final String lastName;
    private final String avatarUrl;
    private final String createdAt;

    public NucleusUser(String id, String email, boolean emailVerified,
                       String firstName, String lastName, String avatarUrl, String createdAt) {
        this.id = id;
        this.email = email;
        this.emailVerified = emailVerified;
        this.firstName = firstName;
        this.lastName = lastName;
        this.avatarUrl = avatarUrl;
        this.createdAt = createdAt;
    }

    public static NucleusUser fromJson(JSONObject json) {
        return new NucleusUser(
            json.optString("id"), json.optString("email"),
            json.optBoolean("email_verified", false),
            json.optString("first_name", null), json.optString("last_name", null),
            json.optString("avatar_url", null), json.optString("created_at")
        );
    }

    public String getId() { return id; }
    public String getEmail() { return email; }
    public boolean isEmailVerified() { return emailVerified; }
    public String getFirstName() { return firstName; }
    public String getLastName() { return lastName; }
    public String getAvatarUrl() { return avatarUrl; }
    public String getCreatedAt() { return createdAt; }
    public String getFullName() {
        StringBuilder sb = new StringBuilder();
        if (firstName != null) sb.append(firstName);
        if (lastName != null) { if (sb.length() > 0) sb.append(" "); sb.append(lastName); }
        return sb.toString();
    }
}
