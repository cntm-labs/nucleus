package dev.nucleus.model;

import org.json.JSONObject;
import org.junit.Test;
import static org.junit.Assert.*;

public class NucleusOrganizationTest {
    @Test
    public void fromJson_parsesAllFields() throws Exception {
        JSONObject json = new JSONObject();
        json.put("id", "org_1");
        json.put("name", "Test Org");
        json.put("slug", "test-org");
        json.put("created_at", "2024-01-01T00:00:00Z");

        NucleusOrganization org = NucleusOrganization.fromJson(json);
        assertEquals("org_1", org.getId());
        assertEquals("Test Org", org.getName());
        assertEquals("test-org", org.getSlug());
    }
}
