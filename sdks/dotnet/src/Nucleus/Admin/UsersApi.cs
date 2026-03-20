using System.Net.Http.Json;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Nucleus.Admin;

/// <summary>
/// Admin API for managing Nucleus users.
/// </summary>
public sealed class UsersApi
{
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower,
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
    };

    private readonly HttpClient _http;
    private readonly string _baseUrl;

    internal UsersApi(HttpClient http, string baseUrl)
    {
        _http = http;
        _baseUrl = baseUrl;
    }

    /// <summary>Retrieve a user by ID.</summary>
    public async Task<NucleusUser> GetAsync(string userId, CancellationToken ct = default)
    {
        var res = await _http.GetAsync($"{_baseUrl}/api/v1/admin/users/{userId}", ct);
        await EnsureSuccessAsync(res);
        return (await res.Content.ReadFromJsonAsync<NucleusUser>(JsonOptions, ct))!;
    }

    /// <summary>List users with optional filtering and pagination.</summary>
    public async Task<PaginatedResponse<NucleusUser>> ListAsync(ListUsersParams? parameters = null, CancellationToken ct = default)
    {
        var query = new List<string>();
        if (parameters?.Limit is > 0) query.Add($"limit={parameters.Limit}");
        if (!string.IsNullOrEmpty(parameters?.Cursor)) query.Add($"cursor={Uri.EscapeDataString(parameters.Cursor)}");
        if (!string.IsNullOrEmpty(parameters?.EmailContains)) query.Add($"email_contains={Uri.EscapeDataString(parameters.EmailContains)}");

        var qs = query.Count > 0 ? "?" + string.Join("&", query) : "";
        var res = await _http.GetAsync($"{_baseUrl}/api/v1/admin/users{qs}", ct);
        await EnsureSuccessAsync(res);
        return (await res.Content.ReadFromJsonAsync<PaginatedResponse<NucleusUser>>(JsonOptions, ct))!;
    }

    /// <summary>Delete a user by ID.</summary>
    public async Task DeleteAsync(string userId, CancellationToken ct = default)
    {
        var res = await _http.DeleteAsync($"{_baseUrl}/api/v1/admin/users/{userId}", ct);
        await EnsureSuccessAsync(res);
    }

    /// <summary>Ban a user.</summary>
    public async Task BanAsync(string userId, CancellationToken ct = default)
    {
        var res = await _http.PostAsync($"{_baseUrl}/api/v1/admin/users/{userId}/ban", null, ct);
        await EnsureSuccessAsync(res);
    }

    /// <summary>Unban a user.</summary>
    public async Task UnbanAsync(string userId, CancellationToken ct = default)
    {
        var res = await _http.PostAsync($"{_baseUrl}/api/v1/admin/users/{userId}/unban", null, ct);
        await EnsureSuccessAsync(res);
    }

    private static async Task EnsureSuccessAsync(HttpResponseMessage response)
    {
        if (!response.IsSuccessStatusCode)
        {
            var body = await response.Content.ReadAsStringAsync();
            throw new NucleusApiException(response.StatusCode, body);
        }
    }
}

public sealed record NucleusUser
{
    public required string Id { get; init; }
    public required string Email { get; init; }
    public bool EmailVerified { get; init; }
    public string? Username { get; init; }
    public string? FirstName { get; init; }
    public string? LastName { get; init; }
    public string? AvatarUrl { get; init; }
    public Dictionary<string, object>? Metadata { get; init; }
    public string? CreatedAt { get; init; }
    public string? UpdatedAt { get; init; }
}

public sealed record PaginatedResponse<T>
{
    public required IReadOnlyList<T> Data { get; init; }
    public bool HasMore { get; init; }
    public string? NextCursor { get; init; }
    public int? TotalCount { get; init; }
}

public sealed record ListUsersParams
{
    public int? Limit { get; init; }
    public string? Cursor { get; init; }
    public string? EmailContains { get; init; }
}

public sealed class NucleusApiException : Exception
{
    public System.Net.HttpStatusCode StatusCode { get; }
    public string ResponseBody { get; }

    public NucleusApiException(System.Net.HttpStatusCode statusCode, string responseBody)
        : base($"Nucleus API error: {(int)statusCode} — {responseBody}")
    {
        StatusCode = statusCode;
        ResponseBody = responseBody;
    }
}
