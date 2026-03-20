package dev.nucleus;
import android.content.Context;
public final class Nucleus {
    private static NucleusAuth auth;
    private Nucleus() {}
    private static boolean warned = false;
    public static void configure(Context context, String publishableKey) {
        if (!warned) {
            String version = "0.1.0-dev.1";
            if (version.contains("dev")) {
                android.util.Log.w("Nucleus", "WARNING: You are using a dev preview (" + version + "). Do not use in production.");
            }
            warned = true;
        }
        auth = new NucleusAuth();
    }
    public static NucleusAuth getAuth() {
        if (auth == null) throw new IllegalStateException("Call Nucleus.configure() first");
        return auth;
    }
}
