namespace Nucleus;

/// <summary>
/// Represents the JWT claims issued by Nucleus.
/// </summary>
public sealed record NucleusClaims
{
    /// <summary>The Nucleus user ID (JWT "sub" claim).</summary>
    public required string UserId { get; init; }

    /// <summary>JWT issuer.</summary>
    public string? Issuer { get; init; }

    /// <summary>Audience (project ID).</summary>
    public string? Audience { get; init; }

    /// <summary>Token expiration (Unix seconds).</summary>
    public long? Exp { get; init; }

    /// <summary>Token issued-at (Unix seconds).</summary>
    public long? Iat { get; init; }

    /// <summary>Unique token identifier.</summary>
    public string? Jti { get; init; }

    public string? Email { get; init; }
    public string? FirstName { get; init; }
    public string? LastName { get; init; }
    public string? AvatarUrl { get; init; }
    public bool EmailVerified { get; init; }
    public Dictionary<string, object>? Metadata { get; init; }
    public string? OrgId { get; init; }
    public string? OrgSlug { get; init; }
    public string? OrgRole { get; init; }
    public IReadOnlyList<string>? OrgPermissions { get; init; }
}
