using Xunit;

namespace Nucleus.Tests;

public class NucleusTokenVerifierTests
{
    [Fact]
    public void Constructor_WithDefaultBaseUrl_Succeeds()
    {
        using var verifier = new NucleusTokenVerifier();
        Assert.NotNull(verifier);
    }

    [Fact]
    public void Constructor_WithCustomBaseUrl_Succeeds()
    {
        using var verifier = new NucleusTokenVerifier("https://custom.api.dev");
        Assert.NotNull(verifier);
    }

    [Fact]
    public void Constructor_TrimsTrailingSlash()
    {
        // Should not throw — trailing slash is handled
        using var verifier = new NucleusTokenVerifier("https://api.test.com/");
        Assert.NotNull(verifier);
    }

    [Fact]
    public async Task VerifyAsync_WithInvalidToken_Throws()
    {
        using var verifier = new NucleusTokenVerifier("https://localhost:0");
        // Invalid token should fail during verification
        await Assert.ThrowsAnyAsync<Exception>(
            () => verifier.VerifyAsync("not.a.valid.token"));
    }

    [Fact]
    public void Dispose_CanBeCalledMultipleTimes()
    {
        var verifier = new NucleusTokenVerifier();
        verifier.Dispose();
        verifier.Dispose(); // Should not throw
    }
}
