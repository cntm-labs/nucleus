using Xunit;

namespace Nucleus.Tests;

public class NucleusClientTests
{
    [Fact]
    public void Constructor_WithValidSecretKey_Succeeds()
    {
        using var client = new NucleusClient("sk_test_123");
        Assert.NotNull(client);
    }

    [Fact]
    public void Constructor_WithCustomBaseUrl_Succeeds()
    {
        using var client = new NucleusClient("sk_test_123", "https://custom.api.dev");
        Assert.NotNull(client);
    }

    [Fact]
    public void Constructor_WithNullSecretKey_Throws()
    {
        Assert.ThrowsAny<ArgumentException>(() => new NucleusClient(null!));
    }

    [Fact]
    public void Constructor_WithEmptySecretKey_Throws()
    {
        Assert.Throws<ArgumentException>(() => new NucleusClient(""));
    }

    [Fact]
    public void Constructor_WithWhitespaceSecretKey_Throws()
    {
        Assert.Throws<ArgumentException>(() => new NucleusClient("   "));
    }

    [Fact]
    public void Users_ReturnsNonNull()
    {
        using var client = new NucleusClient("sk_test_123");
        Assert.NotNull(client.Users);
    }

    [Fact]
    public void Orgs_ReturnsNonNull()
    {
        using var client = new NucleusClient("sk_test_123");
        Assert.NotNull(client.Orgs);
    }

    [Fact]
    public void Dispose_CanBeCalledMultipleTimes()
    {
        var client = new NucleusClient("sk_test_123");
        client.Dispose();
        client.Dispose(); // Should not throw
    }
}
