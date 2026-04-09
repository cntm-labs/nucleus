package dev.nucleus.network;

import org.json.JSONObject;
import org.json.JSONArray;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.Executor;
import java.util.concurrent.Executors;
import dev.nucleus.NucleusCallback;
import dev.nucleus.NucleusException;
import dev.nucleus.NucleusUser;
import dev.nucleus.model.*;
import okhttp3.*;

public class ApiClient {
    private static final MediaType JSON = MediaType.get("application/json; charset=utf-8");
    private final OkHttpClient client;
    private final String baseUrl;
    private final String publishableKey;
    private String accessToken;
    private final Executor executor = Executors.newCachedThreadPool();

    public ApiClient(String publishableKey, String baseUrl) {
        this.publishableKey = publishableKey;
        this.baseUrl = baseUrl.replaceAll("/$", "");
        this.client = new OkHttpClient();
    }

    public void setToken(String token) { this.accessToken = token; }

    private Request.Builder requestBuilder(String path) {
        Request.Builder b = new Request.Builder()
            .url(baseUrl + path)
            .header("Content-Type", "application/json")
            .header("X-Nucleus-Publishable-Key", publishableKey);
        if (accessToken != null) b.header("Authorization", "Bearer " + accessToken);
        return b;
    }

    private JSONObject execute(Request request) throws NucleusException {
        try (Response response = client.newCall(request).execute()) {
            String body = response.body() != null ? response.body().string() : "";
            if (!response.isSuccessful()) throw new NucleusException("API error (" + response.code() + "): " + body);
            if (body.isEmpty() || response.code() == 204) return new JSONObject();
            return new JSONObject(body);
        } catch (NucleusException e) { throw e; }
        catch (Exception e) { throw new NucleusException("Network error", e); }
    }

    private JSONObject post(String path, JSONObject body) throws NucleusException {
        Request req = requestBuilder(path).post(RequestBody.create(body.toString(), JSON)).build();
        return execute(req);
    }

    private JSONObject get(String path) throws NucleusException {
        return execute(requestBuilder(path).get().build());
    }

    private JSONObject put(String path, JSONObject body) throws NucleusException {
        return execute(requestBuilder(path).put(RequestBody.create(body.toString(), JSON)).build());
    }

    private JSONObject patch(String path, JSONObject body) throws NucleusException {
        return execute(requestBuilder(path).patch(RequestBody.create(body.toString(), JSON)).build());
    }

    private void delete(String path) throws NucleusException {
        execute(requestBuilder(path).delete().build());
    }

    // --- Async wrapper ---
    private <T> void async(NucleusCallback<T> cb, AsyncTask<T> task) {
        executor.execute(() -> {
            try { T result = task.run(); cb.onSuccess(result); }
            catch (NucleusException e) { cb.onError(e); }
            catch (org.json.JSONException e) { cb.onError(new NucleusException("JSON parse error: " + e.getMessage(), e)); }
        });
    }

    @FunctionalInterface
    interface AsyncTask<T> { T run() throws NucleusException, org.json.JSONException; }

    // --- Auth ---
    public void signIn(String email, String password, NucleusCallback<AuthResult> cb) {
        async(cb, () -> {
            JSONObject body = new JSONObject().put("email", email).put("password", password);
            JSONObject json = post("/v1/auth/sign-in", body);
            return new AuthResult(NucleusUser.fromJson(json.getJSONObject("user")), NucleusSession.fromJson(json.getJSONObject("session")));
        });
    }

    public void signUp(String email, String password, String firstName, String lastName, NucleusCallback<AuthResult> cb) {
        async(cb, () -> {
            JSONObject body = new JSONObject().put("email", email).put("password", password);
            if (firstName != null) body.put("first_name", firstName);
            if (lastName != null) body.put("last_name", lastName);
            JSONObject json = post("/v1/auth/sign-up", body);
            return new AuthResult(NucleusUser.fromJson(json.getJSONObject("user")), NucleusSession.fromJson(json.getJSONObject("session")));
        });
    }

    public void signOut(NucleusCallback<Void> cb) {
        async(cb, () -> { post("/v1/auth/sign-out", new JSONObject()); return null; });
    }

    // --- OAuth ---
    public String getOAuthUrl(String provider, String redirectUri) {
        return baseUrl + "/v1/oauth/" + provider + "/authorize?redirect_uri=" + redirectUri + "&publishable_key=" + publishableKey;
    }

    public void exchangeOAuthCode(String code, String redirectUri, NucleusCallback<AuthResult> cb) {
        async(cb, () -> {
            JSONObject body = new JSONObject().put("code", code).put("redirect_uri", redirectUri);
            JSONObject json = post("/v1/oauth/token", body);
            return new AuthResult(NucleusUser.fromJson(json.getJSONObject("user")), NucleusSession.fromJson(json.getJSONObject("session")));
        });
    }

