package dev.nucleus.ui;

import android.content.Context;
import android.graphics.Canvas;
import android.graphics.Paint;
import android.util.AttributeSet;
import android.view.View;
import android.widget.PopupMenu;
import dev.nucleus.Nucleus;
import dev.nucleus.NucleusUser;

public class NucleusUserButton extends View {
    private final Paint paint = new Paint(Paint.ANTI_ALIAS_FLAG);
    private final Paint textPaint = new Paint(Paint.ANTI_ALIAS_FLAG);
    private SignOutListener listener;

    public interface SignOutListener { void onSignOut(); }

    public NucleusUserButton(Context c) { super(c); init(); }
    public NucleusUserButton(Context c, AttributeSet a) { super(c, a); init(); }

    public void setSignOutListener(SignOutListener listener) { this.listener = listener; }

    private void init() {
        paint.setColor(0xFF4C6EF5);
        textPaint.setColor(0xFFFFFFFF);
        textPaint.setTextSize(dp(14));
        textPaint.setTextAlign(Paint.Align.CENTER);

        setOnClickListener(v -> showMenu());
    }

    @Override
    protected void onMeasure(int widthMeasureSpec, int heightMeasureSpec) {
        int size = dp(36);
        setMeasuredDimension(size, size);
    }

    @Override
    protected void onDraw(Canvas canvas) {
        float cx = getWidth() / 2f, cy = getHeight() / 2f, r = Math.min(cx, cy);
        canvas.drawCircle(cx, cy, r, paint);

        NucleusUser user = Nucleus.isConfigured() ? Nucleus.getAuth().getUser() : null;
        if (user != null) {
            String initials = getInitials(user);
            canvas.drawText(initials, cx, cy + textPaint.getTextSize() / 3f, textPaint);
        }
    }

    private String getInitials(NucleusUser user) {
        StringBuilder sb = new StringBuilder();
        if (user.getFirstName() != null && !user.getFirstName().isEmpty()) sb.append(user.getFirstName().charAt(0));
        if (user.getLastName() != null && !user.getLastName().isEmpty()) sb.append(user.getLastName().charAt(0));
        if (sb.length() == 0 && user.getEmail() != null) sb.append(Character.toUpperCase(user.getEmail().charAt(0)));
        return sb.toString().toUpperCase();
    }

    private void showMenu() {
        PopupMenu popup = new PopupMenu(getContext(), this);
        NucleusUser user = Nucleus.getAuth().getUser();
        if (user != null) popup.getMenu().add(user.getFullName()).setEnabled(false);
        popup.getMenu().add("Sign Out").setOnMenuItemClickListener(item -> {
            Nucleus.getAuth().signOut();
            if (listener != null) listener.onSignOut();
            invalidate();
            return true;
        });
        popup.show();
    }

    private int dp(float v) { return (int) (v * getResources().getDisplayMetrics().density); }
}
