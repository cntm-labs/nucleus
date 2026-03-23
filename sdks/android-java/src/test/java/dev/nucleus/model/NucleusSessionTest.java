package dev.nucleus.model;

import org.json.JSONObject;
import org.junit.Test;
import static org.junit.Assert.*;

public class NucleusSessionTest {
    @Test
    public void fromJson_parsesAllFields() throws Exception {
        JSONObject json = new JSONObject();
        json.put("id", "sess_1");
        json.put("token", "tok_123");
        json.put("refresh_token", "ref_456");
        json.put("expires_at", "2024-12-31T23:59:59Z");
        json.put("user_id", "user_1");

        NucleusSession session = NucleusSession.fromJson(json);
        assertEquals("sess_1", session.getId());
        assertEquals("tok_123", session.getToken());
        assertEquals("ref_456", session.getRefreshToken());
        assertEquals("2024-12-31T23:59:59Z", session.getExpiresAt());
        assertEquals("user_1", session.getUserId());
    }
}
