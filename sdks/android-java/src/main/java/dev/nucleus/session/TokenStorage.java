package dev.nucleus.session;

import android.content.Context;
import android.content.SharedPreferences;
import androidx.security.crypto.EncryptedSharedPreferences;
import androidx.security.crypto.MasterKey;

public class TokenStorage {
    private static final String PREFS_NAME = "nucleus_auth";
    private static final String KEY_TOKEN = "session_token";
    private static final String KEY_REFRESH = "refresh_token";
    private static final String KEY_EXPIRES = "expires_at";

    private final SharedPreferences prefs;

    public TokenStorage(Context context) {
        try {
            MasterKey masterKey = new MasterKey.Builder(context).setKeyScheme(MasterKey.KeyScheme.AES256_GCM).build();
            prefs = EncryptedSharedPreferences.create(
                context, PREFS_NAME, masterKey,
                EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
                EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM
            );
        } catch (Exception e) {
            throw new RuntimeException("Failed to create encrypted storage", e);
        }
    }

    public void saveSession(String token, String refreshToken, String expiresAt) {
        prefs.edit()
            .putString(KEY_TOKEN, token)
            .putString(KEY_REFRESH, refreshToken)
            .putString(KEY_EXPIRES, expiresAt)
            .apply();
    }

    public String getToken() { return prefs.getString(KEY_TOKEN, null); }
    public String getRefreshToken() { return prefs.getString(KEY_REFRESH, null); }
    public String getExpiresAt() { return prefs.getString(KEY_EXPIRES, null); }

    public void clear() {
        prefs.edit().remove(KEY_TOKEN).remove(KEY_REFRESH).remove(KEY_EXPIRES).apply();
    }
}
