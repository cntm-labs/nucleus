package dev.nucleus;
import android.content.Context;
public final class Nucleus {
    private static NucleusAuth auth;
    private Nucleus() {}
    public static void configure(Context context, String publishableKey) {
        auth = new NucleusAuth();
    }
    public static NucleusAuth getAuth() {
        if (auth == null) throw new IllegalStateException("Call Nucleus.configure() first");
        return auth;
    }
}
