using Microsoft.AspNetCore.Http;

namespace Nucleus.AspNetCore;

/// <summary>
/// Extension methods for extracting Nucleus claims from HttpContext.
/// </summary>
public static class ClaimsExtensions
{
    /// <summary>
    /// Returns the <see cref="NucleusClaims"/> for the current request,
    /// or <c>null</c> if the user is not authenticated via Nucleus.
    /// </summary>
    public static NucleusClaims? GetNucleusClaims(this HttpContext context)
    {
        return context.Items.TryGetValue("NucleusClaims", out var claims)
            ? claims as NucleusClaims
            : null;
    }

    /// <summary>
    /// Returns the Nucleus user ID from the current request, or <c>null</c> if not authenticated.
    /// </summary>
    public static string? GetNucleusUserId(this HttpContext context)
    {
        return context.GetNucleusClaims()?.UserId;
    }

    /// <summary>
    /// Returns the Nucleus org ID from the current request, or <c>null</c> if not present.
    /// </summary>
    public static string? GetNucleusOrgId(this HttpContext context)
    {
        return context.GetNucleusClaims()?.OrgId;
    }
}
