import { describe, it, expect } from 'vitest'
import { validateSignIn, validateSignUp, ValidationError, emailSchema, passwordSchema } from '../src/validation'

describe('emailSchema', () => {
  it('rejects empty email', () => {
    expect(() => emailSchema.parse('')).toThrow()
  })

  it('rejects invalid email', () => {
    expect(() => emailSchema.parse('notanemail')).toThrow()
  })

  it('accepts valid email', () => {
    expect(emailSchema.parse('a@b.com')).toBe('a@b.com')
  })

  it('trims whitespace', () => {
    expect(emailSchema.parse('  a@b.com  ')).toBe('a@b.com')
  })
})

describe('passwordSchema', () => {
  it('rejects empty password', () => {
    expect(() => passwordSchema.parse('')).toThrow()
  })

  it('rejects short password', () => {
    expect(() => passwordSchema.parse('abc')).toThrow()
  })

  it('accepts valid password', () => {
    expect(passwordSchema.parse('password123')).toBe('password123')
  })
})

describe('validateSignIn', () => {
  it('throws ValidationError for invalid email', () => {
    expect(() => validateSignIn('bad', 'password123')).toThrow(ValidationError)
  })

  it('throws ValidationError for short password', () => {
    expect(() => validateSignIn('a@b.com', 'short')).toThrow(ValidationError)
  })

  it('does not throw for valid input', () => {
    expect(() => validateSignIn('a@b.com', 'password123')).not.toThrow()
  })

  it('includes issues array on ValidationError', () => {
    try {
      validateSignIn('', '')
    } catch (err) {
      expect(err).toBeInstanceOf(ValidationError)
      expect((err as ValidationError).issues.length).toBeGreaterThan(0)
    }
  })
})

describe('validateSignUp', () => {
  it('validates email and password', () => {
    expect(() => validateSignUp('bad', 'short')).toThrow(ValidationError)
  })

  it('does not throw for valid input', () => {
    expect(() => validateSignUp('a@b.com', 'password123')).not.toThrow()
  })
})
