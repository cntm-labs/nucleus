import { test, expect } from '@playwright/test'

const ALL_PROVIDERS = [
  { id: 'google', label: 'Continue with Google' },
  { id: 'github', label: 'Continue with Github' },
  { id: 'apple', label: 'Continue with Apple' },
  { id: 'microsoft', label: 'Continue with Microsoft' },
  { id: 'facebook', label: 'Continue with Facebook' },
  { id: 'discord', label: 'Continue with Discord' },
  { id: 'twitter', label: 'Continue with Twitter' },
  { id: 'linkedin', label: 'Continue with Linkedin' },
  { id: 'slack', label: 'Continue with Slack' },
]

test.describe('OAuth providers — SignIn', () => {
  test('renders all 9 OAuth provider buttons', async ({ page }) => {
    await page.goto('/oauth/sign-in')
    for (const provider of ALL_PROVIDERS) {
      await expect(
        page.getByRole('button', { name: provider.label })
      ).toBeVisible()
    }
  })

  test('renders exactly 9 OAuth buttons', async ({ page }) => {
    await page.goto('/oauth/sign-in')
    const buttons = page.locator('button').filter({ hasText: /Continue with/ })
    await expect(buttons).toHaveCount(9)
  })

  test('OAuth buttons are clickable', async ({ page }) => {
    await page.goto('/oauth/sign-in')
    for (const provider of ALL_PROVIDERS) {
      const button = page.getByRole('button', { name: provider.label })
      await expect(button).toBeEnabled()
    }
  })

  test('form fields still render alongside OAuth buttons', async ({ page }) => {
    await page.goto('/oauth/sign-in')
    await expect(page.getByPlaceholder('Email')).toBeVisible()
    await expect(page.getByPlaceholder('Password')).toBeVisible()
    await expect(page.getByText('or', { exact: true })).toBeVisible()
  })
})

test.describe('OAuth providers — SignUp', () => {
  test('renders all 9 OAuth provider buttons', async ({ page }) => {
    await page.goto('/oauth/sign-up')
    for (const provider of ALL_PROVIDERS) {
      await expect(
        page.getByRole('button', { name: provider.label })
      ).toBeVisible()
    }
  })

  test('renders exactly 9 OAuth buttons', async ({ page }) => {
    await page.goto('/oauth/sign-up')
    const buttons = page.locator('button').filter({ hasText: /Continue with/ })
    await expect(buttons).toHaveCount(9)
  })

  test('form fields still render alongside OAuth buttons', async ({ page }) => {
    await page.goto('/oauth/sign-up')
    await expect(page.getByPlaceholder('Email')).toBeVisible()
    await expect(page.getByPlaceholder('Password')).toBeVisible()
    await expect(page.getByPlaceholder('First Name')).toBeVisible()
    await expect(page.getByPlaceholder('Last Name')).toBeVisible()
  })
})
