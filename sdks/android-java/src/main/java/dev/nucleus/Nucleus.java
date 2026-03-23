package dev.nucleus;

import android.content.Context;
import dev.nucleus.network.ApiClient;
import dev.nucleus.session.TokenStorage;
import dev.nucleus.session.SessionManager;
import dev.nucleus.model.NucleusSession;

public final class Nucleus {
    private static ApiClient apiClient;
    private static NucleusAuth auth;
    private static TokenStorage tokenStorage;
    private static SessionManager sessionManager;
    private static boolean configured = false;

    private Nucleus() {}

    public static void configure(Context context, String publishableKey) {
        configure(context, publishableKey, "https://api.nucleus.dev");
    }

    public static void configure(Context context, String publishableKey, String baseUrl) {
        String version = "0.1.0-dev.1";
        if (version.contains("dev")) {
            android.util.Log.w("Nucleus", "WARNING: You are using a dev preview (" + version + "). Do not use in production.");
        }
        apiClient = new ApiClient(publishableKey, baseUrl);
        tokenStorage = new TokenStorage(context);
        sessionManager = new SessionManager(apiClient, tokenStorage);
        auth = new NucleusAuth(apiClient, sessionManager);
        configured = true;

        // Restore session
        String token = tokenStorage.getToken();
        String refreshToken = tokenStorage.getRefreshToken();
        String expiresAt = tokenStorage.getExpiresAt();
        if (token != null && refreshToken != null) {
            apiClient.setToken(token);
            NucleusSession session = new NucleusSession("", token, refreshToken, expiresAt != null ? expiresAt : "", "");
            auth.setSession(session);
            if (expiresAt != null) sessionManager.scheduleRefresh(expiresAt);
            // Load user async
            apiClient.getUser(new NucleusCallback<NucleusUser>() {
                @Override public void onSuccess(NucleusUser user) { auth.setUser(user); }
                @Override public void onError(NucleusException error) { sessionManager.clear(); }
            });
        }
    }

    public static NucleusAuth getAuth() {
        if (!configured) throw new IllegalStateException("Call Nucleus.configure() first");
        return auth;
    }

    public static ApiClient getApiClient() {
        if (!configured) throw new IllegalStateException("Call Nucleus.configure() first");
        return apiClient;
    }

    public static boolean isConfigured() { return configured; }
}
