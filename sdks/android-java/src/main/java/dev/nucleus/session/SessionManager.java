package dev.nucleus.session;

import android.os.Handler;
import android.os.Looper;
import dev.nucleus.NucleusCallback;
import dev.nucleus.NucleusException;
import dev.nucleus.model.NucleusSession;
import dev.nucleus.network.ApiClient;
import java.time.Instant;
import java.time.temporal.ChronoUnit;

public class SessionManager {
    private static final long REFRESH_BUFFER_MS = 60_000;
    private final ApiClient apiClient;
    private final TokenStorage tokenStorage;
    private final Handler handler = new Handler(Looper.getMainLooper());
    private Runnable scheduledRefresh;

    public SessionManager(ApiClient apiClient, TokenStorage tokenStorage) {
        this.apiClient = apiClient;
        this.tokenStorage = tokenStorage;
    }

    public void setSession(NucleusSession session) {
        tokenStorage.saveSession(session.getToken(), session.getRefreshToken(), session.getExpiresAt());
        apiClient.setToken(session.getToken());
        scheduleRefresh(session.getExpiresAt());
    }

    public void scheduleRefresh(String expiresAt) {
        cancelRefresh();
        try {
            long expiresMs = Instant.parse(expiresAt).toEpochMilli();
            long refreshAt = expiresMs - REFRESH_BUFFER_MS - System.currentTimeMillis();
            if (refreshAt <= 0) {
                doRefresh();
                return;
            }
            scheduledRefresh = this::doRefresh;
            handler.postDelayed(scheduledRefresh, refreshAt);
        } catch (Exception e) {
            // Invalid date format, skip scheduling
        }
    }

    private void doRefresh() {
        String refreshToken = tokenStorage.getRefreshToken();
        if (refreshToken == null) return;
        apiClient.refreshSession(refreshToken, new NucleusCallback<NucleusSession>() {
            @Override public void onSuccess(NucleusSession session) { setSession(session); }
            @Override public void onError(NucleusException error) { clear(); }
        });
    }

    public void cancelRefresh() {
        if (scheduledRefresh != null) { handler.removeCallbacks(scheduledRefresh); scheduledRefresh = null; }
    }

    public void clear() {
        cancelRefresh();
        tokenStorage.clear();
        apiClient.setToken(null);
    }
}
