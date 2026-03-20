using Microsoft.AspNetCore.Authentication;
using Microsoft.Extensions.DependencyInjection;

namespace Nucleus.AspNetCore;

/// <summary>
/// Extension methods for registering Nucleus services and authentication in ASP.NET Core.
/// </summary>
public static class NucleusExtensions
{
    /// <summary>
    /// Registers the Nucleus SDK services (NucleusClient, NucleusTokenVerifier) into DI.
    /// </summary>
    public static IServiceCollection AddNucleus(this IServiceCollection services, Action<NucleusOptions> configure)
    {
        var options = new NucleusOptions();
        configure(options);

        services.AddSingleton(options);
        services.AddSingleton(new NucleusTokenVerifier(options.BaseUrl));
        services.AddSingleton(new NucleusClient(options.SecretKey, options.BaseUrl));

        return services;
    }

    /// <summary>
    /// Adds the Nucleus JWT bearer authentication scheme.
    /// <example>
    /// <code>
    /// builder.Services.AddAuthentication().AddNucleus(opts =&gt; {
    ///     opts.BaseUrl = "https://api.nucleus.dev";
    /// });
    /// </code>
    /// </example>
    /// </summary>
    public static AuthenticationBuilder AddNucleus(
        this AuthenticationBuilder builder,
        Action<NucleusOptions>? configure = null)
    {
        return builder.AddNucleus(NucleusDefaults.AuthenticationScheme, configure);
    }

    /// <summary>
    /// Adds the Nucleus JWT bearer authentication scheme with a custom scheme name.
    /// </summary>
    public static AuthenticationBuilder AddNucleus(
        this AuthenticationBuilder builder,
        string authenticationScheme,
        Action<NucleusOptions>? configure = null)
    {
        if (configure is not null)
        {
            var options = new NucleusOptions();
            configure(options);

            // Ensure verifier is registered for the auth handler.
            builder.Services.AddSingleton(options);
            builder.Services.AddSingleton(new NucleusTokenVerifier(options.BaseUrl));
        }

        builder.AddScheme<NucleusAuthSchemeOptions, NucleusAuthHandler>(
            authenticationScheme, _ => { });

        return builder;
    }
}
