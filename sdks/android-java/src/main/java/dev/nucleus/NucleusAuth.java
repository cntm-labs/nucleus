package dev.nucleus;

import dev.nucleus.model.*;
import dev.nucleus.network.ApiClient;
import dev.nucleus.session.SessionManager;
import java.util.List;

public class NucleusAuth {
    private final ApiClient apiClient;
    private final SessionManager sessionManager;
    private NucleusUser user;
    private NucleusSession session;
    private NucleusOrganization organization;
    private AuthListener listener;

    public interface AuthListener { void onAuthStateChanged(); }

    public NucleusAuth(ApiClient apiClient, SessionManager sessionManager) {
        this.apiClient = apiClient;
        this.sessionManager = sessionManager;
    }

    public void setListener(AuthListener listener) { this.listener = listener; }
    public NucleusUser getUser() { return user; }
    public NucleusSession getSession() { return session; }
    public NucleusOrganization getOrganization() { return organization; }
    public boolean isSignedIn() { return user != null && session != null; }
    public String getToken() { return session != null ? session.getToken() : null; }

    // --- Auth ---
    public void signIn(String email, String password, NucleusCallback<ApiClient.AuthResult> cb) {
        apiClient.signIn(email, password, new NucleusCallback<ApiClient.AuthResult>() {
            @Override public void onSuccess(ApiClient.AuthResult result) {
                setAuthResult(result); cb.onSuccess(result);
            }
            @Override public void onError(NucleusException error) { cb.onError(error); }
        });
    }

    public void signUp(String email, String password, String firstName, String lastName, NucleusCallback<ApiClient.AuthResult> cb) {
        apiClient.signUp(email, password, firstName, lastName, new NucleusCallback<ApiClient.AuthResult>() {
            @Override public void onSuccess(ApiClient.AuthResult result) {
                setAuthResult(result); cb.onSuccess(result);
            }
            @Override public void onError(NucleusException error) { cb.onError(error); }
        });
    }

    public void signOut() {
        apiClient.signOut(new NucleusCallback<Void>() {
            @Override public void onSuccess(Void result) {}
            @Override public void onError(NucleusException error) {}
        });
        user = null; session = null; organization = null;
        sessionManager.clear();
        notifyListener();
    }

    // --- User Profile ---
    public void updateProfile(String firstName, String lastName, NucleusCallback<NucleusUser> cb) {
        apiClient.updateUser(firstName, lastName, new NucleusCallback<NucleusUser>() {
            @Override public void onSuccess(NucleusUser u) { user = u; notifyListener(); cb.onSuccess(u); }
            @Override public void onError(NucleusException error) { cb.onError(error); }
        });
    }

    public void updatePassword(String currentPassword, String newPassword, NucleusCallback<Void> cb) {
        apiClient.updatePassword(currentPassword, newPassword, cb);
    }

    // --- MFA ---
    public void mfaSetupTotp(NucleusCallback<ApiClient.MfaSetupResult> cb) { apiClient.mfaTotpSetup(cb); }
    public void mfaVerifyTotp(String code, NucleusCallback<Boolean> cb) { apiClient.mfaTotpVerify(code, cb); }
    public void mfaSendSms(String phone, NucleusCallback<Void> cb) { apiClient.mfaSmsSend(phone, cb); }
    public void mfaVerifySms(String code, NucleusCallback<Boolean> cb) { apiClient.mfaSmsVerify(code, cb); }

    // --- Organizations ---
    public void getOrganizations(NucleusCallback<List<NucleusOrganization>> cb) { apiClient.getOrganizations(cb); }
    public void createOrganization(String name, String slug, NucleusCallback<NucleusOrganization> cb) { apiClient.createOrganization(name, slug, cb); }
    public void setActiveOrganization(NucleusOrganization org) { this.organization = org; notifyListener(); }
    public void getMembers(String orgId, NucleusCallback<List<NucleusMember>> cb) { apiClient.getMembers(orgId, cb); }
    public void removeMember(String orgId, String memberId, NucleusCallback<Void> cb) { apiClient.removeMember(orgId, memberId, cb); }
    public void inviteMember(String orgId, String email, String role, NucleusCallback<NucleusInvitation> cb) { apiClient.createInvitation(orgId, email, role, cb); }

    // --- Internal ---
    void setUser(NucleusUser user) { this.user = user; notifyListener(); }
    void setSession(NucleusSession session) { this.session = session; }

    private void setAuthResult(ApiClient.AuthResult result) {
        this.user = result.user;
        this.session = result.session;
        sessionManager.setSession(result.session);
        notifyListener();
    }

    private void notifyListener() { if (listener != null) listener.onAuthStateChanged(); }
}
