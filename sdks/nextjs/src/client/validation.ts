import { z } from 'zod'

export const emailSchema = z
  .string({ required_error: 'Email is required' })
  .trim()
  .min(1, 'Email is required')
  .email('Invalid email format')

export const passwordSchema = z
  .string({ required_error: 'Password is required' })
  .min(1, 'Password is required')
  .min(8, 'Password must be at least 8 characters')

export const signInSchema = z.object({
  email: emailSchema,
  password: passwordSchema,
})

export const signUpSchema = z.object({
  email: emailSchema,
  password: passwordSchema,
  firstName: z.string().optional(),
  lastName: z.string().optional(),
})

export type SignInInput = z.infer<typeof signInSchema>
export type SignUpInput = z.infer<typeof signUpSchema>

export class ValidationError extends Error {
  public issues: z.ZodIssue[]

  constructor(result: z.ZodError) {
    const first = result.issues[0]
    super(first.message)
    this.name = 'ValidationError'
    this.issues = result.issues
  }
}

export function validateSignIn(email: string, password: string): void {
  const result = signInSchema.safeParse({ email, password })
  if (!result.success) throw new ValidationError(result.error)
}

export function validateSignUp(email: string, password: string): void {
  const result = signUpSchema.safeParse({ email, password })
  if (!result.success) throw new ValidationError(result.error)
}
