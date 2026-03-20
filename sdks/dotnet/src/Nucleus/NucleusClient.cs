using Nucleus.Admin;

namespace Nucleus;

/// <summary>
/// Main entry point for the Nucleus .NET SDK.
/// Provides token verification and admin API access.
/// </summary>
public sealed class NucleusClient : IDisposable
{
    private readonly HttpClient _httpClient;
    private readonly string _baseUrl;
    private readonly NucleusTokenVerifier _verifier;

    private static bool _devWarned;

    public NucleusClient(string secretKey, string baseUrl = "https://api.nucleus.dev")
    {
        if (!_devWarned)
        {
            var version = typeof(NucleusClient).Assembly.GetName().Version?.ToString() ?? "0.1.0-dev.1";
            if (version.Contains("dev") || version == "0.0.0.0")
            {
                Console.Error.WriteLine($"[Nucleus] WARNING: You are using a dev preview ({version}). Do not use in production.");
            }
            _devWarned = true;
        }

        ArgumentException.ThrowIfNullOrWhiteSpace(secretKey);

        _baseUrl = baseUrl.TrimEnd('/');
        _httpClient = new HttpClient();
        _httpClient.DefaultRequestHeaders.Authorization =
            new System.Net.Http.Headers.AuthenticationHeaderValue("Bearer", secretKey);
        _httpClient.DefaultRequestHeaders.Add("Accept", "application/json");
        _verifier = new NucleusTokenVerifier(_baseUrl);

        Users = new UsersApi(_httpClient, _baseUrl);
        Orgs = new OrgsApi(_httpClient, _baseUrl);
    }

    /// <summary>Admin Users API.</summary>
    public UsersApi Users { get; }

    /// <summary>Admin Organizations API.</summary>
    public OrgsApi Orgs { get; }

    /// <summary>
    /// Verifies a Nucleus JWT and returns the parsed claims.
    /// </summary>
    public Task<NucleusClaims> VerifyTokenAsync(string token, CancellationToken cancellationToken = default) =>
        _verifier.VerifyAsync(token, cancellationToken);

    public void Dispose()
    {
        _httpClient.Dispose();
        _verifier.Dispose();
    }
}
