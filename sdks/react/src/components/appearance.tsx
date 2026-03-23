import React from 'react'

const defaultStyles: Record<string, React.CSSProperties> = {
  card: {
    maxWidth: 400, margin: '0 auto', padding: 24,
    background: 'var(--nucleus-bg, white)',
    borderRadius: 'var(--nucleus-radius, 8px)',
    border: '1px solid var(--nucleus-border, #e5e7eb)',
    fontFamily: 'var(--nucleus-font, system-ui, -apple-system, sans-serif)',
    color: 'var(--nucleus-text, #111827)',
  },
  title: {
    fontSize: 24, fontWeight: 'bold', marginBottom: 16, textAlign: 'center' as const,
  },
  input: {
    width: '100%', padding: '10px 12px', marginBottom: 8,
    border: '1px solid var(--nucleus-border, #d1d5db)',
    borderRadius: 'var(--nucleus-radius, 6px)',
    fontSize: 14, boxSizing: 'border-box' as const,
    background: 'var(--nucleus-input-bg, white)',
    color: 'var(--nucleus-text, #111827)',
  },
  button: {
    width: '100%', padding: '10px 16px',
    background: 'var(--nucleus-primary, #4c6ef5)',
    color: 'var(--nucleus-primary-text, white)',
    border: 'none',
    borderRadius: 'var(--nucleus-radius, 6px)',
    cursor: 'pointer', fontSize: 14, fontWeight: 600,
  },
  secondaryButton: {
    width: '100%', padding: '10px 16px',
    background: 'var(--nucleus-secondary, #f3f4f6)',
    color: 'var(--nucleus-text, #111827)',
    border: '1px solid var(--nucleus-border, #d1d5db)',
    borderRadius: 'var(--nucleus-radius, 6px)',
    cursor: 'pointer', fontSize: 14, fontWeight: 500,
    marginBottom: 8,
    display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 8,
  },
  error: {
    padding: '8px 12px', marginBottom: 12,
    background: '#fef2f2', color: '#dc2626',
    borderRadius: 'var(--nucleus-radius, 6px)', fontSize: 14,
  },
  divider: {
    display: 'flex', alignItems: 'center', gap: 12,
    margin: '16px 0', color: '#9ca3af', fontSize: 13,
  },
  dividerLine: {
    flex: 1, height: 1, background: '#e5e7eb',
  },
  link: {
    color: 'var(--nucleus-primary, #4c6ef5)',
    cursor: 'pointer', fontSize: 14, textDecoration: 'none',
  },
}

export function useStyles() {
  return defaultStyles
}

export function Divider({ text }: { text: string }) {
  const s = defaultStyles
  return (
    <div style={s.divider}>
      <div style={s.dividerLine} />
      <span>{text}</span>
      <div style={s.dividerLine} />
    </div>
  )
}
