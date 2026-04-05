import { test, expect } from '@playwright/test'

test.describe('Thai locale (i18n)', () => {
  test.describe('SignIn component', () => {
    test('renders title in Thai', async ({ page }) => {
      await page.goto('/sign-in/th')
      await expect(page.getByText('เข้าสู่ระบบ').first()).toBeVisible()
    })

    test('renders form labels in Thai', async ({ page }) => {
      await page.goto('/sign-in/th')
      await expect(page.getByPlaceholder('อีเมล')).toBeVisible()
      await expect(page.getByPlaceholder('รหัสผ่าน')).toBeVisible()
    })

    test('renders submit button in Thai', async ({ page }) => {
      await page.goto('/sign-in/th')
      await expect(page.getByRole('button', { name: 'เข้าสู่ระบบ' })).toBeVisible()
    })

    test('English locale renders correctly for comparison', async ({ page }) => {
      await page.goto('/sign-in/en')
      await expect(page.getByText('Sign In').first()).toBeVisible()
      await expect(page.getByPlaceholder('Email')).toBeVisible()
      await expect(page.getByPlaceholder('Password')).toBeVisible()
    })
  })

  test.describe('SignUp component', () => {
    test('renders title in Thai', async ({ page }) => {
      await page.goto('/sign-up/th')
      await expect(page.getByText('สร้างบัญชี')).toBeVisible()
    })

    test('renders form fields in Thai', async ({ page }) => {
      await page.goto('/sign-up/th')
      await expect(page.getByPlaceholder('ชื่อ')).toBeVisible()
      await expect(page.getByPlaceholder('นามสกุล')).toBeVisible()
      await expect(page.getByPlaceholder('อีเมล')).toBeVisible()
      await expect(page.getByPlaceholder('รหัสผ่าน')).toBeVisible()
    })

    test('renders submit button in Thai', async ({ page }) => {
      await page.goto('/sign-up/th')
      await expect(page.getByRole('button', { name: 'สมัครสมาชิก' })).toBeVisible()
    })

    test('English locale renders correctly for comparison', async ({ page }) => {
      await page.goto('/sign-up/en')
      await expect(page.getByText('Create Account')).toBeVisible()
      await expect(page.getByRole('button', { name: 'Sign Up' })).toBeVisible()
    })
  })

  test.describe('OrgSwitcher component', () => {
    test('renders button in Thai', async ({ page }) => {
      await page.goto('/org-switcher/th')
      await expect(page.getByRole('button', { name: 'เลือกองค์กร' })).toBeVisible()
    })

    test('shows Thai text in dropdown', async ({ page }) => {
      await page.goto('/org-switcher/th')
      await page.getByRole('button', { name: 'เลือกองค์กร' }).click()
      await expect(page.getByText('ไม่มีองค์กร')).toBeVisible()
      await expect(page.getByText('+ สร้างองค์กร')).toBeVisible()
    })

    test('shows create form in Thai', async ({ page }) => {
      await page.goto('/org-switcher/th')
      await page.getByRole('button', { name: 'เลือกองค์กร' }).click()
      await page.getByText('+ สร้างองค์กร').click()
      await expect(page.getByPlaceholder('ชื่อ')).toBeVisible()
      await expect(page.getByRole('button', { name: 'ยกเลิก' })).toBeVisible()
      await expect(page.getByRole('button', { name: 'สร้าง' })).toBeVisible()
    })

    test('English locale renders correctly for comparison', async ({ page }) => {
      await page.goto('/org-switcher/en')
      await expect(page.getByRole('button', { name: 'Select Organization' })).toBeVisible()
    })
  })

  test.describe('OrgProfile component', () => {
    test('renders no-org message in Thai', async ({ page }) => {
      await page.goto('/org-profile/th')
      await expect(page.getByText('ไม่ได้เลือกองค์กร')).toBeVisible()
    })
  })
})
