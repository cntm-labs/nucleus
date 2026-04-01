using Xunit;

namespace Nucleus.Tests;

public class NucleusClaimsTests
{
    [Fact]
    public void Record_StoresAllFields()
    {
        var claims = new NucleusClaims
        {
            UserId = "user_123",
            Issuer = "https://api.test.com",
            Audience = "project_456",
            Exp = 1700000000,
            Iat = 1699996400,
            Jti = "jwt_abc",
            Email = "test@example.com",
            FirstName = "Test",
            LastName = "User",
            AvatarUrl = "https://img.test/a.png",
            EmailVerified = true,
            OrgId = "org_1",
            OrgSlug = "my-org",
            OrgRole = "admin",
            OrgPermissions = new List<string> { "read", "write" },
        };

        Assert.Equal("user_123", claims.UserId);
        Assert.Equal("project_456", claims.Audience);
        Assert.Equal("test@example.com", claims.Email);
        Assert.Equal("Test", claims.FirstName);
        Assert.Equal("User", claims.LastName);
        Assert.True(claims.EmailVerified);
        Assert.Equal("org_1", claims.OrgId);
        Assert.Equal("admin", claims.OrgRole);
        Assert.Equal(new[] { "read", "write" }, claims.OrgPermissions);
    }

    [Fact]
    public void Record_DefaultOptionalFieldsAreNull()
    {
        var claims = new NucleusClaims { UserId = "user_1" };

        Assert.Equal("user_1", claims.UserId);
        Assert.Null(claims.Email);
        Assert.Null(claims.FirstName);
        Assert.Null(claims.LastName);
        Assert.Null(claims.OrgId);
        Assert.Null(claims.OrgPermissions);
        Assert.Null(claims.Metadata);
        Assert.False(claims.EmailVerified);
    }

    [Fact]
    public void Record_SupportsEquality()
    {
        var a = new NucleusClaims { UserId = "user_1", Email = "a@test.com" };
        var b = new NucleusClaims { UserId = "user_1", Email = "a@test.com" };

        Assert.Equal(a, b);
    }

    [Fact]
    public void Record_InequalityOnDifferentValues()
    {
        var a = new NucleusClaims { UserId = "user_1" };
        var b = new NucleusClaims { UserId = "user_2" };

        Assert.NotEqual(a, b);
    }
}
