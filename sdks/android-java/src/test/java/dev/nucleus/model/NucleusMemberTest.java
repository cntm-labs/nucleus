package dev.nucleus.model;

import org.json.JSONArray;
import org.json.JSONObject;
import org.junit.Test;
import static org.junit.Assert.*;

public class NucleusMemberTest {
    @Test
    public void fromJson_parsesPermissions() throws Exception {
        JSONObject json = new JSONObject();
        json.put("id", "m_1");
        json.put("user_id", "u_1");
        json.put("org_id", "o_1");
        json.put("role", "admin");
        json.put("email", "admin@test.com");
        json.put("permissions", new JSONArray().put("read").put("write").put("delete"));
        json.put("first_name", "John");
        json.put("last_name", "Doe");

        NucleusMember member = NucleusMember.fromJson(json);
        assertEquals("m_1", member.getId());
        assertEquals("admin", member.getRole());
        assertEquals(3, member.getPermissions().size());
        assertTrue(member.getPermissions().contains("write"));
    }

    @Test
    public void fromJson_handlesEmptyPermissions() throws Exception {
        JSONObject json = new JSONObject();
        json.put("id", "m_1");
        json.put("user_id", "u_1");
        json.put("org_id", "o_1");
        json.put("role", "member");
        json.put("email", "user@test.com");

        NucleusMember member = NucleusMember.fromJson(json);
        assertTrue(member.getPermissions().isEmpty());
    }
}
