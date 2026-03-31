package dev.nucleus;

import org.junit.jupiter.api.Test;
import java.util.List;
import java.util.Map;
import static org.junit.jupiter.api.Assertions.*;

class NucleusClaimsTest {

    @Test
    void recordStoresAllFields() {
        NucleusClaims claims = new NucleusClaims(
            "user_123", "project_456", "test@example.com",
            "Test", "User", "https://img.test/a.png",
            true, Map.of("role", "admin"),
            "org_1", "my-org", "admin", List.of("read", "write")
        );

        assertEquals("user_123", claims.userId());
        assertEquals("project_456", claims.projectId());
        assertEquals("test@example.com", claims.email());
        assertEquals("Test", claims.firstName());
        assertEquals("User", claims.lastName());
        assertEquals("https://img.test/a.png", claims.avatarUrl());
        assertTrue(claims.emailVerified());
        assertEquals("admin", claims.metadata().get("role"));
        assertEquals("org_1", claims.orgId());
        assertEquals("my-org", claims.orgSlug());
        assertEquals("admin", claims.orgRole());
        assertEquals(List.of("read", "write"), claims.orgPermissions());
    }

    @Test
    void getUserIdReturnsSubject() {
        NucleusClaims claims = new NucleusClaims(
            "user_abc", "proj_1", null, null, null, null,
            null, null, null, null, null, null
        );

        assertEquals("user_abc", claims.getUserId());
    }

    @Test
    void getPermissionsReturnsEmptyListWhenNull() {
        NucleusClaims claims = new NucleusClaims(
            "user_1", "proj_1", null, null, null, null,
            null, null, null, null, null, null
        );

        assertNotNull(claims.getPermissions());
        assertTrue(claims.getPermissions().isEmpty());
    }

    @Test
    void getPermissionsReturnsListWhenPresent() {
        NucleusClaims claims = new NucleusClaims(
            "user_1", "proj_1", null, null, null, null,
            null, null, "org_1", null, "admin", List.of("manage")
        );

        assertEquals(List.of("manage"), claims.getPermissions());
    }

    @Test
    void getOrgIdReturnsOrgId() {
        NucleusClaims claims = new NucleusClaims(
            "user_1", "proj_1", null, null, null, null,
            null, null, "org_abc", null, null, null
        );

        assertEquals("org_abc", claims.getOrgId());
    }

    @Test
    void getOrgRoleReturnsOrgRole() {
        NucleusClaims claims = new NucleusClaims(
            "user_1", "proj_1", null, null, null, null,
            null, null, null, null, "member", null
        );

        assertEquals("member", claims.getOrgRole());
    }
}
