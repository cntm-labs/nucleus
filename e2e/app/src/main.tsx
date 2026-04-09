import React from 'react'
import { createRoot } from 'react-dom/client'
import { BrowserRouter, Routes, Route, Link } from 'react-router-dom'
import { SignInEnglish, SignInThai } from './pages/sign-in'
import { SignUpEnglish, SignUpThai } from './pages/sign-up'
import { OrgSwitcherEnglish, OrgSwitcherThai } from './pages/org-switcher'
import { OrgProfileThai } from './pages/org-profile'
import { OAuthSignIn, OAuthSignUp } from './pages/oauth'

function Home() {
  return (
    <div style={{ padding: 24 }}>
      <h1>Nucleus E2E Test App</h1>
      <nav>
        <ul>
          <li><Link to="/sign-in/en">SignIn (English)</Link></li>
          <li><Link to="/sign-in/th">SignIn (Thai)</Link></li>
          <li><Link to="/sign-up/en">SignUp (English)</Link></li>
          <li><Link to="/sign-up/th">SignUp (Thai)</Link></li>
          <li><Link to="/org-switcher/en">OrgSwitcher (English)</Link></li>
          <li><Link to="/org-switcher/th">OrgSwitcher (Thai)</Link></li>
          <li><Link to="/org-profile/th">OrgProfile (Thai)</Link></li>
          <li><Link to="/oauth/sign-in">OAuth SignIn (all providers)</Link></li>
          <li><Link to="/oauth/sign-up">OAuth SignUp (all providers)</Link></li>
        </ul>
      </nav>
    </div>
  )
}

createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/sign-in/en" element={<SignInEnglish />} />
        <Route path="/sign-in/th" element={<SignInThai />} />
        <Route path="/sign-up/en" element={<SignUpEnglish />} />
        <Route path="/sign-up/th" element={<SignUpThai />} />
        <Route path="/org-switcher/en" element={<OrgSwitcherEnglish />} />
        <Route path="/org-switcher/th" element={<OrgSwitcherThai />} />
        <Route path="/org-profile/th" element={<OrgProfileThai />} />
        <Route path="/oauth/sign-in" element={<OAuthSignIn />} />
        <Route path="/oauth/sign-up" element={<OAuthSignUp />} />
      </Routes>
    </BrowserRouter>
  </React.StrictMode>
)