    // --- MFA ---
    public void mfaTotpSetup(NucleusCallback<MfaSetupResult> cb) {
        async(cb, () -> {
            JSONObject json = post("/v1/auth/mfa/totp/setup", new JSONObject());
            return new MfaSetupResult(json.getString("secret"), json.getString("qr_uri"));
        });
    }

    public void mfaTotpVerify(String code, NucleusCallback<Boolean> cb) {
        async(cb, () -> post("/v1/auth/mfa/totp/verify", new JSONObject().put("code", code)).getBoolean("verified"));
    }

    public void mfaSmsSend(String phone, NucleusCallback<Void> cb) {
        async(cb, () -> { post("/v1/auth/mfa/sms/send", new JSONObject().put("phone", phone)); return null; });
    }

    public void mfaSmsVerify(String code, NucleusCallback<Boolean> cb) {
        async(cb, () -> post("/v1/auth/mfa/sms/verify", new JSONObject().put("code", code)).getBoolean("verified"));
    }

    // --- User ---
    public void getUser(NucleusCallback<NucleusUser> cb) {
        async(cb, () -> NucleusUser.fromJson(get("/v1/user")));
    }

    public NucleusUser getUserSync() throws NucleusException {
        return NucleusUser.fromJson(get("/v1/user"));
    }

    public void updateUser(String firstName, String lastName, NucleusCallback<NucleusUser> cb) {
        async(cb, () -> {
            JSONObject body = new JSONObject();
            if (firstName != null) body.put("first_name", firstName);
            if (lastName != null) body.put("last_name", lastName);
            return NucleusUser.fromJson(patch("/v1/user", body));
        });
    }

    public void updatePassword(String currentPassword, String newPassword, NucleusCallback<Void> cb) {
        async(cb, () -> {
            put("/v1/user/password", new JSONObject().put("current_password", currentPassword).put("new_password", newPassword));
            return null;
        });
    }

    // --- Sessions ---
    public void refreshSession(String refreshToken, NucleusCallback<NucleusSession> cb) {
        async(cb, () -> NucleusSession.fromJson(post("/v1/sessions/refresh", new JSONObject().put("refresh_token", refreshToken))));
    }

    public NucleusSession refreshSessionSync(String refreshToken) throws NucleusException, org.json.JSONException {
        return NucleusSession.fromJson(post("/v1/sessions/refresh", new JSONObject().put("refresh_token", refreshToken)));
    }

    // --- Organizations ---
    public void getOrganizations(NucleusCallback<List<NucleusOrganization>> cb) {
        async(cb, () -> {
            JSONObject json = get("/v1/organizations");
            List<NucleusOrganization> list = new ArrayList<>();
            JSONArray arr = json.optJSONArray("organizations");
            if (arr == null && json.names() == null) return list;
            if (arr != null) { for (int i = 0; i < arr.length(); i++) list.add(NucleusOrganization.fromJson(arr.getJSONObject(i))); }
            return list;
        });
    }

    public void createOrganization(String name, String slug, NucleusCallback<NucleusOrganization> cb) {
        async(cb, () -> NucleusOrganization.fromJson(post("/v1/organizations", new JSONObject().put("name", name).put("slug", slug))));
    }

    public void getMembers(String orgId, NucleusCallback<List<NucleusMember>> cb) {
        async(cb, () -> {
            JSONObject json = get("/v1/organizations/" + orgId + "/members");
            List<NucleusMember> list = new ArrayList<>();
            JSONArray arr = json.optJSONArray("members");
            if (arr != null) { for (int i = 0; i < arr.length(); i++) list.add(NucleusMember.fromJson(arr.getJSONObject(i))); }
            return list;
        });
    }

    public void removeMember(String orgId, String memberId, NucleusCallback<Void> cb) {
        async(cb, () -> { delete("/v1/organizations/" + orgId + "/members/" + memberId); return null; });
    }

    public void createInvitation(String orgId, String email, String role, NucleusCallback<NucleusInvitation> cb) {
        async(cb, () -> NucleusInvitation.fromJson(post("/v1/organizations/" + orgId + "/invitations", new JSONObject().put("email", email).put("role", role))));
    }

    // --- Result types ---
    public static class AuthResult {
        public final NucleusUser user;
        public final NucleusSession session;
        public AuthResult(NucleusUser user, NucleusSession session) { this.user = user; this.session = session; }
    }

    public static class MfaSetupResult {
        public final String secret;
        public final String qrUri;
        public MfaSetupResult(String secret, String qrUri) { this.secret = secret; this.qrUri = qrUri; }
    }
}
