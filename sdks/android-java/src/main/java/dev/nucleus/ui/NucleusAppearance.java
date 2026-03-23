package dev.nucleus.ui;

import android.graphics.Color;

public class NucleusAppearance {
    private int primaryColor = Color.parseColor("#4C6EF5");
    private int backgroundColor = Color.WHITE;
    private int textColor = Color.parseColor("#111827");
    private int errorColor = Color.parseColor("#DC2626");
    private int borderColor = Color.parseColor("#D1D5DB");
    private float cornerRadius = 8f;

    public static class Builder {
        private final NucleusAppearance appearance = new NucleusAppearance();
        public Builder primaryColor(int color) { appearance.primaryColor = color; return this; }
        public Builder backgroundColor(int color) { appearance.backgroundColor = color; return this; }
        public Builder textColor(int color) { appearance.textColor = color; return this; }
        public Builder errorColor(int color) { appearance.errorColor = color; return this; }
        public Builder borderColor(int color) { appearance.borderColor = color; return this; }
        public Builder cornerRadius(float radius) { appearance.cornerRadius = radius; return this; }
        public NucleusAppearance build() { return appearance; }
    }

    public int getPrimaryColor() { return primaryColor; }
    public int getBackgroundColor() { return backgroundColor; }
    public int getTextColor() { return textColor; }
    public int getErrorColor() { return errorColor; }
    public int getBorderColor() { return borderColor; }
    public float getCornerRadius() { return cornerRadius; }
}
