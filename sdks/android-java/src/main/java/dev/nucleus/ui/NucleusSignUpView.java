package dev.nucleus.ui;

import android.content.Context;
import android.graphics.drawable.GradientDrawable;
import android.util.AttributeSet;
import android.view.Gravity;
import android.widget.*;
import dev.nucleus.*;
import dev.nucleus.network.ApiClient;

public class NucleusSignUpView extends LinearLayout {
    private EditText firstNameInput, lastNameInput, emailInput, passwordInput;
    private Button signUpButton;
    private TextView errorText;
    private SignUpListener listener;
    private NucleusAppearance appearance = new NucleusAppearance.Builder().build();

    public interface SignUpListener { void onSignUp(); }

    public NucleusSignUpView(Context c) { super(c); init(); }
    public NucleusSignUpView(Context c, AttributeSet a) { super(c, a); init(); }

    public void setListener(SignUpListener listener) { this.listener = listener; }
    public void setAppearance(NucleusAppearance appearance) { this.appearance = appearance; init(); }

    private void init() {
        removeAllViews();
        setOrientation(VERTICAL);
        setPadding(dp(24), dp(24), dp(24), dp(24));

        GradientDrawable bg = new GradientDrawable();
        bg.setColor(appearance.getBackgroundColor());
        bg.setCornerRadius(dp(appearance.getCornerRadius()));
        bg.setStroke(1, appearance.getBorderColor());
        setBackground(bg);

        TextView title = new TextView(getContext());
        title.setText("Create Account");
        title.setTextSize(20);
        title.setTextColor(appearance.getTextColor());
        title.setGravity(Gravity.CENTER);
        addView(title, new LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.WRAP_CONTENT));

        errorText = new TextView(getContext());
        errorText.setTextColor(appearance.getErrorColor());
        errorText.setVisibility(GONE);
        addView(errorText, lp(dp(12)));

        LinearLayout nameRow = new LinearLayout(getContext());
        nameRow.setOrientation(HORIZONTAL);
        firstNameInput = new EditText(getContext()); firstNameInput.setHint("First Name");
        lastNameInput = new EditText(getContext()); lastNameInput.setHint("Last Name");
        LayoutParams half = new LayoutParams(0, LayoutParams.WRAP_CONTENT, 1f);
        half.rightMargin = dp(4);
        nameRow.addView(firstNameInput, half);
        LayoutParams half2 = new LayoutParams(0, LayoutParams.WRAP_CONTENT, 1f);
        half2.leftMargin = dp(4);
        nameRow.addView(lastNameInput, half2);
        addView(nameRow, lp(dp(8)));

        emailInput = new EditText(getContext());
        emailInput.setHint("Email");
        emailInput.setInputType(android.text.InputType.TYPE_TEXT_VARIATION_EMAIL_ADDRESS);
        addView(emailInput, lp(dp(8)));

        passwordInput = new EditText(getContext());
        passwordInput.setHint("Password");
        passwordInput.setInputType(android.text.InputType.TYPE_CLASS_TEXT | android.text.InputType.TYPE_TEXT_VARIATION_PASSWORD);
        addView(passwordInput, lp(dp(16)));

        signUpButton = new Button(getContext());
        signUpButton.setText("Sign Up");
        signUpButton.setBackgroundColor(appearance.getPrimaryColor());
        signUpButton.setTextColor(0xFFFFFFFF);
        signUpButton.setOnClickListener(v -> handleSignUp());
        addView(signUpButton, lp(0));
    }

    private void handleSignUp() {
        String email = emailInput.getText().toString().trim();
        String password = passwordInput.getText().toString();
        if (email.isEmpty() || password.isEmpty()) { showError("Email and password required"); return; }

        signUpButton.setEnabled(false);
        errorText.setVisibility(GONE);
        String fn = firstNameInput.getText().toString().trim();
        String ln = lastNameInput.getText().toString().trim();

        Nucleus.getAuth().signUp(email, password, fn.isEmpty() ? null : fn, ln.isEmpty() ? null : ln,
            new NucleusCallback<ApiClient.AuthResult>() {
                @Override public void onSuccess(ApiClient.AuthResult result) {
                    post(() -> { signUpButton.setEnabled(true); if (listener != null) listener.onSignUp(); });
                }
                @Override public void onError(NucleusException error) {
                    post(() -> { signUpButton.setEnabled(true); showError(error.getMessage()); });
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
