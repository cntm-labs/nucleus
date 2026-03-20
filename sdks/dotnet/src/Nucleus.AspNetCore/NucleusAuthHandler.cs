using System.Security.Claims;
using System.Text.Encodings.Web;
using Microsoft.AspNetCore.Authentication;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Options;

namespace Nucleus.AspNetCore;

/// <summary>
/// ASP.NET Core authentication handler that validates Nucleus JWTs.
/// </summary>
public sealed class NucleusAuthHandler : AuthenticationHandler<NucleusAuthSchemeOptions>
{
    private readonly NucleusTokenVerifier _verifier;

    public NucleusAuthHandler(
        IOptionsMonitor<NucleusAuthSchemeOptions> options,
        ILoggerFactory logger,
        UrlEncoder encoder,
        NucleusTokenVerifier verifier)
        : base(options, logger, encoder)
    {
        _verifier = verifier;
    }

    protected override async Task<AuthenticateResult> HandleAuthenticateAsync()
    {
        var authorization = Request.Headers.Authorization.ToString();
        if (string.IsNullOrEmpty(authorization))
            return AuthenticateResult.NoResult();

        string token;
        if (authorization.StartsWith("Bearer ", StringComparison.OrdinalIgnoreCase))
            token = authorization["Bearer ".Length..].Trim();
        else
            return AuthenticateResult.NoResult();

        if (string.IsNullOrEmpty(token))
            return AuthenticateResult.NoResult();

        try
        {
            var nucleusClaims = await _verifier.VerifyAsync(token, Context.RequestAborted);

            var claims = new List<Claim>
            {
                new(ClaimTypes.NameIdentifier, nucleusClaims.UserId),
                new("nucleus:user_id", nucleusClaims.UserId),
            };

            if (nucleusClaims.Email is not null)
                claims.Add(new Claim(ClaimTypes.Email, nucleusClaims.Email));
            if (nucleusClaims.FirstName is not null)
                claims.Add(new Claim("nucleus:first_name", nucleusClaims.FirstName));
            if (nucleusClaims.LastName is not null)
                claims.Add(new Claim("nucleus:last_name", nucleusClaims.LastName));
            if (nucleusClaims.AvatarUrl is not null)
                claims.Add(new Claim("nucleus:avatar_url", nucleusClaims.AvatarUrl));
            if (nucleusClaims.OrgId is not null)
                claims.Add(new Claim("nucleus:org_id", nucleusClaims.OrgId));
            if (nucleusClaims.OrgSlug is not null)
                claims.Add(new Claim("nucleus:org_slug", nucleusClaims.OrgSlug));
            if (nucleusClaims.OrgRole is not null)
                claims.Add(new Claim("nucleus:org_role", nucleusClaims.OrgRole));
            if (nucleusClaims.OrgPermissions is not null)
                foreach (var perm in nucleusClaims.OrgPermissions)
                    claims.Add(new Claim("nucleus:org_permission", perm));

            claims.Add(new Claim("nucleus:email_verified", nucleusClaims.EmailVerified.ToString().ToLowerInvariant()));

            // Store the full NucleusClaims object in HttpContext.Items for easy access.
            Context.Items["NucleusClaims"] = nucleusClaims;

            var identity = new ClaimsIdentity(claims, Scheme.Name);
            var principal = new ClaimsPrincipal(identity);
            var ticket = new AuthenticationTicket(principal, Scheme.Name);

            return AuthenticateResult.Success(ticket);
        }
        catch (Exception ex)
        {
            Logger.LogDebug(ex, "Nucleus token validation failed.");
            return AuthenticateResult.Fail("Invalid Nucleus token.");
        }
    }
}

/// <summary>
/// Options for the Nucleus authentication scheme (extends AuthenticationSchemeOptions).
/// </summary>
public sealed class NucleusAuthSchemeOptions : AuthenticationSchemeOptions
{
}
