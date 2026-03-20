using System.Net.Http.Json;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Nucleus.Admin;

/// <summary>
/// Admin API for managing Nucleus organizations.
/// </summary>
public sealed class OrgsApi
{
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower,
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
    };

    private readonly HttpClient _http;
    private readonly string _baseUrl;

    internal OrgsApi(HttpClient http, string baseUrl)
    {
        _http = http;
        _baseUrl = baseUrl;
    }

    /// <summary>Retrieve an organization by ID.</summary>
    public async Task<NucleusOrg> GetAsync(string orgId, CancellationToken ct = default)
    {
        var res = await _http.GetAsync($"{_baseUrl}/api/v1/admin/orgs/{orgId}", ct);
        await EnsureSuccessAsync(res);
        return (await res.Content.ReadFromJsonAsync<NucleusOrg>(JsonOptions, ct))!;
    }

    /// <summary>List organizations with optional pagination.</summary>
    public async Task<PaginatedResponse<NucleusOrg>> ListAsync(ListOrgsParams? parameters = null, CancellationToken ct = default)
    {
        var query = new List<string>();
        if (parameters?.Limit is > 0) query.Add($"limit={parameters.Limit}");
        if (!string.IsNullOrEmpty(parameters?.Cursor)) query.Add($"cursor={Uri.EscapeDataString(parameters.Cursor)}");

        var qs = query.Count > 0 ? "?" + string.Join("&", query) : "";
        var res = await _http.GetAsync($"{_baseUrl}/api/v1/admin/orgs{qs}", ct);
        await EnsureSuccessAsync(res);
        return (await res.Content.ReadFromJsonAsync<PaginatedResponse<NucleusOrg>>(JsonOptions, ct))!;
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

public sealed record NucleusOrg
{
    public required string Id { get; init; }
    public required string Name { get; init; }
    public string? Slug { get; init; }
    public string? LogoUrl { get; init; }
    public string? CreatedAt { get; init; }
    public string? UpdatedAt { get; init; }
}

public sealed record ListOrgsParams
{
    public int? Limit { get; init; }
    public string? Cursor { get; init; }
}
