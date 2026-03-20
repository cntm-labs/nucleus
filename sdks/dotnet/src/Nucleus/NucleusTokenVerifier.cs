using System.IdentityModel.Tokens.Jwt;
using System.Text.Json;
using Microsoft.IdentityModel.Protocols;
using Microsoft.IdentityModel.Protocols.OpenIdConnect;
using Microsoft.IdentityModel.Tokens;

namespace Nucleus;

/// <summary>
/// Verifies Nucleus JWTs against the JWKS endpoint.
/// </summary>
public sealed class NucleusTokenVerifier : IDisposable
{
    private readonly ConfigurationManager<OpenIdConnectConfiguration> _configManager;
    private readonly JwtSecurityTokenHandler _handler = new();
    private readonly string _baseUrl;

    public NucleusTokenVerifier(string baseUrl = "https://api.nucleus.dev")
    {
        _baseUrl = baseUrl.TrimEnd('/');
        var jwksUrl = $"{_baseUrl}/.well-known/jwks.json";

        _configManager = new ConfigurationManager<OpenIdConnectConfiguration>(
            $"{_baseUrl}/.well-known/openid-configuration",
            new OpenIdConnectConfigurationRetriever(),
            new HttpDocumentRetriever());

        // Allow direct JWKS usage via metadata address fallback.
        _handler.InboundClaimTypeMap.Clear();
    }

    /// <summary>
    /// Verifies a JWT token and returns the parsed Nucleus claims.
    /// </summary>
    public async Task<NucleusClaims> VerifyAsync(string token, CancellationToken cancellationToken = default)
    {
        var config = await _configManager.GetConfigurationAsync(cancellationToken);

        var validationParameters = new TokenValidationParameters
        {
            ValidateIssuerSigningKey = true,
            IssuerSigningKeys = config.SigningKeys,
            ValidateIssuer = false,
            ValidateAudience = false,
            ValidateLifetime = true,
            ValidAlgorithms = new[] { SecurityAlgorithms.RsaSha256 },
            ClockSkew = TimeSpan.FromSeconds(30),
        };

        var result = await _handler.ValidateTokenAsync(token, validationParameters);

        if (!result.IsValid)
            throw new SecurityTokenValidationException("Token validation failed.", result.Exception);

        return MapClaims(result.Claims);
    }

    private static NucleusClaims MapClaims(IDictionary<string, object> claims)
    {
        return new NucleusClaims
        {
            UserId = GetString(claims, "sub") ?? throw new SecurityTokenValidationException("Missing 'sub' claim."),
            Issuer = GetString(claims, "iss"),
            Audience = GetString(claims, "aud"),
            Exp = GetLong(claims, "exp"),
            Iat = GetLong(claims, "iat"),
            Jti = GetString(claims, "jti"),
            Email = GetString(claims, "email"),
            FirstName = GetString(claims, "first_name"),
            LastName = GetString(claims, "last_name"),
            AvatarUrl = GetString(claims, "avatar_url"),
            EmailVerified = GetBool(claims, "email_verified"),
            Metadata = GetDictionary(claims, "metadata"),
            OrgId = GetString(claims, "org_id"),
            OrgSlug = GetString(claims, "org_slug"),
            OrgRole = GetString(claims, "org_role"),
            OrgPermissions = GetStringList(claims, "org_permissions"),
        };
    }

    private static string? GetString(IDictionary<string, object> claims, string key) =>
        claims.TryGetValue(key, out var v) ? v?.ToString() : null;

    private static long? GetLong(IDictionary<string, object> claims, string key) =>
        claims.TryGetValue(key, out var v) && v is long l ? l
        : claims.TryGetValue(key, out var v2) && long.TryParse(v2?.ToString(), out var parsed) ? parsed
        : null;

    private static bool GetBool(IDictionary<string, object> claims, string key) =>
        claims.TryGetValue(key, out var v) && v is bool b && b;

    private static Dictionary<string, object>? GetDictionary(IDictionary<string, object> claims, string key)
    {
        if (!claims.TryGetValue(key, out var v) || v is null) return null;
        if (v is JsonElement el && el.ValueKind == JsonValueKind.Object)
            return JsonSerializer.Deserialize<Dictionary<string, object>>(el.GetRawText());
        return null;
    }

    private static IReadOnlyList<string>? GetStringList(IDictionary<string, object> claims, string key)
    {
        if (!claims.TryGetValue(key, out var v) || v is null) return null;
        if (v is JsonElement el && el.ValueKind == JsonValueKind.Array)
            return JsonSerializer.Deserialize<List<string>>(el.GetRawText());
        return null;
    }

    public void Dispose() { }
}
