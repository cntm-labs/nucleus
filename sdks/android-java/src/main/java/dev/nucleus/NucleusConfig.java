package dev.nucleus;

public class NucleusConfig {
    private String apiBaseUrl = "https://api.nucleus.dev";
    private boolean biometricAuth = false;

    public static class Builder {
        private final NucleusConfig config = new NucleusConfig();
        public Builder apiBaseUrl(String url) { config.apiBaseUrl = url; return this; }
        public Builder biometricAuth(boolean enabled) { config.biometricAuth = enabled; return this; }
        public NucleusConfig build() { return config; }
    }

    public String getApiBaseUrl() { return apiBaseUrl; }
    public boolean isBiometricAuth() { return biometricAuth; }
}
