package dev.nucleus.auth;

import android.content.Context;
import android.content.Intent;
import android.net.Uri;
import dev.nucleus.NucleusCallback;
import dev.nucleus.NucleusException;
import dev.nucleus.network.ApiClient;
import androidx.browser.customtabs.CustomTabsIntent;

public class OAuthManager {
    private final ApiClient apiClient;

    public OAuthManager(ApiClient apiClient) {
        this.apiClient = apiClient;
    }

    public void launchOAuth(Context context, String provider, String redirectUri) {
        String url = apiClient.getOAuthUrl(provider, redirectUri);
        CustomTabsIntent customTabsIntent = new CustomTabsIntent.Builder().build();
        customTabsIntent.launchUrl(context, Uri.parse(url));
    }

    public void handleCallback(String code, String redirectUri, NucleusCallback<ApiClient.AuthResult> cb) {
        apiClient.exchangeOAuthCode(code, redirectUri, cb);
    }
}
