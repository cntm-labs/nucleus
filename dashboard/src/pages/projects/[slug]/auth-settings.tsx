import { useState, useEffect } from 'react'
import { useParams } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { projectsApi, type ProjectSettings } from '../../../lib/api'

export function AuthSettingsPage() {
  const { slug } = useParams()
  const queryClient = useQueryClient()
  const [localSettings, setLocalSettings] = useState<ProjectSettings | null>(null)
  const [saveSuccess, setSaveSuccess] = useState(false)

  const { data: settings, isLoading, error } = useQuery({
    queryKey: ['project-settings', slug],
    queryFn: () => projectsApi.getSettings(slug!),
    enabled: !!slug,
  })

  const mutation = useMutation({
    mutationFn: (newSettings: Partial<ProjectSettings>) => 
      projectsApi.updateSettings(slug!, newSettings),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['project-settings', slug] })
      setSaveSuccess(true)
      setTimeout(() => setSaveSuccess(false), 3000)
    },
  })

  useEffect(() => {
    if (settings) {
      setLocalSettings(settings)
    }
  }, [settings])

  if (isLoading) return <div className="p-8 text-center text-gray-500 text-sm">Loading settings...</div>
  if (error) return <div className="p-8 text-center text-red-500 text-sm font-medium">Error loading settings: {(error as Error).message}</div>
  if (!localSettings) return null

  const handleToggleMethod = (key: string, enabled: boolean) => {
    setLocalSettings({
      ...localSettings,
      settings: {
        ...localSettings.settings,
        [key]: enabled
      }
    })
  }

  const handleSave = () => {
    mutation.mutate(localSettings)
  }

  const methods = [
    { key: 'password_enabled', label: 'Email + Password', desc: 'Traditional email/password authentication' },
    { key: 'magic_link_enabled', label: 'Magic Link', desc: 'Passwordless sign-in via email link' },
    { key: 'otp_enabled', label: 'One-Time Password', desc: 'SMS or email verification code' },
    { key: 'passkeys_enabled', label: 'Passkeys', desc: 'WebAuthn/FIDO2 biometric authentication' },
    { key: 'mfa_enabled', label: 'Multi-Factor Authentication', desc: 'Require a second factor during sign-in' },
  ]

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold">Authentication Settings</h1>
        {saveSuccess && (
          <span className="text-green-600 text-sm font-medium bg-green-50 px-3 py-1 rounded-full animate-fade-in">
            Settings saved successfully!
          </span>
        )}
      </div>

      <div className="space-y-6">
        {/* Auth methods */}
        <div className="bg-white rounded-xl border border-gray-200 p-6 shadow-sm">
          <h2 className="text-lg font-semibold mb-4 text-gray-900">Sign-in Methods</h2>
          <div className="space-y-3">
            {methods.map(method => (
              <label
                key={method.key}
                htmlFor={`auth-method-${method.key}`}
                className="flex items-center justify-between p-4 rounded-xl border border-gray-100 hover:border-nucleus-200 hover:bg-gray-50 transition-all cursor-pointer"
              >
                <div>
                  <p className="font-semibold text-sm text-gray-900">{method.label}</p>
                  <p className="text-xs text-gray-500 mt-0.5">{method.desc}</p>
                </div>
                <div className="relative inline-flex items-center cursor-pointer">
                  <input
                    id={`auth-method-${method.key}`}
                    type="checkbox"
                    className="sr-only peer"
                    checked={!!localSettings.settings[method.key]}
                    onChange={e => handleToggleMethod(method.key, e.target.checked)}
                  />
                  <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-nucleus-600"></div>
                </div>
              </label>
            ))}
          </div>
        </div>

        {/* Session settings */}
        <div className="bg-white rounded-xl border border-gray-200 p-6 shadow-sm">
          <h2 className="text-lg font-semibold mb-4 text-gray-900">Session Configuration</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div>
              <label htmlFor="session-ttl" className="block text-sm font-semibold text-gray-700 mb-2">Session TTL (seconds)</label>
              <input 
                id="session-ttl" 
                type="number" 
                value={localSettings.session_ttl}
                onChange={e => setLocalSettings({...localSettings, session_ttl: parseInt(e.target.value) || 0})}
                className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-nucleus-500 focus:border-transparent outline-none transition-all" 
              />
              <p className="text-xs text-gray-400 mt-2 font-medium">{Math.floor(localSettings.session_ttl / 86400)} days</p>
            </div>
            <div>
              <label htmlFor="jwt-lifetime" className="block text-sm font-semibold text-gray-700 mb-2">JWT Lifetime (seconds)</label>
              <input 
                id="jwt-lifetime" 
                type="number" 
                value={localSettings.jwt_lifetime}
                onChange={e => setLocalSettings({...localSettings, jwt_lifetime: parseInt(e.target.value) || 0})}
                className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-nucleus-500 focus:border-transparent outline-none transition-all" 
              />
              <p className="text-xs text-gray-400 mt-2 font-medium">{localSettings.jwt_lifetime / 60} minutes</p>
            </div>
          </div>
        </div>

        <div className="flex justify-end pt-2">
          <button 
            onClick={handleSave}
            disabled={mutation.isPending}
            className="px-8 py-2.5 bg-nucleus-600 text-white rounded-xl hover:bg-nucleus-700 font-semibold text-sm shadow-lg shadow-nucleus-200 disabled:opacity-50 transition-all"
          >
            {mutation.isPending ? 'Saving...' : 'Save Changes'}
          </button>
        </div>
      </div>
    </div>
  )
}
