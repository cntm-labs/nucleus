namespace Nucleus.AspNetCore;

/// <summary>
/// Configuration options for the Nucleus SDK.
/// </summary>
public sealed class NucleusOptions
{
    /// <summary>
    /// The secret key used for admin API calls.
    /// </summary>
    public string SecretKey { get; set; } = string.Empty;

    /// <summary>
    /// Base URL of the Nucleus API. Defaults to https://api.nucleus.dev.
    /// </summary>
    public string BaseUrl { get; set; } = "https://api.nucleus.dev";

    /// <summary>
    /// The authentication scheme name. Defaults to "Nucleus".
    /// </summary>
    public string AuthenticationScheme { get; set; } = NucleusDefaults.AuthenticationScheme;
}

public static class NucleusDefaults
{
    public const string AuthenticationScheme = "Nucleus";
}
