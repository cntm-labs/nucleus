package dev.nucleus;

import dev.nucleus.admin.UsersApi;
import dev.nucleus.admin.OrgsApi;

public class NucleusClient {
    private final String secretKey;
    private final String baseUrl;

    private NucleusClient(Builder builder) {
        this.secretKey = builder.secretKey;
        this.baseUrl = builder.baseUrl;
    }

    public UsersApi users() { return new UsersApi(baseUrl, secretKey); }
    public OrgsApi organizations() { return new OrgsApi(baseUrl, secretKey); }

    public NucleusClaims verifyToken(String jwt) { return NucleusTokenVerifier.verify(jwt, baseUrl); }

    public static Builder builder() { return new Builder(); }

    public static class Builder {
        private String secretKey;
        private String baseUrl = "https://api.nucleus.dev";
        public Builder secretKey(String key) { this.secretKey = key; return this; }
        public Builder baseUrl(String url) { this.baseUrl = url; return this; }
        public NucleusClient build() { return new NucleusClient(this); }
    }
}
