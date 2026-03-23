package dev.nucleus.ui;

import android.content.Context;
import android.graphics.drawable.GradientDrawable;
import android.util.AttributeSet;
import android.view.Gravity;
import android.widget.*;
import dev.nucleus.*;
import dev.nucleus.network.ApiClient;

public class NucleusSignInView extends LinearLayout {
    private EditText emailInput, passwordInput;
    private Button signInButton;
    private TextView errorText;
    private SignInListener listener;
    private NucleusAppearance appearance = new NucleusAppearance.Builder().build();

    public interface SignInListener { void onSignIn(); }

    public NucleusSignInView(Context c) { super(c); init(); }
    public NucleusSignInView(Context c, AttributeSet a) { super(c, a); init(); }

    public void setListener(SignInListener listener) { this.listener = listener; }
    public void setAppearance(NucleusAppearance appearance) { this.appearance = appearance; init(); }

    private void init() {
        removeAllViews();
        setOrientation(VERTICAL);
        setPadding(dp(24), dp(24), dp(24), dp(24));
        setGravity(Gravity.CENTER_HORIZONTAL);

        GradientDrawable bg = new GradientDrawable();
        bg.setColor(appearance.getBackgroundColor());
        bg.setCornerRadius(dp(appearance.getCornerRadius()));
        bg.setStroke(1, appearance.getBorderColor());
        setBackground(bg);

        TextView title = new TextView(getContext());
        title.setText("Sign In");
        title.setTextSize(20);
        title.setTextColor(appearance.getTextColor());
        title.setGravity(Gravity.CENTER);
        addView(title, new LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.WRAP_CONTENT));

        errorText = new TextView(getContext());
        errorText.setTextColor(appearance.getErrorColor());
        errorText.setVisibility(GONE);
        errorText.setPadding(dp(8), dp(8), dp(8), dp(8));
        addView(errorText, lp(dp(16)));

        emailInput = new EditText(getContext());
        emailInput.setHint("Email");
        emailInput.setInputType(android.text.InputType.TYPE_TEXT_VARIATION_EMAIL_ADDRESS);
        addView(emailInput, lp(dp(8)));

        passwordInput = new EditText(getContext());
        passwordInput.setHint("Password");
        passwordInput.setInputType(android.text.InputType.TYPE_CLASS_TEXT | android.text.InputType.TYPE_TEXT_VARIATION_PASSWORD);
        addView(passwordInput, lp(dp(16)));

        signInButton = new Button(getContext());
        signInButton.setText("Sign In");
        signInButton.setBackgroundColor(appearance.getPrimaryColor());
        signInButton.setTextColor(0xFFFFFFFF);
        signInButton.setOnClickListener(v -> handleSignIn());
        addView(signInButton, lp(0));
    }

    private void handleSignIn() {
        String email = emailInput.getText().toString().trim();
        String password = passwordInput.getText().toString();
        if (email.isEmpty() || password.isEmpty()) { showError("Email and password required"); return; }

        signInButton.setEnabled(false);
        errorText.setVisibility(GONE);

        Nucleus.getAuth().signIn(email, password, new NucleusCallback<ApiClient.AuthResult>() {
            @Override public void onSuccess(ApiClient.AuthResult result) {
                post(() -> { signInButton.setEnabled(true); if (listener != null) listener.onSignIn(); });
            }
            @Override public void onError(NucleusException error) {
                post(() -> { signInButton.setEnabled(true); showError(error.getMessage()); });
            }
        });
    }

    private void showError(String msg) { errorText.setText(msg); errorText.setVisibility(VISIBLE); }
    private int dp(float v) { return (int) (v * getResources().getDisplayMetrics().density); }
    private LayoutParams lp(int bottomMargin) {
        LayoutParams p = new LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.WRAP_CONTENT);
        p.bottomMargin = bottomMargin;
        return p;
    }
}
