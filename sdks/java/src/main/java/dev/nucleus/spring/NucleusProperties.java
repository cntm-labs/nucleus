package dev.nucleus.spring;

import org.springframework.boot.context.properties.ConfigurationProperties;

@ConfigurationProperties(prefix = "nucleus")
public class NucleusProperties {
    private String secretKey;
    private String baseUrl = "https://api.nucleus.dev";

    public String getSecretKey() { return secretKey; }
    public void setSecretKey(String k) { this.secretKey = k; }
    public String getBaseUrl() { return baseUrl; }
    public void setBaseUrl(String u) { this.baseUrl = u; }
}
