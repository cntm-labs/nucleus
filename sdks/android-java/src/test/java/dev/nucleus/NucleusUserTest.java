package dev.nucleus;

import org.json.JSONObject;
import org.junit.Test;
import static org.junit.Assert.*;

public class NucleusUserTest {
    @Test
    public void fromJson_parsesAllFields() throws Exception {
        JSONObject json = new JSONObject();
        json.put("id", "user_1");
        json.put("email", "test@example.com");
        json.put("email_verified", true);
        json.put("first_name", "John");
        json.put("last_name", "Doe");
        json.put("avatar_url", "https://example.com/avatar.jpg");
        json.put("created_at", "2024-01-01T00:00:00Z");

        NucleusUser user = NucleusUser.fromJson(json);
        assertEquals("user_1", user.getId());
        assertEquals("test@example.com", user.getEmail());
        assertTrue(user.isEmailVerified());
        assertEquals("John", user.getFirstName());
        assertEquals("Doe", user.getLastName());
    }

    @Test
    public void getFullName_returnsCombinedName() {
        NucleusUser user = new NucleusUser("u", "e", true, "John", "Doe", null, null);
        assertEquals("John Doe", user.getFullName().trim());
    }

    @Test
    public void getFullName_handlesNullNames() {
        NucleusUser user = new NucleusUser("u", "e", false, null, null, null, null);
        assertEquals("", user.getFullName().trim());
    }
}
