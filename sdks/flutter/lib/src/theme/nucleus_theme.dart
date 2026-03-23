import 'package:flutter/material.dart';

class NucleusTheme {
  final Color primaryColor;
  final Color backgroundColor;
  final Color textColor;
  final Color errorColor;
  final Color borderColor;
  final double borderRadius;
  final TextStyle? titleStyle;
  final TextStyle? bodyStyle;

  const NucleusTheme({
    this.primaryColor = const Color(0xFF4C6EF5),
    this.backgroundColor = Colors.white,
    this.textColor = const Color(0xFF111827),
    this.errorColor = const Color(0xFFDC2626),
    this.borderColor = const Color(0xFFD1D5DB),
    this.borderRadius = 8.0,
    this.titleStyle,
    this.bodyStyle,
  });

  InputDecoration inputDecoration(String hint) => InputDecoration(
    hintText: hint,
    contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
    border: OutlineInputBorder(
      borderRadius: BorderRadius.circular(borderRadius),
      borderSide: BorderSide(color: borderColor),
    ),
    enabledBorder: OutlineInputBorder(
      borderRadius: BorderRadius.circular(borderRadius),
      borderSide: BorderSide(color: borderColor),
    ),
  );

  ButtonStyle get primaryButtonStyle => ElevatedButton.styleFrom(
    backgroundColor: primaryColor,
    foregroundColor: Colors.white,
    minimumSize: const Size(double.infinity, 44),
    shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(borderRadius)),
  );

  ButtonStyle get secondaryButtonStyle => OutlinedButton.styleFrom(
    foregroundColor: textColor,
    minimumSize: const Size(double.infinity, 44),
    side: BorderSide(color: borderColor),
    shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(borderRadius)),
  );
}
