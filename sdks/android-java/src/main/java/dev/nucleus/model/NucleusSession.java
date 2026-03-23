package dev.nucleus.model;

import org.json.JSONObject;

public class NucleusSession {
    private final String id;
    private final String token;
    private final String refreshToken;
    private final String expiresAt;
    private final String userId;

    public NucleusSession(String id, String token, String refreshToken, String expiresAt, String userId) {
        this.id = id; this.token = token; this.refreshToken = refreshToken;
        this.expiresAt = expiresAt; this.userId = userId;
    }

    public static NucleusSession fromJson(JSONObject json) {
        return new NucleusSession(
            json.optString("id"), json.optString("token"),
            json.optString("refresh_token"), json.optString("expires_at"),
            json.optString("user_id")
        );
    }

    public String getId() { return id; }
    public String getToken() { return token; }
    public String getRefreshToken() { return refreshToken; }
    public String getExpiresAt() { return expiresAt; }
    public String getUserId() { return userId; }
}
