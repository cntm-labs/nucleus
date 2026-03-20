package dev.nucleus;
public class NucleusAuth {
    private NucleusUser user;
    public NucleusUser getUser() { return user; }
    public boolean isSignedIn() { return user != null; }
    public void signIn(String email, String password, NucleusCallback<NucleusUser> cb) { cb.onError(new NucleusException("Not implemented")); }
    public void signOut() { user = null; }
    public void getToken(NucleusCallback<String> cb) { cb.onError(new NucleusException("Not implemented")); }
}
